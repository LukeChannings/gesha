use std::time::{Duration, SystemTime};

use anyhow::{anyhow, Result};
use log::{error, info};
use max31855::Max31855;
use rppal::{gpio, spi};
use tokio::{
    sync::broadcast::Sender,
    task::{self, JoinHandle},
};

use crate::config::Spi;

use super::{
    config::Config,
    state::{Event as StateEvent, Mode, TemperatureMeasurement},
};

pub struct Thermocouple {
    name: String,
    spi: rppal::spi::Spi,
    pin: rppal::gpio::OutputPin,
}

impl Thermocouple {
    pub fn read(&mut self) -> Result<f32> {
        self.spi
            .read_thermocouple(&mut self.pin, max31855::Unit::Celsius)
            .map_err(|err| {
                let error_detail = match err {
                    max31855::Error::SpiError(error) => format!("SPI {}", error),
                    max31855::Error::ChipSelectError(error) => format!("Chip select {}", error),
                    max31855::Error::Fault => "Fault".into(),
                    max31855::Error::VccShortFault => "VccShortFault".into(),
                    max31855::Error::GroundShortFault => "GroundShortFault".into(),
                    max31855::Error::MissingThermocoupleFault => "MissingThermocoupleFault".into(),
                };

                anyhow!(
                    "Error reading temp from {}. Detail: {}",
                    self.name,
                    error_detail
                )
            })
    }
}

impl TryFrom<Spi> for Thermocouple {
    type Error = anyhow::Error;

    fn try_from(value: Spi) -> Result<Self, anyhow::Error> {
        let name: String;
        let bus: spi::Bus;
        let slave_select: spi::SlaveSelect;
        let pin: rppal::gpio::OutputPin;

        // Ref: https://docs.rs/rppal/latest/rppal/spi/index.html
        match value {
            Spi::Rpi0 => {
                name = String::from("rpi0");
                bus = spi::Bus::Spi0;
                slave_select = spi::SlaveSelect::Ss0;
                pin = gpio::Gpio::new()?.get(8)?.into_output();
            }
            Spi::Rpi0_1 => {
                name = String::from("rpi0.1");
                bus = spi::Bus::Spi0;
                slave_select = spi::SlaveSelect::Ss1;
                pin = gpio::Gpio::new()?.get(7)?.into_output();
            }
            Spi::Rpi1 => {
                name = String::from("rpi1");
                bus = spi::Bus::Spi1;
                slave_select = spi::SlaveSelect::Ss0;
                pin = gpio::Gpio::new()?.get(18)?.into_output();
            }
            Spi::Rpi1_1 => {
                name = String::from("rpi1.1");
                bus = spi::Bus::Spi1;
                slave_select = spi::SlaveSelect::Ss1;
                pin = gpio::Gpio::new()?.get(17)?.into_output();
            }
            Spi::Rpi1_2 => {
                name = String::from("rpi1.2");
                bus = spi::Bus::Spi1;
                slave_select = spi::SlaveSelect::Ss2;
                pin = gpio::Gpio::new()?.get(16)?.into_output();
            }
        }

        let spi = spi::Spi::new(bus, slave_select, 1_000_000u32, spi::Mode::Mode0)?;

        Ok(Thermocouple { name, spi, pin })
    }
}

pub struct ThermocouplePoller {
    mode: Mode,
    event_tx: Sender<StateEvent>,
    config: Config,
    poller: Option<JoinHandle<()>>,
}

impl ThermocouplePoller {
    pub fn new(mode: Mode, event_tx: Sender<StateEvent>, config: Config) -> ThermocouplePoller {
        ThermocouplePoller {
            mode,
            event_tx,
            config: config.clone(),
            poller: None,
        }
    }

    pub fn poll(&mut self) -> Result<()> {
        let mut boiler: Thermocouple = self
            .config
            .boiler_spi
            .expect("Boiler SPI is not configured")
            .try_into()?;

        let mut grouphead: Thermocouple = self
            .config
            .grouphead_spi
            .expect("Group head SPI is not configured")
            .try_into()?;

        let mut thermofilter: Option<Thermocouple> = self
            .config
            .thermofilter_spi
            .map(|spi| spi.try_into().unwrap());

        let poller_tx = self.event_tx.clone();

        let interval = self.get_interval();

        info!("Thermocouple polling interval is {interval:#?}");

        let poller = task::spawn(async move {
            let mut interval = tokio::time::interval(interval);

            loop {
                let boiler_temp = boiler.read().expect("Error reading boiler temperature");
                let grouphead_temp = grouphead
                    .read()
                    .expect("Error reading grouphead temperature");
                let thermofilter_temp = thermofilter.as_mut().map(|thermofilter| {
                    let temp = thermofilter.read();
                    if temp.is_err() {
                        0.0
                    } else {
                        temp.unwrap()
                    }
                });

                if let Err(err) =
                    poller_tx.send(StateEvent::TemperatureChange(TemperatureMeasurement {
                        boiler_temp,
                        grouphead_temp,
                        thermofilter_temp,
                        timestamp: SystemTime::now(),
                    }))
                {
                    error!("Error sending temperature update: {}", err);
                };

                interval.tick().await;
            }
        });

        let flush_tx = self.event_tx.clone();

        task::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));

            loop {
                interval.tick().await;

                if let Err(err) =
                    flush_tx.send(StateEvent::FlushTemperatureMeasurementBufferRequest)
                {
                    error!("Error sending flush event: {err}");
                }
            }
        });

        self.poller = Some(poller);

        Ok(())
    }

    pub async fn update_mode(&mut self, mode: Mode) -> Result<()> {
        if mode != self.mode {
            self.mode = mode;

            if let Some(poller) = self.poller.take() {
                poller.abort();
                let _ = poller.await;
                self.poll()?;
            }
        }

        Ok(())
    }

    fn get_interval(&self) -> Duration {
        Duration::from_millis(match self.mode {
            Mode::Idle => 1_000,
            Mode::Active => 100,
            Mode::Brew => 100,
            Mode::Steam => 100,
            Mode::Offline => 0,
        })
    }
}

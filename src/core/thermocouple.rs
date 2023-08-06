use std::{
    time::{Duration, SystemTime},
    vec,
};

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
            loop {
                let mut smoothing_interval = tokio::time::interval(interval / 10);

                let mut boiler_temp_samples: Vec<f32> = vec![];
                let mut grouphead_temp_samples: Vec<f32> = vec![];
                let mut thermofilter_temp_samples: Vec<f32> = vec![];

                // Sample 5 times for the given `interval` period and only emit the median reading.
                // This smooths out fluctuations in readings.
                for _i in 0..10 {
                    if let Ok(boiler_temp) = boiler.read() {
                        boiler_temp_samples.push(boiler_temp);
                    }

                    if let Ok(grouphead_temp) = grouphead.read() {
                        grouphead_temp_samples.push(grouphead_temp);
                    }

                    let thermofilter_temp = thermofilter.as_mut().map(|thermofilter| {
                        let temp = thermofilter.read();
                        if temp.is_err() {
                            0.0
                        } else {
                            temp.unwrap()
                        }
                    });

                    if let Some(thermofilter_temp) = thermofilter_temp {
                        thermofilter_temp_samples.push(thermofilter_temp);
                    }

                    smoothing_interval.tick().await;
                }

                if boiler_temp_samples.len() < 2 || grouphead_temp_samples.len() < 2 {
                    if boiler_temp_samples.len() < 2 {
                        let _ = poller_tx.send(StateEvent::TemperatureReadError(String::from(
                            format!(
                                "Error reading the boiler temperature. Got {} / 10 samples",
                                boiler_temp_samples.len()
                            ),
                        )));
                    }

                    if grouphead_temp_samples.len() < 2 {
                        let _ = poller_tx.send(StateEvent::TemperatureReadError(String::from(
                            format!(
                                "Error reading the boiler temperature. Got {} / 10 samples",
                                grouphead_temp_samples.len()
                            ),
                        )));
                    }

                    continue;
                }

                boiler_temp_samples.sort_by(|a, b| a.partial_cmp(b).unwrap());

                if (boiler_temp_samples.first().unwrap() - boiler_temp_samples.last().unwrap())
                    .abs()
                    > 5.0
                {
                    info!(
                        "Temperature swing detected in boiler samples: {:#?}",
                        boiler_temp_samples
                    );
                }

                let boiler_temp = boiler_temp_samples
                    .get(boiler_temp_samples.len() / 2)
                    .expect("Failed to get the median boiler temp");

                grouphead_temp_samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
                let grouphead_temp = grouphead_temp_samples
                    .get(grouphead_temp_samples.len() / 2)
                    .expect("Failed to get the median grouphead temp");

                let mut thermofilter_temp: Option<f32> = None;

                if thermofilter_temp_samples.len() != 0 {
                    thermofilter_temp_samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
                    let filtered_thermofilter_temp_samples = thermofilter_temp_samples
                        .into_iter()
                        .filter(|v| v > &0.0)
                        .collect::<Vec<_>>();

                    if filtered_thermofilter_temp_samples.len() > 2 {
                        thermofilter_temp = filtered_thermofilter_temp_samples
                            .get(filtered_thermofilter_temp_samples.len() / 2)
                            .copied();
                    }
                }

                if let Err(err) =
                    poller_tx.send(StateEvent::TemperatureChange(TemperatureMeasurement {
                        boiler_temp: *boiler_temp,
                        grouphead_temp: *grouphead_temp,
                        thermofilter_temp,
                        timestamp: SystemTime::now(),
                    }))
                {
                    error!("Error sending temperature update: {}", err);
                };
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

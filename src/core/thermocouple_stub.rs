use std::{
    time::{Duration, SystemTime},
    vec,
};

use anyhow::{anyhow, Result};
use log::{error, info};

use tokio::{
    sync::broadcast::Sender,
    task::{self, JoinHandle}, select, time,
};

use super::{
    config::{Config, Spi},
    state::{Event as StateEvent, Mode, TemperatureMeasurement},
};

pub struct Thermocouple {
    name: String,
    spi: rppal::spi::Spi,
    pin: rppal::gpio::OutputPin,
}

impl Thermocouple {
    pub fn read(&mut self) -> Result<f32> {
        Ok(0.0)
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
        Ok(())
    }
}

use std::{
    collections::VecDeque,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use super::{
    config::ControlMethod,
    db::{write_measurements, DBHandle, Measurement},
    mqtt::MqttOutgoingMessage,
};

pub struct State {
    pub mode: Mode,
    pub power_state: PowerState,
    pub control_method: ControlMethod,
    pub boiler_state: PowerState,
    pub temp: Option<TemperatureMeasurement>,
    pub target_temp: f32,
    measurement_write_queue: VecDeque<Measurement>,
    pool: DBHandle,
}

impl State {
    pub fn new(pool: DBHandle) -> State {
        State {
            mode: Mode::Idle,
            control_method: ControlMethod::Threshold,
            power_state: PowerState::Off,
            boiler_state: PowerState::Off,
            temp: None,
            target_temp: 95.0,
            measurement_write_queue: VecDeque::new(),
            pool,
        }
    }

    pub fn flush_measurements(&mut self) -> Result<()> {
        let measurements: Vec<Measurement> = self
            .measurement_write_queue
            .drain(..)
            .collect::<VecDeque<_>>()
            .into();
        write_measurements(measurements, &self.pool)?;

        Ok(())
    }

    pub fn update(&mut self, event: Event) -> Result<Vec<MqttOutgoingMessage>> {
        let mut mqtt_messages: Vec<MqttOutgoingMessage> = vec![];

        match event {
            Event::PowerStateUpdate(state) => {
                match state {
                    PowerState::Off => {
                        self.power_state = PowerState::Off;
                        self.mode = Mode::Idle;
                    }
                    PowerState::On => {
                        self.power_state = PowerState::On;
                        self.mode = Mode::Heat;
                    }
                }

                mqtt_messages.push(MqttOutgoingMessage::StatusUpdate(self.mode.clone()));
            }
            Event::TempUpdate(temp) => {
                self.temp = Some(temp.clone());

                mqtt_messages.push(MqttOutgoingMessage::TemperatureUpdate(temp.clone()));

                self.measurement_write_queue.push_back(Measurement {
                    time: temp.timestamp.duration_since(UNIX_EPOCH)?.as_millis() as i64,
                    target_temp_c: self.target_temp,
                    boiler_temp_c: temp.boiler_temp,
                    grouphead_temp_c: temp.grouphead_temp,
                    thermofilter_temp_c: temp.thermofilter_temp.unwrap_or(-1000.0),
                    power: self.power_state == PowerState::On,
                    heat: self.boiler_state == PowerState::On,
                    pull: false,
                    steam: false,
                });
            }
            Event::BoilerStateUpdate(state) => {
                if self.power_state == PowerState::Off && state == PowerState::On {
                    Err(anyhow!(
                        "Cannot set boiler state to On when the machine is powered off!"
                    ))?;
                }

                self.boiler_state = state.clone();

                mqtt_messages.push(MqttOutgoingMessage::BoilerStatusUpdate(state.clone()));
            }
            Event::TargetTempSet(temp) => {
                self.target_temp = temp;
                mqtt_messages.push(MqttOutgoingMessage::TargetTemperatureUpdate(temp));
            }
            Event::ControlMethodSet(control_method) => {
                self.control_method = control_method.clone();
                mqtt_messages.push(MqttOutgoingMessage::ControlMethodUpdate(control_method));
            }
            Event::BoilerStateSet(power_state) => {
                self.boiler_state = power_state.clone();
                mqtt_messages.push(MqttOutgoingMessage::BoilerStatusUpdate(power_state));
            }
            Event::FlushDb => {
                self.flush_measurements()?;
            }
        }

        Ok(mqtt_messages)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum Mode {
    Idle,
    Heat,
    Brew,
    Steam,
    Offline,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TemperatureMeasurement {
    pub boiler_temp: f32,
    pub grouphead_temp: f32,
    pub thermofilter_temp: Option<f32>,
    pub timestamp: SystemTime,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum PowerState {
    On,
    Off,
}

impl Into<bool> for PowerState {
    fn into(self) -> bool {
        self == PowerState::On
    }
}

#[derive(Clone, Debug)]
pub enum Event {
    TempUpdate(TemperatureMeasurement),
    PowerStateUpdate(PowerState),
    BoilerStateUpdate(PowerState),
    FlushDb,

    TargetTempSet(f32),
    ControlMethodSet(ControlMethod),
    BoilerStateSet(PowerState),
}

use std::time::SystemTime;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use super::{config::ControlMethod, mqtt::MqttOutgoingMessage};

pub struct State {
    mode: Mode,
    power_state: PowerState,
    control_method: ControlMethod,
    boiler_state: PowerState,
    temp: Option<TemperatureMeasurement>,
    temp_history: Vec<TemperatureMeasurement>,
    extraction_temp_prediction: Option<f32>,
    extraction_temp_target: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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

impl Default for State {
    fn default() -> Self {
        State {
            mode: Mode::Idle,
            control_method: ControlMethod::Threshold,
            power_state: PowerState::Off,
            boiler_state: PowerState::Off,
            temp: None,
            temp_history: vec![],
            extraction_temp_prediction: None,
            extraction_temp_target: 95.0,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Event {
    TempUpdate(TemperatureMeasurement),
    PowerStateUpdate(PowerState),
    BoilerStateUpdate(PowerState),

    TargetTempSet(f32),
    ControlMethodSet(ControlMethod),
    BoilerStateSet(PowerState),
}

impl State {
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
            }
            Event::BoilerStateUpdate(state) => {
                if self.power_state == PowerState::On {
                    Err(anyhow!(
                        "Cannot set boiler state to On when the machine is powered off!"
                    ))?;
                }

                self.boiler_state = state.clone();

                mqtt_messages.push(MqttOutgoingMessage::BoilerStatusUpdate(state.clone()));
            }
            Event::TargetTempSet(temp) => {
                self.extraction_temp_target = temp;
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
        }

        Ok(mqtt_messages)
    }
}

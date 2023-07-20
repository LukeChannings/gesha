use std::{
    collections::VecDeque,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{anyhow, Result};
use log::error;
use serde::{Deserialize, Serialize};
use sqlx::query_as;

use super::{
    config::ControlMethod,
    db::{write_measurements, DBHandle, Measurement},
    mqtt::MqttOutgoingMessage,
};

pub struct State {
    pub mode: Mode,
    pub power_state: PowerState,
    pub control_method: ControlMethod,
    pub boiler_state: f32,
    pub temp: Option<TemperatureMeasurement>,
    pub target_temp: f32,
    pub is_brewing: bool,
    measurement_write_queue: VecDeque<Measurement>,
    pool: DBHandle,
}

impl State {
    pub fn new(pool: DBHandle) -> State {
        State {
            mode: Mode::Idle,
            control_method: ControlMethod::Threshold,
            power_state: PowerState::Off,
            boiler_state: 0.0,
            temp: None,
            target_temp: 95.0,
            is_brewing: false,
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

    pub async fn update(&mut self, event: Event) -> Result<Vec<MqttOutgoingMessage>> {
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
                        self.mode = Mode::Active;
                    }
                }

                mqtt_messages.push(MqttOutgoingMessage::ModeUpdate(self.mode.clone()));
            }
            Event::ModeSet(mode) => match mode {
                Mode::Brew => {
                    if self.mode == Mode::Active {
                        self.mode = Mode::Brew;
                        mqtt_messages.push(MqttOutgoingMessage::ModeUpdate(self.mode.clone()));
                    } else if self.mode != Mode::Brew {
                        error!("Cannot transition from {:?} to {:?}", self.mode, mode);
                    }
                }
                Mode::Active => {
                    if self.mode == Mode::Brew {
                        self.mode = Mode::Active;
                        mqtt_messages.push(MqttOutgoingMessage::ModeUpdate(self.mode.clone()));
                    } else if self.mode == Mode::Idle {
                        mqtt_messages.push(MqttOutgoingMessage::PowerRelayStatus(PowerState::On))
                    } else {
                        error!("Cannot transition from {:?} to {:?}", self.mode, mode);
                    }
                }
                Mode::Idle => {
                    mqtt_messages.push(MqttOutgoingMessage::PowerRelayStatus(PowerState::Off))
                }
                Mode::Offline => {
                    error!("Cannot set mode to offline.");
                }
                Mode::Steam => {
                    error!("Cannot set mode to Steam (yet).");
                }
            },
            Event::TempUpdate(temp) => {
                self.temp = Some(temp.clone());

                mqtt_messages.push(MqttOutgoingMessage::TemperatureUpdate(temp.clone()));

                self.measurement_write_queue.push_front(Measurement {
                    time: temp.timestamp.duration_since(UNIX_EPOCH)?.as_millis() as i64,
                    target_temp_c: self.target_temp,
                    boiler_temp_c: temp.boiler_temp,
                    grouphead_temp_c: temp.grouphead_temp,
                    thermofilter_temp_c: temp.thermofilter_temp,
                    power: self.power_state == PowerState::On,
                    heat_level: Some(self.boiler_state),
                    pull: self.mode == Mode::Brew,
                    steam: false,
                });
            }
            Event::BoilerStateUpdate(state) => {
                if self.power_state == PowerState::Off && state > 0.0 {
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
            Event::BoilerStateSet(power_level) => {
                self.boiler_state = power_level.clone();
                mqtt_messages.push(MqttOutgoingMessage::BoilerStatusUpdate(power_level));
            }
            Event::FlushDb => {
                self.flush_measurements()?;
            }
            Event::TempHistoryRequest(request) => {
                let mut result = query_as!(
                    Measurement,
                    r#"
                    SELECT time, power, pull, steam,
                        heat_level as "heat_level: f32",
                        target_temp_c as "target_temp_c: f32",
                        boiler_temp_c as "boiler_temp_c: f32",
                        grouphead_temp_c as "grouphead_temp_c: f32",
                        thermofilter_temp_c as "thermofilter_temp_c: f32"
                    FROM measurement
                    WHERE time > ? AND TIME < ?
                    ORDER BY time DESC"#,
                    request.from,
                    request.to
                )
                .fetch_all(&self.pool)
                .await?;

                result.extend( self.measurement_write_queue.iter().clone().cloned());

                let json_result = serde_json::to_string(&result)?;

                mqtt_messages.push(MqttOutgoingMessage::TemperatureHistoryResult(json_result))
            }
        }

        Ok(mqtt_messages)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum Mode {
    Idle,
    Active,
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TempHistoryRange {
    from: i64,
    to: i64,
}

#[derive(Clone, Debug)]
pub enum Event {
    TempUpdate(TemperatureMeasurement),
    PowerStateUpdate(PowerState),
    BoilerStateUpdate(f32),
    ModeSet(Mode),
    FlushDb,
    TempHistoryRequest(TempHistoryRange),

    TargetTempSet(f32),
    ControlMethodSet(ControlMethod),
    BoilerStateSet(f32),
}

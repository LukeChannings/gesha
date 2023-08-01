use std::{
    collections::VecDeque,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{anyhow, Result};
use log::{error, info};
use serde::{Deserialize, Serialize};
use sqlx::query;

use crate::controller::ControlMethod;

use super::{
    db::{read_measurements, write_measurements, DBHandle, Measurement},
    mqtt::MqttOutgoingMessage,
};

pub struct State {
    pub mode: Mode,
    pub power_state: PowerState,
    pub control_method: ControlMethod,
    pub boiler_state: f32,
    pub temp: Option<TemperatureMeasurement>,
    pub target_temperature: f32,
    pub is_brewing: bool,
    measurement_write_queue: VecDeque<Measurement>,
    pool: DBHandle,
}

const DB_KEY_TARGET_TEMPERATURE: &str = "TargetTemperature";
const DB_KEY_CONTROL_METHOD: &str = "ControlMethod";

impl State {
    pub async fn new(pool: DBHandle) -> Result<State> {
        let mut state = State {
            mode: Mode::Idle,
            control_method: ControlMethod::None,
            power_state: PowerState::Off,
            boiler_state: 0.0,
            temp: None,
            target_temperature: 0.0,
            is_brewing: false,
            measurement_write_queue: VecDeque::new(),
            pool,
        };

        State::load_db_config(&mut state).await?;

        Ok(state)
    }

    async fn load_db_config(&mut self) -> Result<()> {
        let result = query!("SELECT key, value FROM config")
            .fetch_all(&self.pool)
            .await?;

        for record in result {
            info!("Reading '{}={}'", record.key, record.value);

            match record.key.as_str() {
                DB_KEY_TARGET_TEMPERATURE => {
                    self.target_temperature = serde_plain::from_str(&record.value)?
                }
                DB_KEY_CONTROL_METHOD => {
                    self.control_method = serde_plain::from_str(&record.value)?;
                }
                _ => {
                    info!("Unknown config key: {:?}", record.key);
                }
            }
        }

        Ok(())
    }

    async fn write_config(&mut self, key: &str, value: &str) -> Result<()> {
        query!(
            r#"
        INSERT INTO config VALUES (?1, ?2)
        ON CONFLICT(key) DO
        UPDATE SET value = ?2 WHERE key = ?1;
        "#,
            key,
            value
        )
        .execute(&self.pool)
        .await?;

        info!("Wrote {key}={value} to the DB");

        Ok(())
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

    async fn set_target_temperature(&mut self, target_temperature: f32) -> Result<()> {
        self.target_temperature = target_temperature;
        let value: String = serde_plain::to_string::<f32>(&target_temperature)?;

        self.write_config(DB_KEY_TARGET_TEMPERATURE, &value).await?;

        Ok(())
    }

    async fn set_control_method(&mut self, control_method: &ControlMethod) -> Result<()> {
        self.control_method = control_method.clone();
        let value: String = serde_plain::to_string::<ControlMethod>(control_method)?;

        self.write_config(DB_KEY_CONTROL_METHOD, &value).await?;

        Ok(())
    }

    pub async fn handle_event(&mut self, event: Event) -> Result<Vec<MqttOutgoingMessage>> {
        let mut mqtt_messages: Vec<MqttOutgoingMessage> = vec![];

        match event {
            Event::PowerStateChange(state) => {
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
            Event::ModeChange(mode) => match mode {
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
            Event::TemperatureChange(temp) => {
                self.temp = Some(temp.clone());

                mqtt_messages.push(MqttOutgoingMessage::TemperatureUpdate(temp.clone()));

                self.measurement_write_queue.push_front(Measurement {
                    time: temp.timestamp.duration_since(UNIX_EPOCH)?.as_millis() as i64,
                    target_temp_c: self.target_temperature,
                    boiler_temp_c: temp.boiler_temp,
                    grouphead_temp_c: temp.grouphead_temp,
                    thermofilter_temp_c: temp.thermofilter_temp,
                    power: self.power_state == PowerState::On,
                    heat_level: Some(self.boiler_state),
                    pull: self.mode == Mode::Brew,
                    steam: false,
                });
            }
            Event::BoilerHeatLevelChange(state) => {
                if self.power_state == PowerState::Off && state > 0.0 {
                    Err(anyhow!(
                        "Cannot set boiler state to On when the machine is powered off!"
                    ))?;
                }

                self.boiler_state = state.clone();

                mqtt_messages.push(MqttOutgoingMessage::BoilerStatusUpdate(state.clone()));
            }
            Event::TargetTemperatureChangeRequest(target_temperature) => {
                self.set_target_temperature(target_temperature).await?;
                mqtt_messages.push(MqttOutgoingMessage::TargetTemperatureUpdate(
                    target_temperature,
                ));
            }
            Event::ControlMethodChangeRequest(control_method) => {
                self.set_control_method(&control_method).await?;
                mqtt_messages.push(MqttOutgoingMessage::ControlMethodUpdate(control_method));
            }
            Event::FlushTemperatureMeasurementBufferRequest => {
                self.flush_measurements()?;
            }
            Event::TemperatureHistoryRequest { from, to, limit } => {
                let result = read_measurements(&self.pool, from, to, limit).await?;

                let json_result = serde_json::to_string(&result)?;

                mqtt_messages.push(MqttOutgoingMessage::TemperatureHistoryResponse(json_result))
            }
            _ => {}
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
    TemperatureChange(TemperatureMeasurement),
    PowerStateChange(PowerState),
    BoilerHeatLevelChange(f32),
    ModeChange(Mode),
    FlushTemperatureMeasurementBufferRequest,
    TemperatureHistoryRequest { from: i64, to: i64, limit: i64 },

    TargetTemperatureChangeRequest(f32),
    ManualBoilerHeatLevelRequest(f32),
    ControlMethodChangeRequest(ControlMethod),
}

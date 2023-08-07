use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, Result};
use log::{error, info};
use serde::{Deserialize, Serialize};

use crate::controller::ControlMethod;

use super::{
    db::{ConfigItem, Db, Measurement},
    mqtt::MqttOutgoingMessage,
};

pub struct State {
    pub mode: Mode,
    pub power_state: PowerState,
    pub control_method: ControlMethod,
    pub boiler_state: f32,
    pub temp: Option<TemperatureMeasurement>,
    pub target_temperature: f32,
    pub shot_state: Shot,
    db: Db,
}

pub enum Shot {
    NotPulling,
    PullStarted(i64),
}

const DB_KEY_TARGET_TEMPERATURE: &str = "TargetTemperature";
const DB_KEY_CONTROL_METHOD: &str = "ControlMethod";

impl State {
    pub async fn new() -> Result<State> {
        let db = Db::new("/opt/gesha/var/db/gesha.db").await?;

        let mut state = State {
            mode: Mode::Idle,
            control_method: ControlMethod::None,
            power_state: PowerState::Off,
            boiler_state: 0.0,
            temp: None,
            target_temperature: 0.0,
            shot_state: Shot::NotPulling,
            db,
        };

        State::load_db_config(&mut state).await?;

        Ok(state)
    }

    async fn load_db_config(&mut self) -> Result<()> {
        let configs = self.db.read_config().await?;

        for config in configs {
            info!("Reading '{}={}'", config.key, config.value);

            match config.key.as_str() {
                DB_KEY_TARGET_TEMPERATURE => {
                    self.target_temperature = serde_plain::from_str(&config.value)?
                }
                DB_KEY_CONTROL_METHOD => {
                    self.control_method = serde_plain::from_str(&config.value)?;
                }
                _ => {
                    // Skip unknown keys
                }
            }
        }

        Ok(())
    }

    async fn set_target_temperature(&mut self, target_temperature: f32) -> Result<ConfigItem> {
        self.target_temperature = target_temperature;
        let value: String = serde_plain::to_string::<f32>(&target_temperature)?;

        let config_item = ConfigItem {
            key: DB_KEY_TARGET_TEMPERATURE.to_string(),
            value,
        };

        self.db.write_config(&config_item).await?;

        Ok(config_item)
    }

    async fn set_control_method(&mut self, control_method: &ControlMethod) -> Result<ConfigItem> {
        self.control_method = control_method.clone();
        let config_item = ConfigItem {
            key: DB_KEY_CONTROL_METHOD.to_string(),
            value: serde_plain::to_string::<ControlMethod>(control_method)?,
        };

        self.db.write_config(&config_item).await?;

        Ok(config_item)
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
                        self.shot_state = Shot::PullStarted(
                            SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64,
                        );
                        mqtt_messages.push(MqttOutgoingMessage::ModeUpdate(self.mode.clone()));
                    } else if self.mode != Mode::Brew {
                        error!(
                            "Cannot transition from {:?} to {:?}. Only Active -> Brew is allowed.",
                            self.mode, mode
                        );
                    }
                }
                Mode::Active => {
                    if self.mode == Mode::Brew {
                        if let Shot::PullStarted(start_time) = self.shot_state {
                            self.shot_state = Shot::NotPulling;
                            let end_time =
                                SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64;

                            match self.db.write_shot(start_time, end_time).await {
                                Ok(_) => info!("Recorded shot {}-{}", start_time, end_time),
                                Err(msg) => error!(
                                    "Failed to store shot for {}-{}: {}",
                                    start_time, end_time, msg
                                ),
                            }
                        }

                        mqtt_messages.push(MqttOutgoingMessage::ModeUpdate(self.mode.clone()));
                    }

                    if self.mode == Mode::Idle {
                        mqtt_messages.push(MqttOutgoingMessage::PowerRelayStatus(PowerState::On))
                    }

                    self.mode = Mode::Active;
                    mqtt_messages.push(MqttOutgoingMessage::ModeUpdate(self.mode.clone()));
                }
                Mode::Idle => {
                    self.mode = Mode::Idle;
                    mqtt_messages.push(MqttOutgoingMessage::ModeUpdate(self.mode.clone()));
                    mqtt_messages.push(MqttOutgoingMessage::PowerRelayStatus(PowerState::Off))
                }
                Mode::Steam => {
                    self.mode = Mode::Steam;

                    if self.mode == Mode::Idle {
                        mqtt_messages.push(MqttOutgoingMessage::PowerRelayStatus(PowerState::On))
                    }

                    mqtt_messages.push(MqttOutgoingMessage::ModeUpdate(self.mode.clone()));
                }
                Mode::Offline => {
                    error!("Cannot set mode to offline.");
                }
            },
            Event::TemperatureChange(temp) => {
                self.temp = Some(temp.clone());

                mqtt_messages.push(MqttOutgoingMessage::TemperatureUpdate(temp.clone()));

                self.db.write_measurement_queue(Measurement {
                    time: temp.timestamp.duration_since(UNIX_EPOCH)?.as_millis() as i64,
                    target_temp_c: self.target_temperature,
                    boiler_temp_c: temp.boiler_temp,
                    grouphead_temp_c: temp.grouphead_temp,
                    thermofilter_temp_c: temp.thermofilter_temp,
                    power: self.power_state == PowerState::On,
                    heat_level: Some(self.boiler_state),
                    pull: self.mode == Mode::Brew,
                    steam: false,
                })?;
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
                let config_item = self.set_target_temperature(target_temperature).await?;
                mqtt_messages.push(MqttOutgoingMessage::TargetTemperatureUpdate(
                    target_temperature,
                ));
                mqtt_messages.push(MqttOutgoingMessage::ConfigUpdate(vec![config_item]))
            }
            Event::ControlMethodChangeRequest(control_method) => {
                let config_item = self.set_control_method(&control_method).await?;
                mqtt_messages.push(MqttOutgoingMessage::ControlMethodUpdate(control_method));
                mqtt_messages.push(MqttOutgoingMessage::ConfigUpdate(vec![config_item]))
            }
            Event::FlushTemperatureMeasurementBufferRequest => {
                self.db.flush_measurements()?;
            }
            Event::TemperatureHistoryRequest {
                from,
                to,
                limit,
                bucket_size,
            } => {
                let result = self
                    .db
                    .read_measurements(from, to, limit, bucket_size)
                    .await?;

                let json_result = serde_json::to_string(&result)?;

                mqtt_messages.push(MqttOutgoingMessage::TemperatureHistoryResponse(json_result))
            }
            Event::TemperatureReadError(message) => {
                error!("Temperature read error: {message}")
            }
            Event::ReadConfig => {
                let config = self.db.read_config().await?;
                mqtt_messages.push(MqttOutgoingMessage::ConfigUpdate(config));
            }
            Event::WriteConfigItem(config_item) => {
                self.db.write_config(&config_item).await?;

                mqtt_messages.push(MqttOutgoingMessage::ConfigUpdate(vec![config_item]));
            }
            Event::ShotHistoryRequest { from, to, limit } => {
                let shots = self.db.read_shots(from, to, limit).await?;
                let json_result = serde_json::to_string(&shots)?;

                mqtt_messages.push(MqttOutgoingMessage::ShotHistoryResponse(json_result));
            }
            _ => {}
        }

        Ok(mqtt_messages)
    }

    pub fn close(&mut self) -> Result<()> {
        self.db.flush_measurements()?;

        Ok(())
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
    TemperatureReadError(String),
    PowerStateChange(PowerState),
    BoilerHeatLevelChange(f32),
    ModeChange(Mode),
    FlushTemperatureMeasurementBufferRequest,
    TemperatureHistoryRequest {
        from: i64,
        to: i64,
        limit: Option<i64>,
        bucket_size: Option<i64>,
    },
    ShotHistoryRequest {
        from: i64,
        to: i64,
        limit: Option<i64>,
    },
    ReadConfig,

    TargetTemperatureChangeRequest(f32),
    ManualBoilerHeatLevelRequest(f32),
    ControlMethodChangeRequest(ControlMethod),
    WriteConfigItem(ConfigItem),
}

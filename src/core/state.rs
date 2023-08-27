use std::time::{SystemTime, Duration};

use anyhow::{anyhow, Result};
use log::{error, info};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast::Sender;

use crate::{
    controller::ControlMethod,
    core::db::{DB_KEY_CONTROL_METHOD, DB_KEY_TARGET_TEMPERATURE},
};

use super::{
    db::{ConfigItem, Db, Measurement},
    mqtt::{MqttIncomingMessage, MqttOutgoingMessage, ValueChange},
    util,
};

pub struct State {
    pub mode: Mode,
    pub power_relay_available: bool,
    pub power_state: IsPowerOn,
    pub control_method: ControlMethod,
    pub boiler_state: f32,
    pub current_temperature: Option<TemperatureMeasurement>,
    pub target_temperature: f32,
    pub target_temperature_steam: f32,
    pub shot_state: Shot,
    db: Db,
}

pub enum Shot {
    NotPulling,
    PullStarted(i64),
}

impl State {
    pub async fn new(event_tx: Sender<Event>) -> Result<State> {
        let mut db = Db::new("/opt/gesha/var/db/gesha.db").await?;

        db.start_measurement_writer_interval(Duration::from_secs(60));

        let configs = (&db).read_config().await?;

        let target_temperature: f32 = configs
            .get(DB_KEY_TARGET_TEMPERATURE)
            .map(|s| serde_plain::from_str(s).unwrap())
            .unwrap_or(95.0);
        let control_method: ControlMethod = configs
            .get(DB_KEY_CONTROL_METHOD)
            .map(|s| serde_plain::from_str(s).unwrap())
            .unwrap_or(ControlMethod::None);

        let state = State {
            mode: Mode::Idle,
            control_method,
            power_relay_available: true,
            power_state: false,
            boiler_state: 0.0,
            current_temperature: None,
            target_temperature,
            target_temperature_steam: 130.0,
            shot_state: Shot::NotPulling,
            db,
        };

        for event in vec![
            Event::OutgoingMqttMessage(MqttOutgoingMessage::ModeUpdate(state.mode.clone())),
            Event::OutgoingMqttMessage(MqttOutgoingMessage::ControlMethodUpdate(
                state.control_method.clone(),
            )),
            Event::OutgoingMqttMessage(MqttOutgoingMessage::TargetTemperatureUpdate(
                state.target_temperature,
            )),
        ] {
            event_tx.send(event)?;
        }

        Ok(state)
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

    pub async fn handle_event(&mut self, event: &Event) -> Result<Vec<Event>> {
        return match event {
            Event::IncomingMqttMessage(message) => match message {
                MqttIncomingMessage::ExternRelayAvailabilityChanged(relay_is_available) => {
                    self.power_relay_available = *relay_is_available;

                    if !relay_is_available && self.mode != Mode::Idle {
                        let mut events: Vec<Event> = vec![Event::ModeChanged(Mode::Idle)];

                        if self.mode == Mode::Brew {
                            self.toggle_brew_mode().await?;
                        }

                        if self.mode == Mode::Steam {
                            self.add_steam_mode_events(false, &mut events);
                        }

                        self.add_power_mode_events(false, &mut events);

                        self.power_state = false;
                        self.mode = Mode::Idle;

                        return Ok(events);
                    }

                    Ok(vec![])
                }
                MqttIncomingMessage::ExternRelayPowerStateChanged(new_power_state) => {
                    if self.power_state == *new_power_state {
                        return Ok(vec![]);
                    }

                    let mut events: Vec<Event> = vec![];

                    if *new_power_state == true {
                        if self.mode == Mode::Idle {
                            self.mode = Mode::Active;
                            events.push(Event::ModeChanged(Mode::Active));
                        }

                        self.power_state = true;
                        events.push(Event::PowerStateChanged(true));
                    } else {
                        self.power_state = false;
                        events.push(Event::PowerStateChanged(false));

                        if self.mode != Mode::Idle {
                            self.mode = Mode::Idle;
                            events.push(Event::ModeChanged(Mode::Idle));
                        }
                    }

                    Ok(events)
                }
                MqttIncomingMessage::ModeSet(new_mode) => {
                    let current_mode = self.mode.clone();

                    if current_mode != *new_mode {
                        self.mode = new_mode.clone();
                        let mut events: Vec<Event> = vec![Event::ModeChanged(new_mode.clone())];

                        match (current_mode, new_mode) {
                            // Handle transitions to Active mode
                            (Mode::Idle, Mode::Active) => {
                                self.add_power_mode_events(true, &mut events);
                            }
                            (Mode::Brew, Mode::Active) => {
                                self.toggle_brew_mode().await?;
                            }
                            (Mode::Steam, Mode::Active) => {
                                self.add_steam_mode_events(false, &mut events);
                            }

                            // Handle transitions to Idle mode
                            (Mode::Active, Mode::Idle) => {
                                self.add_power_mode_events(false, &mut events);
                            }
                            (Mode::Brew, Mode::Idle) => {
                                self.add_power_mode_events(false, &mut events);
                                self.toggle_brew_mode().await?;
                            }
                            (Mode::Steam, Mode::Idle) => {
                                self.add_steam_mode_events(false, &mut events);
                                self.add_power_mode_events(false, &mut events);
                            }

                            // Handle transitions to Brew mode
                            (Mode::Active, Mode::Brew) => {
                                self.toggle_brew_mode().await?;
                            }

                            // Handle transitions to Steam mode
                            (Mode::Idle, Mode::Steam) => {
                                self.add_power_mode_events(true, &mut events);
                                self.add_steam_mode_events(true, &mut events);
                            }
                            (Mode::Active, Mode::Steam) => {
                                self.add_steam_mode_events(true, &mut events);
                            }
                            (Mode::Brew, Mode::Steam) => {
                                self.toggle_brew_mode().await?;
                                self.add_steam_mode_events(true, &mut events);
                            }
                            _ => {
                                return Err(anyhow!(
                                    "Moving from mode {:?} to {:?} is not supported",
                                    self.mode,
                                    new_mode
                                ))
                            }
                        }

                        Ok(events)
                    } else {
                        Ok(vec![])
                    }
                }
                MqttIncomingMessage::ControlMethodSet(control_method) => {
                    if self.mode == Mode::Steam {
                        return Err(anyhow!("Cannot set control method when mode is Steam"));
                    }

                    let config_item = self.set_control_method(control_method).await?;
                    self.control_method = control_method.clone();

                    Ok(vec![
                        Event::ControlMethodChanged(control_method.clone()),
                        Event::OutgoingMqttMessage(MqttOutgoingMessage::ConfigUpdate(config_item)),
                    ])
                }
                MqttIncomingMessage::TemperatureTargetSet(new_target_temp) => {
                    let config_item = self.set_target_temperature(*new_target_temp).await?;
                    Ok(vec![
                        Event::OutgoingMqttMessage(MqttOutgoingMessage::TargetTemperatureUpdate(
                            *new_target_temp,
                        )),
                        Event::OutgoingMqttMessage(MqttOutgoingMessage::ConfigUpdate(config_item)),
                    ])
                }
                MqttIncomingMessage::TemperatureHistoryRequest(range) => {
                    let result = self.db.read_measurements(range).await?;

                    let json_result = serde_json::to_string(&result)?;

                    Ok(vec![Event::OutgoingMqttMessage(
                        MqttOutgoingMessage::TemperatureHistoryResponse(
                            range.id.clone(),
                            json_result,
                        ),
                    )])
                }
                MqttIncomingMessage::BoilerLevelSet(heat_level) => {
                    if self.control_method == ControlMethod::None {
                        Ok(vec![Event::ManualBoilerHeatLevelRequest(*heat_level)])
                    } else {
                        Err(anyhow!(
                            "Cannot set boiler level when control method is not None"
                        ))
                    }
                }
                MqttIncomingMessage::ShotHistoryRequest(range) => {
                    let shots = self.db.read_shots(&range).await?;
                    let json_result = serde_json::to_string(&shots)?;

                    Ok(vec![Event::OutgoingMqttMessage(
                        MqttOutgoingMessage::ShotHistoryResponse(range.id.clone(), json_result),
                    )])
                }
                MqttIncomingMessage::ConfigSet(config_item) => {
                    self.db.write_config(&config_item).await?;

                    if config_item.key.starts_with("ui_") {
                        Ok(vec![Event::OutgoingMqttMessage(
                            MqttOutgoingMessage::ConfigUpdate(config_item.clone()),
                        )])
                    } else {
                        Ok(vec![])
                    }
                }
            },
            Event::TemperatureChanged(temp) => {
                let timestamp = util::get_unix_timestamp(temp.timestamp)?;
                let mut change_events: Vec<Event> = vec![];
                let prev_temp = self.current_temperature.as_ref();

                for (current_temp, prev_temp, instrument) in vec![
                    (
                        Some(temp.boiler_temp),
                        prev_temp.map(|t| t.boiler_temp),
                        "boiler",
                    ),
                    (
                        Some(temp.grouphead_temp),
                        prev_temp.map(|t| t.grouphead_temp),
                        "grouphead",
                    ),
                    (
                        temp.thermofilter_temp,
                        prev_temp.and_then(|t| t.thermofilter_temp),
                        "thermofilter",
                    ),
                ] {
                    if current_temp.is_some()
                        && current_temp.unwrap() != prev_temp.unwrap_or(-1000.0)
                    {
                        change_events.push(Event::OutgoingMqttMessage(
                            MqttOutgoingMessage::TemperatureUpdate(
                                instrument.to_string(),
                                ValueChange {
                                    value: current_temp.unwrap(),
                                    timestamp,
                                },
                            ),
                        ))
                    }
                }

                if change_events.len() > 0 {
                    self.db.write_measurement_queue(Measurement {
                        time: timestamp,
                        target_temp_c: if self.mode == Mode::Steam {
                            self.target_temperature_steam
                        } else {
                            self.target_temperature
                        },
                        boiler_temp_c: temp.boiler_temp,
                        grouphead_temp_c: temp.grouphead_temp,
                        thermofilter_temp_c: temp.thermofilter_temp,
                        power: self.power_state,
                        heat_level: Some(self.boiler_state),
                        pull: self.mode == Mode::Brew,
                        steam: self.mode == Mode::Steam,
                    }).await?;
                }

                self.current_temperature = Some(temp.clone());

                Ok(change_events)
            }
            Event::TemperatureReadError(message) => {
                error!("Temperature read error: {message}");
                Ok(vec![])
            }
            Event::BoilerHeatLevelChanged(heat_level) => {
                self.boiler_state = *heat_level;

                Ok(vec![])
            }
            _ => {
                // All events except outgoing MQTT messages are handled by the state,
                // but some events are used to notify the rest fo the system of state changes,
                // e.g. ControlMethodChanged, TargetTemperatureChanged, etc.
                // Only the state module can change the control method or target temperature,
                // triggered by incoming MQTT messages.
                Ok(vec![])
            }
        };
    }

    fn add_steam_mode_events(&self, steam_mode_enabled: bool, events: &mut Vec<Event>) {
        if steam_mode_enabled {
            // Manually override the control method and target temperature when moving to steam mode
            events.extend(vec![
                Event::ControlMethodChanged(ControlMethod::Threshold),
                Event::TargetTemperatureChanged(self.target_temperature_steam),
            ]);
        } else {
            // Reset the control method and target temperature
            events.extend(vec![
                Event::ControlMethodChanged(self.control_method.clone()),
                Event::TargetTemperatureChanged(self.target_temperature),
            ]);
        }
    }

    // This adds the event that triggers the external Shelly relay -
    // if the relay switches state and successfully responds then
    // we'll get an IncomingMqttMessage::ExternRelayPowerStateChanged as a callback.
    fn add_power_mode_events(&mut self, power_state: IsPowerOn, events: &mut Vec<Event>) {
        events.push(Event::OutgoingMqttMessage(
            MqttOutgoingMessage::ExternRelayPowerStateSetCmd(power_state),
        ))
    }

    async fn toggle_brew_mode(&mut self) -> Result<()> {
        match self.shot_state {
            Shot::NotPulling => {
                // There is no behavioural change when moving from active to brew,
                // we just need to keep track of when the pull started.
                self.shot_state = Shot::PullStarted(util::get_unix_timestamp(SystemTime::now())?);
            }
            Shot::PullStarted(start_time) => {
                self.shot_state = Shot::NotPulling;

                let end_time = util::get_unix_timestamp(SystemTime::now())?;

                match self.db.write_shot(start_time, end_time).await {
                    Ok(_) => {
                        info!("Shot written to DB");
                    }
                    Err(err) => {
                        error!("Error writing shot to DB: {}", err);
                    }
                }
            }
        }

        Ok(())
    }

    pub async fn stop(&mut self) -> Result<()> {
        self.db.stop_measurement_writer_interval().await?;
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

pub type IsPowerOn = bool;

#[derive(Clone, Debug)]
pub enum Event {
    TemperatureChanged(TemperatureMeasurement),
    TemperatureReadError(String),
    ModeChanged(Mode),
    PowerStateChanged(IsPowerOn),
    ControlMethodChanged(ControlMethod),
    ManualBoilerHeatLevelRequest(f32),
    TargetTemperatureChanged(f32),

    BoilerHeatLevelChanged(f32),

    IncomingMqttMessage(MqttIncomingMessage),
    OutgoingMqttMessage(MqttOutgoingMessage),
}

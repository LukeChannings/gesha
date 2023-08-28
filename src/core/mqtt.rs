use anyhow::{anyhow, Result};
use log::{debug, error, info};
use rumqttc::v5::{
    mqttbytes::{
        v5::{Packet, Publish},
        QoS,
    },
    AsyncClient, Event as MqttEvent, MqttOptions,
};
use serde::{Deserialize, Serialize};
use std::{str, time::SystemTime};
use tokio::{select, sync::broadcast::Sender, task, time};
use tokio_util::sync::CancellationToken;

use crate::{
    controller::ControlMethod,
    core::{
        state::{Event, IsPowerOn},
        util,
    },
};

use super::{db::ConfigItem, state::Mode};

const TOPIC_EXTERN_POWER_STATUS: &str = "ms-silvia-switch/status";
const TOPIC_EXTERN_POWER_STATE_CHANGE: &str = "ms-silvia-switch/switch/power/state";
const TOPIC_EXTERN_POWER_COMMAND: &str = "ms-silvia-switch/switch/power/command";

const TOPIC_CONTROL_METHOD_CHANGE_REQUEST: &str = "gesha/control_method/set";
const TOPIC_TARGET_TEMPERATURE_CHANGE_REQUEST: &str = "gesha/temperature/target/set";
const TOPIC_MODE_CHANGE: &str = "gesha/mode/set";
const TOPIC_TEMPERATURE_HISTORY_REQUEST: &str = "gesha/temperature/history/command";
const TOPIC_MANUAL_BOILER_HEAT_LEVEL_REQUEST: &str = "gesha/boiler_level/set";
const TOPIC_SHOT_HISTORY_REQUEST: &str = "gesha/shot/history/command";
const TOPIC_CONFIG_SET: &str = "gesha/config/set";

pub struct Mqtt {
    uri: String,
    event_tx: Sender<Event>,
    cancel_token: CancellationToken,
    client: Option<AsyncClient>,
}

impl Mqtt {
    pub fn new(uri: &str, event_tx: Sender<Event>) -> Result<Self> {
        let cancel_token = CancellationToken::new();

        let mqtt = Mqtt {
            uri: String::from(uri),
            event_tx,
            cancel_token,
            client: None,
        };

        Ok(mqtt)
    }

    pub async fn start(&mut self) -> Result<()> {
        debug!("Waiting for notifications");

        let internal_cancel_token = self.cancel_token.clone();

        let options = MqttOptions::parse_url(self.uri.clone())?;

        let (client, mut event_loop) = AsyncClient::new(options, 10);

        self.client = Some(client);

        self.subscribe().await?;

        let mut rx = self.event_tx.subscribe();
        let tx = self.event_tx.clone();

        self.publish(&MqttOutgoingMessage::ModeUpdate(Mode::Idle))
            .await?;

        task::spawn(async move {
            loop {
                select! {
                    Ok(notification) = event_loop.poll() => {
                        if let MqttEvent::Incoming(Packet::Publish(publish_event)) = notification {
                            debug!("Received = {:?}", publish_event);

                            match TryInto::<Event>::try_into(publish_event) {
                                Ok(event) => {
                                    debug!("Sending event: {:?}", event);
                                    if let Err(err) = tx.send(event) {
                                        error!("Failed to send event: {}", err);
                                    }
                                }
                                Err(err) => {
                                    debug!("Failed to parse incoming message: {}", err);
                                }
                            }
                        }
                    },
                    Ok(event) = rx.recv() => {
                        match event {
                            Event::ModeChanged(mode) => {
                                if let Err(err) = tx.send(Event::OutgoingMqttMessage(
                                    MqttOutgoingMessage::ModeUpdate(mode),
                                )) {
                                    error!("Failed to send event: {}", err);
                                }
                            }
                            Event::TargetTemperatureChanged(target_temp) => {
                                if let Err(err) = tx.send(Event::OutgoingMqttMessage(
                                    MqttOutgoingMessage::TargetTemperatureUpdate(target_temp),
                                )) {
                                    error!("Failed to send event: {}", err);
                                }
                            }
                            Event::BoilerHeatLevelChanged(heat_level) => {
                                if let Err(err) = tx.send(Event::OutgoingMqttMessage(
                                    MqttOutgoingMessage::BoilerStatusUpdate(ValueChange {
                                        value: heat_level,
                                        timestamp: util::get_unix_timestamp(SystemTime::now()).unwrap(),
                                    }),
                                )) {
                                    error!("Failed to send event: {}", err);
                                }
                            }
                            Event::ControlMethodChanged(control_method) => {
                                if let Err(err) = tx.send(Event::OutgoingMqttMessage(
                                    MqttOutgoingMessage::ControlMethodUpdate(control_method),
                                )) {
                                    error!("Failed to send event: {}", err);
                                }
                            }
                            _ => {}
                        }
                    },
                    _ = internal_cancel_token.cancelled() => {
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        self.client
            .as_ref()
            .unwrap()
            .publish("gesha/mode", QoS::AtLeastOnce, true, "offline")
            .await?;

        // Wait for a tick of the event loop to ensure the goodbye message is sent...
        time::sleep(time::Duration::from_millis(0)).await;

        // ...and then shut down the event loop.
        self.cancel_token.cancel();

        Ok(())
    }

    pub async fn subscribe(&self) -> Result<()> {
        if let Some(client) = &self.client {
            let topics = vec![
                TOPIC_EXTERN_POWER_STATUS,
                TOPIC_EXTERN_POWER_STATE_CHANGE,
                TOPIC_CONTROL_METHOD_CHANGE_REQUEST,
                TOPIC_TARGET_TEMPERATURE_CHANGE_REQUEST,
                TOPIC_MODE_CHANGE,
                TOPIC_TEMPERATURE_HISTORY_REQUEST,
                TOPIC_MANUAL_BOILER_HEAT_LEVEL_REQUEST,
                TOPIC_SHOT_HISTORY_REQUEST,
                TOPIC_CONFIG_SET,
            ];
            for topic in topics {
                client
                    .subscribe(topic, QoS::ExactlyOnce)
                    .await
                    .map_err(|err| anyhow!("Failed to subscribe to {}, got {}", topic, err))?;
            }
            Ok(())
        } else {
            Err(anyhow!("No MQTT client available"))
        }
    }

    pub async fn publish(&self, message: &MqttOutgoingMessage) -> Result<()> {
        if self.client.is_none() {
            return Err(anyhow!("No MQTT client available"));
        }

        let (topic, payload, retain) = match message {
            MqttOutgoingMessage::ExternRelayPowerStateSetCmd(power_status) => (
                TOPIC_EXTERN_POWER_COMMAND.to_string(),
                (if *power_status { "ON" } else { "OFF" }).to_string(),
                false,
            ),
            MqttOutgoingMessage::ModeUpdate(status) => (
                String::from("gesha/mode"),
                serde_json::to_string(status)?,
                true,
            ),
            MqttOutgoingMessage::BoilerStatusUpdate(heat_level) => (
                String::from("gesha/boiler_level"),
                serde_json::to_string(heat_level)?,
                true,
            ),
            MqttOutgoingMessage::TemperatureUpdate(instrument, measurement) => (
                format!("gesha/temperature/{instrument}"),
                serde_json::to_string(measurement)?,
                true,
            ),
            MqttOutgoingMessage::TargetTemperatureUpdate(temp) => (
                format!("gesha/temperature/target"),
                serde_json::to_string(temp)?,
                true,
            ),
            MqttOutgoingMessage::ControlMethodUpdate(control_method) => (
                format!("gesha/control_method"),
                serde_json::to_string(control_method)?,
                true,
            ),
            MqttOutgoingMessage::TemperatureHistoryResponse(id, result) => (
                format!("gesha/temperature/history/{id}"),
                result.to_string(),
                false,
            ),
            MqttOutgoingMessage::ShotHistoryResponse(id, result) => (
                format!("gesha/shot/history/{id}"),
                result.to_string(),
                false,
            ),
            MqttOutgoingMessage::ConfigUpdate(config_item) => (
                format!("gesha/config/{}", config_item.key),
                config_item.value.to_string(),
                true,
            ),
        };

        self.client
            .as_ref()
            .unwrap()
            .publish(topic, QoS::AtLeastOnce, retain, payload)
            .await
            .map_err(|err| anyhow!("Failed to publish status, got {}", err))?;

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub enum MqttIncomingMessage {
    ExternRelayAvailabilityChanged(bool),
    ExternRelayPowerStateChanged(IsPowerOn),
    ControlMethodSet(ControlMethod),
    TemperatureTargetSet(f32),
    ModeSet(Mode),
    TemperatureHistoryRequest(Range),
    BoilerLevelSet(f32),
    ShotHistoryRequest(Range),
    ConfigSet(ConfigItem),
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Range {
    pub id: String,
    pub from: i64,
    pub to: i64,
    pub limit: Option<i64>,
    pub bucket_size: Option<i64>,
}

#[derive(Clone, Debug)]
pub enum MqttOutgoingMessage {
    ExternRelayPowerStateSetCmd(IsPowerOn),
    ModeUpdate(Mode),
    BoilerStatusUpdate(ValueChange),
    TemperatureUpdate(String, ValueChange),
    TemperatureHistoryResponse(String, String),
    TargetTemperatureUpdate(f32),
    ControlMethodUpdate(ControlMethod),
    ShotHistoryResponse(String, String),
    ConfigUpdate(ConfigItem),
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ValueChange {
    pub value: f32,
    pub timestamp: i64,
}

impl TryInto<Event> for Publish {
    type Error = anyhow::Error;

    fn try_into(self) -> std::result::Result<Event, Self::Error> {
        let topic = str::from_utf8(&self.topic)?;

        match topic {
            TOPIC_CONTROL_METHOD_CHANGE_REQUEST => {
                let control_method = serde_yaml::from_slice(&self.payload)?;
                Ok(Event::IncomingMqttMessage(
                    MqttIncomingMessage::ControlMethodSet(control_method),
                ))
            }
            TOPIC_TARGET_TEMPERATURE_CHANGE_REQUEST => Ok(Event::IncomingMqttMessage(
                MqttIncomingMessage::TemperatureTargetSet(serde_yaml::from_slice(&self.payload)?),
            )),
            TOPIC_MODE_CHANGE => {
                let mode: Mode = serde_yaml::from_slice(&self.payload)?;
                Ok(Event::IncomingMqttMessage(MqttIncomingMessage::ModeSet(
                    mode,
                )))
            }
            TOPIC_TEMPERATURE_HISTORY_REQUEST => {
                let range: Range = serde_json::from_slice(&self.payload)?;

                Ok(Event::IncomingMqttMessage(
                    MqttIncomingMessage::TemperatureHistoryRequest(range),
                ))
            }
            TOPIC_EXTERN_POWER_STATUS => Ok(Event::IncomingMqttMessage(
                MqttIncomingMessage::ExternRelayAvailabilityChanged(self.payload == "online"),
            )),
            TOPIC_EXTERN_POWER_STATE_CHANGE => {
                let power_state: IsPowerOn = if self.payload == "ON" { true } else { false };

                Ok(Event::IncomingMqttMessage(
                    MqttIncomingMessage::ExternRelayPowerStateChanged(power_state),
                ))
            }
            TOPIC_MANUAL_BOILER_HEAT_LEVEL_REQUEST => {
                let heat_level: f32 = serde_yaml::from_slice(&self.payload)?;

                Ok(Event::IncomingMqttMessage(
                    MqttIncomingMessage::BoilerLevelSet(heat_level),
                ))
            }
            TOPIC_CONFIG_SET => {
                let config_item: ConfigItem = serde_json::from_slice(&self.payload)?;

                if config_item.key.starts_with("ui_") {
                    info!("Setting config {:?}", config_item);

                    Ok(Event::IncomingMqttMessage(MqttIncomingMessage::ConfigSet(
                        config_item,
                    )))
                } else {
                    Err(anyhow!(
                        "Refusing to set a config entry that isn't prefixed with 'ui_'."
                    ))
                }
            }
            TOPIC_SHOT_HISTORY_REQUEST => {
                let range: Range = serde_json::from_slice(&self.payload)?;

                Ok(Event::IncomingMqttMessage(
                    MqttIncomingMessage::ShotHistoryRequest(range),
                ))
            }
            _ => Err(anyhow!(
                "There is no incoming message for the topic {}",
                topic
            )),
        }
    }
}

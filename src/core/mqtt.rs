use anyhow::{anyhow, Result};
use log::{debug, error, info};
use rumqttc::v5::{
    mqttbytes::{
        v5::{Packet, Publish},
        QoS,
    },
    AsyncClient, Event, MqttOptions,
};
use serde::Deserialize;
use std::{str, time::UNIX_EPOCH};
use tokio::{select, sync::broadcast::Sender, task, time};
use tokio_util::sync::CancellationToken;

use crate::{
    controller::ControlMethod,
    core::state::{Event as StateEvent, PowerState},
};

use super::{
    db::ConfigItem,
    state::{Mode, TemperatureMeasurement},
};

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
    event_tx: Sender<StateEvent>,
    cancel_token: CancellationToken,
    client: Option<AsyncClient>,
}

pub enum MqttOutgoingMessage {
    ModeUpdate(Mode),
    BoilerStatusUpdate(f32),
    TemperatureUpdate(TemperatureMeasurement),
    TemperatureHistoryResponse(String, String),
    TargetTemperatureUpdate(f32),
    ControlMethodUpdate(ControlMethod),
    PowerRelayStatus(PowerState),
    ShotHistoryResponse(String, String),
    ConfigUpdate(Vec<ConfigItem>),
}

impl TryInto<StateEvent> for Publish {
    type Error = anyhow::Error;

    fn try_into(self) -> std::result::Result<StateEvent, Self::Error> {
        let topic = str::from_utf8(&self.topic)?;

        match topic {
            TOPIC_CONTROL_METHOD_CHANGE_REQUEST => {
                let control_method = serde_yaml::from_slice(&self.payload)?;
                Ok(StateEvent::ControlMethodChangeRequest(control_method))
            }
            TOPIC_TARGET_TEMPERATURE_CHANGE_REQUEST => Ok(
                StateEvent::TargetTemperatureChangeRequest(serde_yaml::from_slice(&self.payload)?),
            ),
            TOPIC_MODE_CHANGE => {
                let mode: Mode = serde_yaml::from_slice(&self.payload)?;
                Ok(StateEvent::ModeChange(mode))
            }
            TOPIC_TEMPERATURE_HISTORY_REQUEST => {
                #[derive(Deserialize)]
                #[serde(rename_all = "camelCase")]
                struct TemperatureHistoryRequest {
                    id: String,
                    from: i64,
                    to: i64,
                    limit: Option<i64>,
                    bucket_size: Option<i64>,
                }

                let request: TemperatureHistoryRequest = serde_json::from_slice(&self.payload)?;

                Ok(StateEvent::TemperatureHistoryRequest {
                    id: request.id,
                    from: request.from,
                    to: request.to,
                    limit: request.limit,
                    bucket_size: request.bucket_size,
                })
            }
            TOPIC_EXTERN_POWER_STATE_CHANGE => {
                let power_state = if self.payload == "ON" {
                    PowerState::On
                } else {
                    PowerState::Off
                };

                Ok(StateEvent::PowerStateChange(power_state))
            }
            TOPIC_MANUAL_BOILER_HEAT_LEVEL_REQUEST => {
                let heat_level: f32 = serde_yaml::from_slice(&self.payload)?;

                Ok(StateEvent::ManualBoilerHeatLevelRequest(heat_level))
            }
            TOPIC_CONFIG_SET => {
                let config_item: ConfigItem = serde_json::from_slice(&self.payload)?;

                if config_item.key.starts_with("ui_") {
                    info!("Setting config {:?}", config_item);

                    Ok(StateEvent::WriteConfigItem(config_item))
                } else {
                    Err(anyhow!(
                        "Refusing to set a config entry that isn't prefixed with 'ui_'."
                    ))
                }
            }
            TOPIC_SHOT_HISTORY_REQUEST => {
                #[derive(Deserialize)]
                #[serde(rename_all = "camelCase")]
                struct ShotHistoryRequest {
                    id: String,
                    from: i64,
                    to: i64,
                    limit: Option<i64>,
                }

                let request: ShotHistoryRequest = serde_json::from_slice(&self.payload)?;

                Ok(StateEvent::ShotHistoryRequest {
                    id: request.id,
                    from: request.from,
                    to: request.to,
                    limit: request.limit,
                })
            }
            _ => Err(anyhow!(
                "There is no incoming message for the topic {}",
                topic
            )),
        }
    }
}

impl Mqtt {
    pub fn new(uri: &str, event_tx: Sender<StateEvent>) -> Result<Self> {
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

        let tx = self.event_tx.clone();

        self.publish(&MqttOutgoingMessage::ModeUpdate(Mode::Idle))
            .await?;

        task::spawn(async move {
            loop {
                select! {
                    Ok(notification) = event_loop.poll() => {
                        if let Event::Incoming(Packet::Publish(publish_event)) = notification {
                            debug!("Received = {:?}", publish_event);

                            match TryInto::<StateEvent>::try_into(publish_event) {
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

        let mut events: Vec<(String, String, bool)> = vec![];

        match message {
            MqttOutgoingMessage::ModeUpdate(status) => {
                events.push((
                    String::from("gesha/mode"),
                    serde_json::to_string(status)?,
                    true,
                ));
            }
            MqttOutgoingMessage::BoilerStatusUpdate(heat_level) => {
                events.push((
                    String::from("gesha/boiler_level"),
                    serde_json::to_string(heat_level)?,
                    true,
                ));
            }
            MqttOutgoingMessage::TemperatureUpdate(measurement) => {
                events.push((
                    format!("gesha/temperature/last_updated"),
                    measurement
                        .timestamp
                        .duration_since(UNIX_EPOCH)?
                        .as_millis()
                        .to_string(),
                    true,
                ));
                events.push((
                    format!("gesha/temperature/boiler"),
                    measurement.boiler_temp.to_string(),
                    true,
                ));

                events.push((
                    format!("gesha/temperature/grouphead"),
                    measurement.grouphead_temp.to_string(),
                    true,
                ));

                if let Some(thermofilter) = measurement.thermofilter_temp {
                    events.push((
                        format!("gesha/temperature/thermofilter"),
                        thermofilter.to_string(),
                        true,
                    ));
                }
            }
            MqttOutgoingMessage::TargetTemperatureUpdate(temp) => {
                events.push((
                    format!("gesha/temperature/target"),
                    serde_json::to_string(temp)?,
                    true,
                ));
            }
            MqttOutgoingMessage::ControlMethodUpdate(control_method) => {
                events.push((
                    format!("gesha/control_method"),
                    serde_json::to_string(control_method)?,
                    true,
                ));
            }
            MqttOutgoingMessage::PowerRelayStatus(power_status) => events.push((
                TOPIC_EXTERN_POWER_COMMAND.to_string(),
                (if power_status.clone() == PowerState::On {
                    "ON"
                } else {
                    "OFF"
                })
                .to_string(),
                false,
            )),
            MqttOutgoingMessage::TemperatureHistoryResponse(id, result) => events.push((
                format!("gesha/temperature/history/{id}"),
                result.to_string(),
                false,
            )),
            MqttOutgoingMessage::ShotHistoryResponse(id, result) => events.push((
                format!("gesha/shot/history/{id}"),
                result.to_string(),
                false,
            )),
            MqttOutgoingMessage::ConfigUpdate(entries) => {
                for entry in entries {
                    // I only want to broadcast UI config entries, not internal ones.
                    if entry.key.starts_with("ui_") {
                        events.push((
                            format!("gesha/config/{}", entry.key),
                            entry.value.to_string(),
                            true,
                        ))
                    }
                }
            }
        }

        for (topic, payload, retain) in events.iter() {
            self.client
                .as_ref()
                .unwrap()
                .publish(
                    topic,
                    QoS::ExactlyOnce,
                    retain.clone(),
                    String::from(payload),
                )
                .await
                .map_err(|err| anyhow!("Failed to publish status, got {}", err))?;
        }

        Ok(())
    }
}

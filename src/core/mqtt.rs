use anyhow::{anyhow, Result};
use bytes::{BufMut, Bytes, BytesMut};
use log::{debug, error};
use rumqttc::v5::{
    mqttbytes::{
        v5::{Packet, Publish},
        QoS,
    },
    AsyncClient, Event, MqttOptions,
};
use std::{str, time::UNIX_EPOCH};
use tokio::{select, sync::broadcast::Sender, task, time};
use tokio_util::sync::CancellationToken;

use crate::core::state::{Event as GeshaEvent, PowerState};

use super::{
    config::ControlMethod,
    state::{Mode, TemperatureMeasurement},
};

// const TOPIC_POWER_STATUS: &str = "ms-silvia-switch/status";
const TOPIC_POWER_STATE: &str = "ms-silvia-switch/switch/power/state";
// const TOPIC_POWER_COMMAND: &str = "ms-silvia-switch/switch/power/command";

const GESHA_TOPIC_CONTROL_METHOD_SET: &str = "gesha/control_method/set";
const GESHA_TOPIC_TARGET_TEMP_SET: &str = "gesha/temperature/target/set";

pub struct Mqtt {
    uri: String,
    event_tx: Sender<GeshaEvent>,
    cancel_token: CancellationToken,
    client: Option<AsyncClient>,
}

pub enum MqttOutgoingMessage {
    StatusUpdate(Mode),
    BoilerStatusUpdate(PowerState),
    TemperatureUpdate(TemperatureMeasurement),
    TargetTemperatureUpdate(f32),
    ControlMethodUpdate(ControlMethod),
}

impl TryInto<GeshaEvent> for Publish {
    type Error = anyhow::Error;

    fn try_into(self) -> std::result::Result<GeshaEvent, Self::Error> {
        let topic = str::from_utf8(&self.topic)?;

        match topic {
            GESHA_TOPIC_CONTROL_METHOD_SET => {
                let control_method = serde_yaml::from_slice(&self.payload)?;
                Ok(GeshaEvent::ControlMethodSet(control_method))
            }
            GESHA_TOPIC_TARGET_TEMP_SET => Ok(GeshaEvent::TargetTempSet(serde_yaml::from_slice(
                &self.payload,
            )?)),
            TOPIC_POWER_STATE => {
                let power_state = if self.payload == "ON" {
                    PowerState::On
                } else {
                    PowerState::Off
                };

                Ok(GeshaEvent::PowerStateUpdate(power_state))
            }
            _ => Err(anyhow!(
                "There is no incoming message for the topic {}",
                topic
            )),
        }
    }
}

impl Mqtt {
    pub fn new(uri: &str, event_tx: Sender<GeshaEvent>) -> Result<Self> {
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

        self.publish(&MqttOutgoingMessage::StatusUpdate(Mode::Idle))
            .await?;

        task::spawn(async move {
            loop {
                select! {
                    Ok(notification) = event_loop.poll() => {
                        if let Event::Incoming(Packet::Publish(publish_event)) = notification {
                            debug!("Received = {:?}", publish_event);

                            match TryInto::<GeshaEvent>::try_into(publish_event) {
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
                TOPIC_POWER_STATE,
                GESHA_TOPIC_CONTROL_METHOD_SET,
                GESHA_TOPIC_TARGET_TEMP_SET,
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
        struct Event<'a> {
            topic: &'a str,
            payload_string: Option<String>,
            payload_f32: Option<f32>,
            payload_u64: Option<u64>,
        }

        let mut events: Vec<Event> = vec![];

        match message {
            MqttOutgoingMessage::StatusUpdate(status) => {
                events.push(Event {
                    topic: "gesha/mode",
                    payload_string: Some(serde_json::to_string(status)?),
                    payload_f32: None,
                    payload_u64: None,
                });
            }
            MqttOutgoingMessage::BoilerStatusUpdate(power_state) => {
                events.push(Event {
                    topic: "gesha/boiler_status",
                    payload_string: Some(serde_json::to_string(power_state)?),
                    payload_f32: None,
                    payload_u64: None,
                });
            }
            MqttOutgoingMessage::TemperatureUpdate(measurement) => {
                // Cast to u64 since 64 bits takes us to 22 July 2554 *in nanoseconds*, let's not waste precious bytes!
                let last_updated = measurement.timestamp.duration_since(UNIX_EPOCH)?.as_millis() as u64;

                events.push(Event {
                    topic: "gesha/temperature/last_updated",
                    payload_string: None,
                    payload_f32: None,
                    payload_u64: Some(
                        last_updated,
                    ),
                });

                events.push(Event {
                    topic: "gesha/temperature/boiler",
                    payload_string: None,
                    payload_f32: Some(measurement.boiler_temp),
                    payload_u64: None,
                });

                events.push(Event {
                    topic: "gesha/temperature/grouphead",
                    payload_string: None,
                    payload_f32: Some(measurement.grouphead_temp),
                    payload_u64: None,
                });

                if let Some(thermofilter_temp) = measurement.thermofilter_temp {
                    events.push(Event {
                        topic: "gesha/temperature/thermofilter",
                        payload_string: None,
                        payload_f32: Some(thermofilter_temp),
                        payload_u64: None,
                    });
                }
            }
            MqttOutgoingMessage::TargetTemperatureUpdate(temp) => {
                events.push(Event {
                    topic: "gesha/temperature/target",
                    payload_string: None,
                    payload_f32: Some(*temp),
                    payload_u64: None,
                });
            }
            MqttOutgoingMessage::ControlMethodUpdate(control_method) => {
                events.push(Event {
                    topic: "gesha/temperature/target",
                    payload_string: Some(serde_json::to_string(control_method)?),
                    payload_f32: None,
                    payload_u64: None,
                });
            }
        }

        let client = self.client.as_ref().unwrap();

        for event in events.iter() {
            if let Some(payload) = &event.payload_string {
                client
                    .publish(event.topic, QoS::ExactlyOnce, true, String::from(payload))
                    .await
                    .map_err(|err| anyhow!("Failed to publish status, got {}", err))?;
            } else if let Some(payload) = event.payload_f32 {
                let mut bytes = BytesMut::with_capacity(4);
                bytes.put_f32(payload);

                client
                    .publish(event.topic, QoS::ExactlyOnce, true, bytes)
                    .await
                    .map_err(|err| anyhow!("Failed to publish status, got {}", err))?;
            } else if let Some(payload) = event.payload_u64 {
                let mut bytes = BytesMut::with_capacity(16);
                bytes.put_u64(payload);

                client
                    .publish(event.topic, QoS::ExactlyOnce, true, bytes)
                    .await
                    .map_err(|err| anyhow!("Failed to publish status, got {}", err))?;
            }
        }

        Ok(())
    }
}

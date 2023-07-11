mod controller;
mod core;

use crate::core::{
    config,
    mqtt::{self, MqttOutgoingMessage},
    state::{self, Event},
    thermocouple::poll_thermocouples,
};
use log::{debug, error, info, trace};
use pretty_env_logger;
use std::error::Error;
use tokio::{select, signal, sync::broadcast};
use tokio_util::sync::CancellationToken;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();

    let panic_cancel_token = create_panic_cancel_token();

    let config = config::Config::load()?;
    let config_clone = &config.clone();

    trace!("Using config:\n {:#?}", config);

    let mut state = state::State::default();

    let (tx, mut rx) = broadcast::channel::<Event>(100);

    let mut mqtt = mqtt::Mqtt::new(
        config.mqtt_url.expect("No MQTT server configured").as_ref(),
        tx.clone(),
    )?;

    mqtt.start().await?;

    let mut controller_manager =
        controller::ControllerManager::new(config.boiler_pin, &config.control_method, tx.clone())?;

    controller_manager.start()?;

    poll_thermocouples(config_clone, std::time::Duration::from_millis(250), tx.clone()).await?;

    loop {
        select! {
            Ok(event) = rx.recv() => {
                debug!("Event: {:?}", event);

                match state.update(event) {
                    Ok(mqtt_messages) => {
                        for message in mqtt_messages.iter() {
                            if let MqttOutgoingMessage::ControlMethodUpdate(control_method) = message {
                                controller_manager.set_controller(control_method)?;
                            }
                            mqtt.publish(&message).await?;
                        }
                    }
                    Err(err) => {
                        error!("Error updating state: {}", err);
                    }
                }
            },
            _ = signal::ctrl_c() => {
                debug!("Ctrl+C received");
                break;
            },
            _ = panic_cancel_token.cancelled() => {
                debug!("Panic Cancel Token triggered");
                break;
            }
        };
    }

    mqtt.stop().await?;
    controller_manager.stop()?;
    info!("Shutting down");

    Ok(())
}

fn create_panic_cancel_token() -> CancellationToken {
    let cancel_token = CancellationToken::new();
    let cancel_token_inner = cancel_token.clone();

    let default_panic = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        error!("Fatal error occurred: {info}");
        default_panic(info);
        cancel_token_inner.cancel();
    }));

    cancel_token
}

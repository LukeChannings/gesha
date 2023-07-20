mod controller;
mod core;

use crate::core::{
    config, db,
    mqtt::{self, MqttOutgoingMessage},
    state::{self, Event},
    thermocouple::ThermocouplePoller,
};
use log::{debug, error, info, trace};
use pretty_env_logger;
use std::error::Error;
use tokio::{select, signal::unix::{signal, SignalKind}, sync::broadcast};
use tokio_util::sync::CancellationToken;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();

    let panic_cancel_token = create_panic_cancel_token();

    let pool = db::open_or_create("/opt/gesha/var/db/gesha.db").await?;

    let config = config::Config::load().await?;
    let config_clone = &config.clone();

    trace!("Using config:\n {:#?}", config);

    let mut state = state::State::new(pool.clone());

    let (tx, mut rx) = broadcast::channel::<Event>(100);

    let mut mqtt = mqtt::Mqtt::new(
        config.mqtt_url.expect("No MQTT server configured").as_ref(),
        tx.clone(),
    )?;

    mqtt.start().await?;

    let mut controller_manager = controller::ControllerManager::new(
        config.boiler_pin,
        &config.control_method,
        tx.clone(),
        state.target_temp,
    )?;

    controller_manager.start()?;

    let mut thermocouples =
        ThermocouplePoller::new(state.mode.clone(), tx.clone(), config_clone.clone());

    thermocouples.poll()?;

    let mut hangup_signal = signal(SignalKind::hangup())?;
    let mut interrupt_signal = signal(SignalKind::interrupt())?;

    loop {
        select! {
            Ok(event) = rx.recv() => {
                debug!("Event: {:?}", event);

                match state.update(event).await {
                    Ok(mqtt_messages) => {
                        for message in mqtt_messages.iter() {
                            if let MqttOutgoingMessage::ControlMethodUpdate(control_method) = message {
                                controller_manager.set_controller(control_method).await?;
                            }

                            if let MqttOutgoingMessage::ModeUpdate(mode) = message {
                                thermocouples.update_mode(mode.clone()).await?;
                            }

                            if let MqttOutgoingMessage::TargetTemperatureUpdate(temp) = message {
                                controller_manager.set_target_temp(temp.clone()).await?;
                            }

                            mqtt.publish(&message).await?;
                        }
                    }
                    Err(err) => {
                        error!("Error updating state: {}", err);
                    }
                }
            },
            _ = interrupt_signal.recv() => {
                debug!("SIGINT received");
                break;
            },
            _ = hangup_signal.recv() => {
                debug!("SIGHUP received");
                break;
            }
            _ = panic_cancel_token.cancelled() => {
                debug!("Panic Cancel Token triggered");
                break;
            }
        };
    }

    info!("Shutting down");

    state.flush_measurements()?;
    mqtt.stop().await?;
    controller_manager.stop().await?;
    pool.close().await;

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

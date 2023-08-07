mod controller;
mod core;

use crate::core::{
    config,
    mqtt::{self, MqttOutgoingMessage},
    state::{self, Event, Mode},
    thermocouple::ThermocouplePoller,
};
use clap::Parser;
use log::{debug, error, info, trace};
use pretty_env_logger;
use std::error::Error;
use tokio::{
    select,
    signal::unix::{signal, SignalKind},
    sync::broadcast,
};
use tokio_util::sync::CancellationToken;

#[derive(Parser, Debug, Clone)]
struct Args {
    #[arg(short, long)]
    pub config_path: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();

    let args = Args::parse();

    let panic_cancel_token = create_panic_cancel_token();

    let config = config::Config::load(args.config_path).await?;
    let config_clone = &config.clone();

    trace!("Using config:\n {:#?}", config);

    let mut state = state::State::new().await?;

    let (tx, mut rx) = broadcast::channel::<Event>(100);

    let mut mqtt = mqtt::Mqtt::new(
        config.mqtt_url.expect("No MQTT server configured").as_ref(),
        tx.clone(),
    )?;

    mqtt.start().await?;

    let mut controller_manager = controller::ControllerManager::new(
        config.boiler_pin,
        &state.control_method,
        tx.clone(),
        state.target_temperature,
        state.mode.clone(),
    )?;

    controller_manager.start()?;

    let mut thermocouples =
        ThermocouplePoller::new(state.mode.clone(), tx.clone(), config_clone.clone());

    thermocouples.poll()?;

    let mut hangup_signal = signal(SignalKind::hangup())?;
    let mut interrupt_signal = signal(SignalKind::interrupt())?;

    mqtt.publish(&MqttOutgoingMessage::ControlMethodUpdate(
        state.control_method.clone(),
    ))
    .await?;

    mqtt.publish(&MqttOutgoingMessage::ModeUpdate(state.mode.clone()))
        .await?;

    mqtt.publish(&MqttOutgoingMessage::TargetTemperatureUpdate(
        state.target_temperature,
    ))
    .await?;

    tx.send(Event::ReadConfig)?;

    loop {
        select! {
            Ok(event) = rx.recv() => {

                match &event {
                    Event::TemperatureChange(_) => {}
                    event => {
                        info!("Event: {:?}", event);
                    }
                }

                if let Event::ModeChange(mode) = &event {
                    if state.mode == Mode::Steam && mode != &Mode::Steam {
                        controller_manager.set_target_temperature(state.target_temperature);
                        controller_manager.set_controller(&state.control_method).await?;
                    } else if mode == &Mode::Steam {
                        controller_manager.set_target_temperature(130.0);
                        controller_manager.set_controller(&controller::ControlMethod::Threshold).await?;
                    }
                }

                match state.handle_event(event).await {
                    Ok(mqtt_messages) => {
                        for message in mqtt_messages.iter() {
                            if let MqttOutgoingMessage::ControlMethodUpdate(control_method) = message {
                                controller_manager.set_controller(control_method).await?;
                            }

                            if let MqttOutgoingMessage::ModeUpdate(mode) = message {
                                thermocouples.update_mode(mode.clone()).await?;
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

    state.close()?;
    mqtt.stop().await?;
    controller_manager.stop().await?;

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

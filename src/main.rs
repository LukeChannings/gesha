use clap::Parser;
use gesha::{
    controller,
    core::{
        config,
        mqtt::Mqtt,
        state::{self, Event},
        thermocouple::ThermocouplePoller,
    },
};
use log::{debug, error, info, trace};
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

    let (tx, mut rx) = broadcast::channel::<Event>(10_000);

    let mut state = state::State::new(tx.clone()).await?;

    let mut mqtt = Mqtt::new(
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

    loop {
        select! {
            Ok(event) = rx.recv() => {
                match &event {
                    // The state is not updated by outgoing MQTT messages,
                    // they're use to update external devices to state changes that
                    // have already happened. We don't want to handle these events in the state.
                    Event::OutgoingMqttMessage(message) => {
                        mqtt.publish(&message).await?;
                    }
                    event => {

                        // Log all events except temperature changes -
                        // they can happen every 100ms and would spam the logs.
                        if !matches!(event, Event::TemperatureChanged(_)) {
                            info!("Received event: {:?}", event);
                        }

                        match state.handle_event(event).await {
                            Ok(events) => {
                                for event in events.iter() {
                                    tx.send(event.clone())?;
                                }
                            }
                            Err(err) => {
                                error!("Error updating state: {}", err);
                            }
                        }
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

    mqtt.stop().await?;
    controller_manager.stop().await?;
    state.stop().await?;

    drop(tx);

    let _ = rx.recv().await;

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

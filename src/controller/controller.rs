use anyhow::{anyhow, Result};
use log::{debug, error};
use rppal::gpio;
use tokio::{select, sync::broadcast::Sender, task};
use tokio_util::sync::CancellationToken;

use super::{MpcController, PidController, ThresholdController};
use crate::{
    config::ControlMethod,
    core::state::Mode,
    state::{Event as GeshaEvent, PowerState},
};

pub trait Controller: Send + Sync {
    fn sample(&self, boiler_temp: f32, group_head_temp: f32) -> bool;
}

pub struct ControllerManager {
    boiler_pin: u8,
    control_method: ControlMethod,
    cancel_token: CancellationToken,
    tx: Sender<GeshaEvent>,
}

impl ControllerManager {
    pub fn new(
        boiler_pin: u8,
        control_method: &ControlMethod,
        tx: Sender<GeshaEvent>,
    ) -> Result<Self> {
        let output_pin = gpio::Gpio::new()?.get(boiler_pin)?.into_output();

        if let Err(err) = tx.send(GeshaEvent::BoilerStateUpdate(if output_pin.is_set_high() {
            PowerState::On
        } else {
            PowerState::Off
        })) {
            return Err(anyhow!("Error sending initial boiler state: {}", err));
        };

        Ok(ControllerManager {
            boiler_pin,
            control_method: control_method.clone(),
            cancel_token: CancellationToken::new(),
            tx,
        })
    }

    pub fn start(&self) -> Result<()> {
        let cancel_token = self.cancel_token.clone();

        let tx = self.tx.clone();
        let mut rx = self.tx.subscribe();

        let mut output_pin = gpio::Gpio::new()?.get(self.boiler_pin)?.into_output();
        let control_method = self.control_method.clone();

        task::spawn(async move {
            let controller: Option<Box<dyn Controller>> = (&control_method).into();
            let mut mode: Option<Mode> = None;

            loop {
                select! {
                    Ok(event) = rx.recv() => {
                        match event {
                            GeshaEvent::TempUpdate(temp) => {
                                let mut boiler_state_changed = false;
                                if let Some(Mode::Heat) = mode {
                                    if let Some(controller) = &controller {
                                        let should_heat = controller.sample(temp.boiler_temp, temp.grouphead_temp);

                                        if should_heat && output_pin.is_set_low() {
                                            output_pin.set_high();
                                            boiler_state_changed = true;
                                        } else if !should_heat && output_pin.is_set_high() {
                                            output_pin.set_low();
                                            boiler_state_changed = true;
                                        }
                                    }
                                } else if output_pin.is_set_high() {
                                    output_pin.set_low();
                                    boiler_state_changed = true;
                                }

                                if boiler_state_changed {
                                    if let Err(err) = tx.send(GeshaEvent::BoilerStateUpdate(
                                        if output_pin.is_set_high() {
                                            PowerState::On
                                        } else {
                                            PowerState::Off
                                        })) {
                                        error!("Error sending boiler state: {}", err);
                                    };
                                }
                            },
                            GeshaEvent::PowerStateUpdate(state) => {
                                mode = Some(if state == PowerState::Off { Mode::Idle } else { Mode::Heat });
                            },
                            _ => {
                                // ignore other events.
                            }
                        }
                    },
                    _ = cancel_token.cancelled() => {
                        debug!("Controller manager stopped");
                        output_pin.set_low();
                        break;
                    }
                }
            }

            if let Err(err) = tx.send(GeshaEvent::BoilerStateUpdate(PowerState::Off)) {
                error!("Error sending boiler state: {}", err);
            }
        });

        Ok(())
    }

    pub fn stop(&self) -> Result<()> {
        self.cancel_token.cancel();
        Ok(())
    }

    pub fn set_controller(&mut self, control_method: &ControlMethod) -> Result<()> {
        self.stop()?;

        self.control_method = control_method.clone();

        self.start()?;

        Ok(())
    }
}

impl Into<Option<Box<dyn Controller>>> for &ControlMethod {
    fn into(self) -> Option<Box<dyn Controller>> {
        match self {
            ControlMethod::Threshold => Some(Box::new(ThresholdController::new(90.0))),
            ControlMethod::MPC => Some(Box::new(MpcController::new())),
            ControlMethod::PID => Some(Box::new(PidController::new(1.0, 1.0, 1.0))),
            ControlMethod::Manual => None,
        }
    }
}

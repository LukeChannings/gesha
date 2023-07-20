use std::time::Duration;

use anyhow::{anyhow, Result};
use log::{debug, error, info};
use rppal::gpio;
use tokio::{
    select,
    sync::broadcast::Sender,
    task::{self, JoinHandle},
};
use tokio_util::sync::CancellationToken;

use super::{ManualController, MpcController, PidController, ThresholdController};
use crate::{
    config::ControlMethod,
    core::state::Mode,
    state::{Event as GeshaEvent, PowerState},
};

pub trait Controller: Send + Sync {
    fn sample(&self, boiler_temp: f32, group_head_temp: f32) -> f32;
}

pub struct ControllerManager {
    boiler_pin: u8,
    control_method: ControlMethod,
    cancel_token: CancellationToken,
    tx: Sender<GeshaEvent>,
    target_temp: f32,
    controller_handle: Option<JoinHandle<()>>,
}

impl ControllerManager {
    pub fn new(
        boiler_pin: u8,
        control_method: &ControlMethod,
        tx: Sender<GeshaEvent>,
        target_temp: f32,
    ) -> Result<Self> {
        let mut output_pin = gpio::Gpio::new()?.get(boiler_pin)?.into_output();
        output_pin.set_low();

        if let Err(err) = tx.send(GeshaEvent::BoilerStateUpdate(0.0)) {
            return Err(anyhow!("Error sending initial boiler state: {}", err));
        };

        Ok(ControllerManager {
            boiler_pin,
            control_method: control_method.clone(),
            cancel_token: CancellationToken::new(),
            tx,
            target_temp,
            controller_handle: None,
        })
    }

    pub fn start(&mut self) -> Result<()> {
        let cancel_token = self.cancel_token.clone();

        let tx = self.tx.clone();
        let mut rx = self.tx.subscribe();

        let mut output_pin = gpio::Gpio::new()?.get(self.boiler_pin)?.into_output();
        let control_method = self.control_method.clone();
        let target_temp = self.target_temp.clone();

        let handle = task::spawn(async move {
            let controller: Option<Box<dyn Controller>> =
                ControllerManager::get_controller(&control_method, target_temp);
            let mut mode: Option<Mode> = None;
            let mut current_duty_cycle: f32 = 0.0;

            loop {
                select! {
                    Ok(event) = rx.recv() => {
                        match event {
                            GeshaEvent::TempUpdate(temp) => {
                                let mut boiler_state_changed = false;
                                if let Some(Mode::Active) = mode {
                                    if let Some(controller) = &controller {
                                        let duty_cycle = controller.sample(temp.boiler_temp, temp.grouphead_temp);

                                        info!("Duty cycle: {:?}", duty_cycle);

                                        if duty_cycle != current_duty_cycle {
                                            let (period, pulse_width) = duty_cycle_to_pulse_width(duty_cycle).unwrap();
                                            output_pin.set_pwm(period, pulse_width).unwrap();
                                            current_duty_cycle = duty_cycle;
                                            boiler_state_changed = true;
                                        }
                                    }
                                } else if output_pin.is_set_high() {
                                    output_pin.set_low();
                                    current_duty_cycle = 0.0;
                                    boiler_state_changed = true;
                                }

                                if boiler_state_changed {
                                    if let Err(err) = tx.send(GeshaEvent::BoilerStateUpdate(current_duty_cycle)) {
                                        error!("Error sending boiler state: {}", err);
                                    };
                                }
                            },
                            GeshaEvent::PowerStateUpdate(state) => {
                                mode = Some(if state == PowerState::Off { Mode::Idle } else { Mode::Active });
                            }
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

            if let Err(err) = tx.send(GeshaEvent::BoilerStateUpdate(0.0)) {
                error!("Error sending boiler state: {}", err);
            }
        });

        self.controller_handle = Some(handle);

        Ok(())
    }

    pub async fn stop(&mut self) -> Result<()> {
        if let Some(handle) = self.controller_handle.take() {
            self.cancel_token.cancel();
            let _ = handle.await;
        }
        Ok(())
    }

    pub async fn set_controller(&mut self, control_method: &ControlMethod) -> Result<()> {
        self.stop().await?;

        self.control_method = control_method.clone();

        self.start()?;

        Ok(())
    }

    pub async fn set_target_temp(&mut self, temp: f32) -> Result<()> {
        self.stop().await?;
        self.target_temp = temp;
        self.start()?;

        Ok(())
    }

    pub fn get_controller(
        control_method: &ControlMethod,
        target_temp: f32,
    ) -> Option<Box<dyn Controller>> {
        match control_method {
            ControlMethod::Threshold => Some(Box::new(ThresholdController::new(target_temp))),
            ControlMethod::MPC => Some(Box::new(MpcController::new(target_temp))),
            ControlMethod::PID => Some(Box::new(PidController::new(1.0, 1.0, 1.0, target_temp))),
            ControlMethod::Manual => Some(Box::new(ManualController::new())),
        }
    }
}

// The duty cycle is a percentage represented as 0.0 - 1.0, 0% and 100% respectively.
fn duty_cycle_to_pulse_width(duty_cycle: f32) -> Result<(Duration, Duration)> {
    if duty_cycle > 1.0 || duty_cycle < 0.0 {
        return Err(anyhow!("The duty cycle must be between 0.0 and 1.0"));
    }

    let period = 50.0;
    let pulse_width = (period * duty_cycle).ceil();

    let period_duration = Duration::from_millis(period as u64);
    let pulse_width_duration = Duration::from_millis(pulse_width as u64);

    Ok((period_duration, pulse_width_duration))
}

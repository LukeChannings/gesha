use std::{
    sync::{Arc, RwLock},
    time::Duration,
};

use anyhow::{anyhow, Result};
use log::{debug, error, info};
use rppal::gpio;
use serde::{Deserialize, Serialize};
use tokio::{
    select,
    sync::broadcast::Sender,
    task::{self, JoinHandle},
};
use tokio_util::sync::CancellationToken;

use super::{MpcController, PidController, ThresholdController};
use crate::{
    core::state::Mode,
    state::{Event as StateEvent, PowerState},
};

pub trait Controller: Send + Sync {
    fn sample(&self, boiler_temp: f32, group_head_temp: f32) -> f32;
    fn update_target_temperature(&mut self, target_temp: f32);
}

pub struct ControllerManager {
    boiler_pin: u8,
    control_method: ControlMethod,
    cancel_token: CancellationToken,
    tx: Sender<StateEvent>,
    target_temp: f32,
    controller_handle: Option<JoinHandle<()>>,
    mode: Arc<RwLock<Mode>>,
}

impl ControllerManager {
    pub fn new(
        boiler_pin: u8,
        control_method: &ControlMethod,
        tx: Sender<StateEvent>,
        target_temp: f32,
        mode: Mode,
    ) -> Result<Self> {
        let mut output_pin = gpio::Gpio::new()?.get(boiler_pin)?.into_output();
        output_pin.set_low();

        if let Err(err) = tx.send(StateEvent::BoilerHeatLevelChange(0.0)) {
            return Err(anyhow!("Error sending initial boiler state: {}", err));
        };

        Ok(ControllerManager {
            boiler_pin,
            control_method: control_method.clone(),
            cancel_token: CancellationToken::new(),
            tx,
            target_temp,
            controller_handle: None,
            mode: Arc::new(RwLock::new(mode)),
        })
    }

    pub fn start(&mut self) -> Result<()> {
        info!("Starting controller {:?}", &self.control_method);

        let cancel_token = self.cancel_token.clone();

        let tx = self.tx.clone();
        let mut rx = self.tx.subscribe();

        let mut output_pin = gpio::Gpio::new()?.get(self.boiler_pin)?.into_output();
        let control_method = self.control_method.clone();
        let target_temp = self.target_temp.clone();

        let mode = Arc::clone(&self.mode);

        let handle = task::spawn(async move {
            let mut controller: Option<Box<dyn Controller>> =
                ControllerManager::get_controller(&control_method, target_temp);
            let mut current_duty_cycle: f32 = 0.0;

            loop {
                select! {
                    Ok(event) = rx.recv() => {
                        match event {
                            StateEvent::TemperatureChange(temp) => {
                                let mut boiler_state_changed = false;

                                match mode.try_read() {
                                    Ok(mode) => {
                                        if let Mode::Active = *mode {
                                            if let Some(controller) = &controller {
                                                let duty_cycle = controller.sample(temp.boiler_temp, temp.grouphead_temp);

                                                if duty_cycle != current_duty_cycle {
                                                    let (period, pulse_width) = duty_cycle_to_pulse_width(duty_cycle).unwrap();
                                                    info!("Duty Cycle: {}, Period: {:?}, Pulse Width: {:?}", duty_cycle, period, pulse_width);
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
                                            if let Err(err) = tx.send(StateEvent::BoilerHeatLevelChange(current_duty_cycle)) {
                                                error!("Error sending boiler state: {}", err);
                                            };
                                        }
                                    }
                                    Err(err) => {
                                        info!("Failed to read mode: {err}");
                                    }
                                }
                            }

                            StateEvent::PowerStateChange(state) => {
                                let mut mode = mode.write().unwrap();
                                *mode = if state == PowerState::Off { Mode::Idle } else { Mode::Active };
                            }

                            StateEvent::ManualBoilerHeatLevelRequest(duty_cycle) => {
                                if let Mode::Idle = *mode.read().unwrap() {
                                    continue;
                                }

                                if controller.is_some() {
                                    continue
                                }

                                let (period, pulse_width) = duty_cycle_to_pulse_width(duty_cycle).unwrap();
                                output_pin.set_pwm(period, pulse_width).unwrap();
                                current_duty_cycle = duty_cycle;

                                if let Err(err) = tx.send(StateEvent::BoilerHeatLevelChange(current_duty_cycle)) {
                                    error!("Error sending boiler state: {}", err);
                                };
                            }

                            StateEvent::TargetTemperatureChangeRequest(target_temperature) => {
                                if let Some(controller) = &mut controller {
                                    controller.update_target_temperature(target_temperature)
                                }
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

            if let Err(err) = tx.send(StateEvent::BoilerHeatLevelChange(0.0)) {
                error!("Error sending boiler state: {}", err);
            }
        });

        self.controller_handle = Some(handle);

        Ok(())
    }

    pub async fn stop(&mut self) -> Result<()> {
        if let Some(handle) = self.controller_handle.take() {
            info!("Shutting down controller {:?}", self.control_method);
            self.cancel_token.cancel();
            let _ = handle.await;
            self.cancel_token = CancellationToken::new();
        }
        Ok(())
    }

    pub async fn set_controller(&mut self, control_method: &ControlMethod) -> Result<()> {
        self.stop().await?;

        self.control_method = control_method.clone();

        self.start()?;

        Ok(())
    }

    pub fn get_controller(
        control_method: &ControlMethod,
        target_temperature: f32,
    ) -> Option<Box<dyn Controller>> {
        match control_method {
            ControlMethod::Threshold => {
                Some(Box::new(ThresholdController::new(target_temperature)))
            }
            ControlMethod::PID => Some(Box::new(PidController::new(
                1.0,
                1.0,
                1.0,
                target_temperature,
            ))),
            ControlMethod::MPC => Some(Box::new(MpcController::new(target_temperature))),
            ControlMethod::None => None,
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

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum ControlMethod {
    // If the current temperature is < threshold, turn heat on, otherwise off.
    #[serde(alias = "threshold", alias = "THRESHOLD")]
    Threshold,

    // https://en.wikipedia.org/wiki/PID_controller
    #[serde(alias = "pid", alias = "Pid")]
    PID,

    // https://en.wikipedia.org/wiki/Model_predictive_control
    #[serde(alias = "mpc", alias = "Mpc")]
    MPC,

    #[serde(alias = "none")]
    None,
}

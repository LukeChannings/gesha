use std:: time::Duration;

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
    core::state::{Mode, IsPowerOn},
    core::state::Event,
};

pub trait Controller: Send + Sync {
    fn sample(&mut self, boiler_temp: f32, grouphead_temp: f32) -> f32;
    fn update_target_temperature(&mut self, target_temp: f32);
}

pub struct ControllerManager {
    boiler_pin: u8,
    control_method: ControlMethod,
    cancel_token: CancellationToken,
    tx: Sender<Event>,
    target_temperature: f32,
    controller_handle: Option<JoinHandle<()>>,
    mode: Mode,
}

impl ControllerManager {
    pub fn new(
        boiler_pin: u8,
        control_method: &ControlMethod,
        tx: Sender<Event>,
        target_temp: f32,
        mode: Mode,
    ) -> Result<Self> {
        let mut output_pin = gpio::Gpio::new()?.get(boiler_pin)?.into_output();
        output_pin.set_low();

        if let Err(err) = tx.send(Event::BoilerHeatLevelChanged(0.0)) {
            return Err(anyhow!("Error sending initial boiler state: {}", err));
        };

        Ok(ControllerManager {
            boiler_pin,
            control_method: control_method.clone(),
            cancel_token: CancellationToken::new(),
            tx,
            target_temperature: target_temp,
            controller_handle: None,
            mode,
        })
    }

    pub fn start(&mut self) -> Result<()> {
        info!("Starting controller {:?}", &self.control_method);

        let cancel_token = self.cancel_token.clone();

        let tx = self.tx.clone();
        let mut rx = self.tx.subscribe();

        let mut output_pin = gpio::Gpio::new()?.get(self.boiler_pin)?.into_output();
        let mut current_target_temperature = self.target_temperature.clone();
        let mut mode = self.mode.clone();
        let mut controller: Option<Box<dyn Controller>> =
            ControllerManager::get_controller(&self.control_method, current_target_temperature);

        let mut power_state: IsPowerOn = true;

        let handle = task::spawn(async move {
            let mut current_duty_cycle: f32 = 0.0;

            loop {
                select! {
                    Ok(event) = rx.recv() => {
                        match event {
                            Event::ControlMethodChanged(control_method) => {
                                info!("Control method changed to {:?}", control_method);
                                controller = ControllerManager::get_controller(&control_method, current_target_temperature);
                            }
                            Event::TemperatureChanged(temp) => {
                                let mut boiler_state_changed = false;

                                if mode != Mode::Idle {
                                    if let Some(controller) = &mut controller {
                                        let duty_cycle = normalize_duty_cycle(controller.sample(temp.boiler_temp, temp.grouphead_temp));

                                        if current_duty_cycle != duty_cycle {
                                            let (period, pulse_width) = duty_cycle_to_pulse_width(duty_cycle).unwrap();
                                            info!("Duty Cycle: {}, Period: {:?}, Pulse Width: {:?}", duty_cycle, period, pulse_width);
                                            output_pin.set_pwm(period, pulse_width).unwrap();
                                            current_duty_cycle = duty_cycle;
                                            boiler_state_changed = true;
                                        }
                                    }
                                } else if output_pin.is_set_high() && power_state == true {
                                    output_pin.set_low();
                                    current_duty_cycle = 0.0;
                                    boiler_state_changed = true;
                                    println!("Pin high state changed");
                                }

                                if boiler_state_changed {
                                    if let Err(err) = tx.send(Event::BoilerHeatLevelChanged(current_duty_cycle)) {
                                        error!("Error sending boiler state: {}", err);
                                    };
                                }
                            }

                            Event::PowerStateChanged(new_power_state) => {
                                power_state = new_power_state;
                            }

                            Event::ModeChanged(new_mode) => {
                                mode = new_mode;
                            }

                            Event::ManualBoilerHeatLevelRequest(duty_cycle) => {
                                if mode == Mode::Idle || controller.is_some(){
                                    continue;
                                }

                                let (period, pulse_width) = duty_cycle_to_pulse_width(duty_cycle).unwrap();
                                info!("Setting duty cycle period={:#?}, pulse_width={:#?}", period, pulse_width);
                                output_pin.set_pwm(period, pulse_width).unwrap();
                                current_duty_cycle = duty_cycle;

                                if let Err(err) = tx.send(Event::BoilerHeatLevelChanged(current_duty_cycle)) {
                                    error!("Error sending boiler state: {}", err);
                                };
                            }

                            Event::TargetTemperatureChanged(next_target_temperature) => {
                                if let Some(controller) = &mut controller {
                                    current_target_temperature = next_target_temperature;
                                    controller.update_target_temperature(next_target_temperature)
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

            if let Err(err) = tx.send(Event::BoilerHeatLevelChanged(0.0)) {
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

    pub fn get_controller(
        control_method: &ControlMethod,
        target_temperature: f32,
    ) -> Option<Box<dyn Controller>> {
        match control_method {
            ControlMethod::Threshold => {
                Some(Box::new(ThresholdController::new(target_temperature)))
            }
            ControlMethod::PID => Some(Box::new(PidController::new(
                45.0,
                1.0,
                60.0,
                target_temperature,
            ))),
            ControlMethod::MPC => Some(Box::new(MpcController::new(target_temperature))),
            ControlMethod::None => None,
        }
    }

    pub fn set_target_temperature(&mut self, target_temperature: f32) {
        self.target_temperature = target_temperature;
    }
}

// Round the duty cycle to increments of 0.1
// This is to avoid resetting the software PWM unnecessarily
fn normalize_duty_cycle(duty_cycle: f32) -> f32 {
    if duty_cycle < 0.0 || duty_cycle > 100.0 {
        info!("Duty cycle out of range: {duty_cycle} (will return 0.0)");
    }

    match (duty_cycle * 100.0) as usize {
        0..=9 => 0.0,
        10..=19 => 0.1,
        20..=29 => 0.2,
        30..=39 => 0.4,
        40..=49 => 0.5,
        50..=59 => 0.6,
        60..=69 => 0.7,
        70..=79 => 0.8,
        80..=89 => 0.9,
        90..=100 => 1.0,
        _ => 0.0,
    }
}

// The duty cycle is a percentage represented as 0.0 - 1.0, 0% and 100% respectively.
fn duty_cycle_to_pulse_width(duty_cycle: f32) -> Result<(Duration, Duration)> {
    if duty_cycle > 1.0 || duty_cycle < 0.0 {
        return Err(anyhow!(
            "The duty cycle must be between 0.0 and 1.0, but got {duty_cycle}"
        ));
    }

    let period = 100.0;

    // 0.1 = 10ms
    // 0.2 = 20ms
    // 1.0 = 100ms
    // etc
    let pulse_width = (period * duty_cycle).ceil();

    let period_duration = Duration::from_millis(period as u64);
    let pulse_width_duration = Duration::from_millis(pulse_width as u64);

    Ok((period_duration, pulse_width_duration))
}

#[derive(PartialEq, Clone, Copy, Debug, Serialize, Deserialize)]
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

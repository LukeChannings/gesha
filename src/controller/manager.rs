use std::time::Duration;

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

use super::{PidController, PredictiveController, ThresholdController};
use crate::{
    core::state::Event,
    core::{
        state::{IsPowerOn, Mode},
        util::FixedCapacityQueue,
    },
};

pub trait Controller: Send + Sync {
    fn sample(&mut self, boiler_temp: f32, grouphead_temp: f32, q: f32) -> f32;
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

        // The sample function is called at a 100ms interval, which is the precision with which we need to store heat_level records.
        // The predictive model we're using has a maximum start_lag of 125 seconds, which is what we use for our look-back period.
        // 50 seconds / 100ms = 500.
        let mut q = FixedCapacityQueue::<u8>::new(500);
        let mut current_boiler_temp: f32 = 0.0;
        let mut current_grouphead_temp: f32 = 0.0;
        let mut power_state: IsPowerOn = true;

        let handle = task::spawn(async move {
            let mut current_duty_cycle: u8 = 0;

            let mut interval = tokio::time::interval(Duration::from_millis(100));

            loop {
                select! {
                    _ = interval.tick() => {
                        let mut boiler_state_changed = false;

                        if power_state == false && current_duty_cycle != 0 && output_pin.is_set_high() {
                            output_pin.set_low();
                            current_duty_cycle = 0;
                        }

                        if mode != Mode::Idle {
                            if let Some(controller) = &mut controller {
                                let duty_cycle = normalize_duty_cycle(controller.sample(current_boiler_temp, current_grouphead_temp, (q.sum as f32) / 10.0));

                                if current_duty_cycle != duty_cycle {
                                    let (period, pulse_width) = duty_cycle_to_pulse_width(duty_cycle).unwrap();
                                    info!("Duty Cycle: {}, Period: {:?}, Pulse Width: {:?}", duty_cycle, period, pulse_width);
                                    output_pin.set_pwm(period, pulse_width).unwrap();
                                    current_duty_cycle = duty_cycle;
                                    boiler_state_changed = true;
                                }
                            }
                        } else if current_duty_cycle == 0 && output_pin.is_set_high() {
                            output_pin.set_low();
                            current_duty_cycle = 0;
                            boiler_state_changed = true;
                            println!("Pin high state changed");
                        }

                        q.push(current_duty_cycle);

                        if boiler_state_changed {
                            if let Err(err) = tx.send(Event::BoilerHeatLevelChanged(current_duty_cycle as f32 / 10.0)) {
                                error!("Error sending boiler state: {}", err);
                            };
                        }

                    },
                    Ok(event) = rx.recv() => {
                        match event {
                            Event::ControlMethodChanged(control_method) => {
                                info!("Control method changed to {:?}", control_method);
                                controller = ControllerManager::get_controller(&control_method, current_target_temperature);
                            }
                            Event::TemperatureChanged(temp) => {
                                current_boiler_temp = temp.boiler_temp;
                                current_grouphead_temp = temp.grouphead_temp;
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

                                let normalised_duty_cycle = normalize_duty_cycle(duty_cycle);

                                let (period, pulse_width) = duty_cycle_to_pulse_width(normalised_duty_cycle).unwrap();
                                info!("Setting duty cycle period={:#?}, pulse_width={:#?}", period, pulse_width);
                                output_pin.set_pwm(period, pulse_width).unwrap();
                                current_duty_cycle = normalised_duty_cycle;

                                if let Err(err) = tx.send(Event::BoilerHeatLevelChanged(duty_cycle)) {
                                    error!("Error sending boiler state: {}", err);
                                };
                            }

                            Event::TargetTemperatureChanged(next_target_temperature) => {
                                if let Some(controller) = &mut controller {
                                    current_target_temperature = next_target_temperature;
                                    controller.update_target_temperature(next_target_temperature);
                                    info!("Updating target temperature to {}", next_target_temperature);
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
            ControlMethod::Predictive => {
                Some(Box::new(PredictiveController::new(target_temperature)))
            }
            ControlMethod::None => None,
        }
    }

    pub fn set_target_temperature(&mut self, target_temperature: f32) {
        self.target_temperature = target_temperature;
    }
}

// Round the duty cycle to increments of 0.1
// This is to avoid resetting the software PWM unnecessarily
fn normalize_duty_cycle(duty_cycle: f32) -> u8 {
    if duty_cycle < 0.0 || duty_cycle > 1.0 {
        info!("Duty cycle out of range: {duty_cycle} (will return 0.0)");
    }

    return (duty_cycle * 10.0).round() as u8;
}

// The duty cycle is a percentage represented as 0.0 - 1.0, 0% and 100% respectively.
fn duty_cycle_to_pulse_width(duty_cycle: u8) -> Result<(Duration, Duration)> {
    if duty_cycle > 10 {
        return Err(anyhow!(
            "The duty cycle must be between 0 and 10, but got {duty_cycle}"
        ));
    }

    let period = 100;

    // A full period is 100ms
    // 1 = 10ms
    // 2 = 20ms
    // 10 = 100ms
    // etc
    let pulse_width = (period * duty_cycle as u64) / 10;

    let period_duration = Duration::from_millis(period);
    let pulse_width_duration = Duration::from_millis(pulse_width);

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

    #[serde(alias = "predictive", alias = "Predictive")]
    Predictive,

    #[serde(alias = "none")]
    None,
}

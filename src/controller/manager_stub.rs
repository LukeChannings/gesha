use anyhow::Result;
use crate::{
    core::state::Event,
    core::state::Mode,
};
use serde::{Deserialize, Serialize};
use std::sync::mpsc::Sender;

pub trait Controller: Send + Sync {
    fn sample(&mut self, boiler_temp: f32, grouphead_temp: f32) -> f32;
    fn update_target_temperature(&mut self, target_temp: f32);
}

pub struct ControllerManager { }

impl ControllerManager {
    pub fn new(
        _boiler_pin: u8,
        _control_method: &ControlMethod,
        _tx: Sender<Event>,
        _target_temp: f32,
        _mode: Mode,
    ) -> Result<Self> {
        Ok(ControllerManager {})
    }

    pub fn start(&mut self) -> Result<()> { Ok(())}
    pub async fn stop(&mut self) -> Result<()> { Ok(()) }
    pub fn set_target_temperature(&mut self, _target_temperature: f32) { }
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

    #[serde(alias = "hold")]
    Hold,

    #[serde(alias = "none")]
    None,
}

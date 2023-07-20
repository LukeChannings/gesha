use super::Controller;

pub struct ManualController {}

impl ManualController {
    pub fn new() -> ManualController {
        ManualController {}
    }
}

impl Controller for ManualController {
    fn sample(&self, _current_temp: f32, _target_temp: f32) -> f32 {
        0.0
    }
}

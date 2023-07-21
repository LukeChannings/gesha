use crate::controller::Controller;

pub struct MpcController {
    target_temperature: f32,
}

impl MpcController {
    pub fn new(target_temperature: f32) -> Self {
        MpcController { target_temperature }
    }
}

impl Controller for MpcController {
    fn sample(&self, _boiler_temp: f32, _group_head_temp: f32) -> f32 {
        0.0
    }

    fn update_target_temperature(&mut self, target_temperature: f32) {
        self.target_temperature = target_temperature;
    }
}

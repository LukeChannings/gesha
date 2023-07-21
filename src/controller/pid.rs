use super::Controller;

pub struct PidController {
    target_temperature: f32,
    p: f32,
    i: f32,
    d: f32,
}

impl PidController {
    pub fn new(p: f32, i: f32, d: f32, target_temperature: f32) -> Self {
        PidController {
            p,
            i,
            d,
            target_temperature,
        }
    }
}

impl Controller for PidController {
    fn sample(&self, _boiler_temp: f32, _group_head_temp: f32) -> f32 {
        0.0
    }

    fn update_target_temperature(&mut self, target_temperature: f32) {
        self.target_temperature = target_temperature;
    }
}

use super::Controller;

pub struct ThresholdController {
    target_temperature: f32,
}

impl ThresholdController {
    pub fn new(target_temperature: f32) -> Self {
        ThresholdController { target_temperature }
    }
}

impl Controller for ThresholdController {
    fn sample(&mut self, boiler_temp: f32, _grouphead_temp: f32, _q: f32) -> f32 {
        if boiler_temp < self.target_temperature {
            1.0
        } else {
            0.0
        }
    }

    fn update_target_temperature(&mut self, target_temp: f32) {
        self.target_temperature = target_temp;
    }
}

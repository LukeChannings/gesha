use log::info;

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
    fn sample(&mut self, boiler_temp: f32, _group_head_temp: f32) -> f32 {
        info!(
            "Threshold sample: {}, {}, {:?}",
            boiler_temp,
            self.target_temperature,
            boiler_temp < self.target_temperature
        );
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

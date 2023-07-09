use super::Controller;

pub struct ThresholdController {
    threshold: f32,
}

impl ThresholdController {
    pub fn new(threshold: f32) -> Self {
        ThresholdController { threshold }
    }
}

impl Controller for ThresholdController {
    fn sample(&self, boiler_temp: f32, _group_head_temp: f32) -> bool {
        boiler_temp < self.threshold
    }
}

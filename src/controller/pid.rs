use super::Controller;

pub struct PidController {
    target_temp: f32,
    p: f32,
    i: f32,
    d: f32,
}

impl PidController {
    pub fn new(p: f32, i: f32, d: f32, target_temp: f32) -> Self {
        PidController {
            p,
            i,
            d,
            target_temp,
        }
    }
}

impl Controller for PidController {
    fn sample(&self, _boiler_temp: f32, _group_head_temp: f32) -> f32 {
        println!("P: {}, I: {}, D: {}", self.p, self.i, self.d);
        todo!("PID Controller not implemented")
    }
}

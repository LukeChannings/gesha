use crate::controller::Controller;

pub struct MpcController {
    target_temp: f32
}

impl MpcController {
    pub fn new(target_temp: f32) -> Self {
        MpcController {
            target_temp
        }
    }
}

impl Controller for MpcController {
    fn sample(&self, _boiler_temp: f32, _group_head_temp: f32) -> f32 {
        todo!("MPC Controller not implemented")
    }
}

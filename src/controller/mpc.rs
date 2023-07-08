use crate::controller::Controller;

pub struct MpcController {}

impl MpcController {
    pub fn new() -> Self {
        MpcController { }
    }
}

impl Controller for MpcController {
    fn sample(&self, _boiler_temp: f32, _group_head_temp: f32) -> bool {
        todo!("MPC Controller not implemented")
    }
}

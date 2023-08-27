use super::Controller;
use pid::Pid;

pub struct PidController {
    target_temperature: f32,
    pid: Pid<f32>,
}

impl PidController {
    pub fn new(p: f32, i: f32, d: f32, target_temperature: f32) -> Self {
        let mut pid = Pid::<f32>::new(target_temperature, 100.0);

        pid.p(p, 100.0).i(i, 100.0).d(d, 100.0);

        PidController {
            pid,
            target_temperature,
        }
    }
}

impl Controller for PidController {
    fn sample(&mut self, boiler_temp: f32, _grouphead_temp: f32) -> f32 {
        let output = self.pid.next_control_output(boiler_temp);

        let output = (output.output + 100.0) / 200.0;
        output
    }

    fn update_target_temperature(&mut self, target_temperature: f32) {
        self.target_temperature = target_temperature;
        self.pid.setpoint(target_temperature);
    }
}

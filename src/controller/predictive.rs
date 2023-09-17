use log::{error, info};

use crate::{controller::Controller, models::PredictiveModels};

pub struct PredictiveController {
    target_temperature: f32,
    model: PredictiveModels,
}

impl PredictiveController {
    pub fn new(target_temperature: f32) -> Self {
        PredictiveController {
            target_temperature,
            model: PredictiveModels::new().unwrap(),
        }
    }
}

impl Controller for PredictiveController {
    fn sample(&mut self, boiler_temp_c: f32, grouphead_temp_c: f32, q: f32) -> f32 {
        let predicted_temp_diff = self.model.predict_boiler_temp_diff(
            grouphead_temp_c,
            boiler_temp_c,
            q,
        );

        if predicted_temp_diff.is_err() {
            error!(
                "Failed to predict boiler temperature difference: {:?}",
                predicted_temp_diff
            );
            return 0.0;
        }

        let predicted_temp_diff = predicted_temp_diff.unwrap();

        let heat_level = if boiler_temp_c + predicted_temp_diff > self.target_temperature {
            0.0
        } else {
            1.0
        };

        info!("Current: {}, Pred: {}, Q: {}, heat: {}", boiler_temp_c, predicted_temp_diff, q, heat_level);

        heat_level
    }

    fn update_target_temperature(&mut self, target_temperature: f32) {
        self.target_temperature = target_temperature;
    }
}

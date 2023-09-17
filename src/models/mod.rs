use std::io::Cursor;

use anyhow::Result;
use tract_core::ndarray;
use tract_onnx::prelude::*;

pub struct PredictiveModels {
    extraction_temp_model:
        SimplePlan<TypedFact, Box<dyn TypedOp>, Graph<TypedFact, Box<dyn TypedOp>>>,
    boiler_temp_diff_model:
        SimplePlan<TypedFact, Box<dyn TypedOp>, Graph<TypedFact, Box<dyn TypedOp>>>,
}

impl PredictiveModels {
    pub fn new() -> Result<PredictiveModels> {
        let mut extraction_temp_onnx = Cursor::new(
            include_bytes!(
                "../../models/extraction_temperature/output/extraction_temperature.onnx"
            )
            .to_vec()
            .clone(),
        );

        let extraction_temp_model = tract_onnx::onnx()
            .model_for_read(&mut extraction_temp_onnx)?
            .with_output_fact(0, Default::default())?
            .into_optimized()?
            .into_runnable()?;

        let mut boiler_temp_diff_onnx = Cursor::new(
            include_bytes!("../../models/predictive/output/subset_model.onnx")
                .to_vec()
                .clone(),
        );

        let boiler_temp_diff_model = tract_onnx::onnx()
            .model_for_read(&mut boiler_temp_diff_onnx)?
            .with_output_fact(0, Default::default())?
            .into_optimized()?
            .into_runnable()?;

        Ok(PredictiveModels {
            extraction_temp_model,
            boiler_temp_diff_model,
        })
    }

    pub fn predict_extraction_temperature(
        &self,
        grouphead_temp_c: f32,
        boiler_temp_c: f32,
    ) -> Result<f32> {
        let input = ndarray::arr1(&[grouphead_temp_c, boiler_temp_c])
            .into_shape([1, 2])?
            .into_tensor();

        let found = self
            .extraction_temp_model
            .run(tvec![input.into()])
            .unwrap()
            .remove(0);
        let n = found.to_scalar::<f32>()?;

        Ok(n.clone())
    }

    pub fn predict_boiler_temp_diff(&self, grouphead_temp_c: f32, boiler_temp_c: f32, q: f32) -> Result<f32> {
        let input = ndarray::arr1(&[grouphead_temp_c, boiler_temp_c, q])
            .into_shape([1, 3])?
            .into_tensor();

        let found = self
            .boiler_temp_diff_model
            .run(tvec![input.into()])
            .unwrap()
            .remove(0);
        let n = found.to_scalar::<f32>()?;

        Ok(n.clone())
    }
}

pub fn get_preheat_level(target_temp: f64, grouphead_temp: f64) -> f64 {
    let level = match target_temp {
        t if t <= 90.0 => grouphead_temp / 74.0,
        t if t <= 93.0 => grouphead_temp / 76.0,
        t if t <= 95.0 => grouphead_temp / 78.0,
        t if t <= 99.0 => grouphead_temp / 80.0,
        t if t <= 101.0 => grouphead_temp / 82.0,
        t if t <= 103.0 => grouphead_temp / 84.0,
        t if t <= 107.0 => grouphead_temp / 86.0,
        t if t <= 109.0 => grouphead_temp / 88.0,
        _ => grouphead_temp / 90.0,
    };

    level.min(1.0)
}

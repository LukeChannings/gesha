use std::{fs::OpenOptions, path::Path};

use anyhow::Result;
use log::{debug, error};
use serde::Serialize;
use sqlx::{migrate, Pool, QueryBuilder, Sqlite, SqlitePool};
use tokio::task;

pub type DBHandle = Pool<Sqlite>;

pub async fn open_or_create(path: &str) -> Result<DBHandle> {
    if !Path::new(path).is_file() {
        if let Err(err) = OpenOptions::new().write(true).create_new(true).open(path) {
            panic!("Failed to create a DB file at {path}, failed with: {err}");
        }
    }

    let pool = SqlitePool::connect(path).await?;

    migrate!().run(&pool).await?;

    Ok(pool)
}

#[derive(Serialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub struct Measurement {
    pub time: i64,
    pub target_temp_c: f32,
    pub boiler_temp_c: f32,
    pub grouphead_temp_c: f32,
    pub thermofilter_temp_c: Option<f32>,
    pub power: bool,
    pub heat_level: Option<f32>,
    pub pull: bool,
    pub steam: bool,
}

pub fn write_measurements(measurements: Vec<Measurement>, pool: &DBHandle) -> Result<()> {
    if measurements.len() > 0 {
        let pool = pool.clone();

        debug!("Writing {} measurements to the DB", measurements.len());

        task::spawn(async move {
            let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new(
                "INSERT INTO measurement (time, target_temp_c, boiler_temp_c, grouphead_temp_c, thermofilter_temp_c, power, heat_level, pull, steam) "
            );

            query_builder.push_values(measurements, |mut b, measurement| {
                b.push_bind(measurement.time)
                    .push_bind(measurement.target_temp_c)
                    .push_bind(measurement.boiler_temp_c)
                    .push_bind(measurement.grouphead_temp_c)
                    .push_bind(measurement.thermofilter_temp_c)
                    .push_bind(measurement.power)
                    .push_bind(measurement.heat_level)
                    .push_bind(measurement.pull)
                    .push_bind(measurement.steam);
            });

            let query = query_builder.build();

            if let Err(err) = query.execute(&pool).await {
                error!("Error writing measurements to the DB: {err}")
            };
        });
    }

    Ok(())
}

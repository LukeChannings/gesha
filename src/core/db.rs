use std::{fs::OpenOptions, path::Path};

use anyhow::Result;
use log::{debug, error, info};
use serde::Serialize;
use sqlx::{migrate, query_as, Pool, QueryBuilder, Sqlite, SqlitePool};
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

pub async fn read_measurements(
    pool: &DBHandle,
    from: i64,
    to: i64,
    limit: Option<i64>,
    bucket_size: Option<i64>,
) -> Result<Vec<Measurement>> {
    let limit = limit.unwrap_or(-1);

    let mut measurements: Vec<Measurement> = query_as!(
        Measurement,
        r#"
        SELECT time, power, pull, steam,
            heat_level as "heat_level: f32",
            target_temp_c as "target_temp_c: f32",
            boiler_temp_c as "boiler_temp_c: f32",
            grouphead_temp_c as "grouphead_temp_c: f32",
            thermofilter_temp_c as "thermofilter_temp_c: f32"
        FROM measurement
        WHERE time > ? AND time < ?
        ORDER BY time DESC
        LIMIT ?"#,
        from,
        to,
        limit
    )
    .fetch_all(pool)
    .await?;

    if measurements.len() == 0 {
        log::error!("There were no measurements in the range {from}-{to}");
        return Ok(measurements);
    }

    // I have found that despite 'ORDER BY time DESC' the measurements sometimes are not ordered.
    measurements.sort_by(|a, b| a.time.cmp(&b.time));

    // Compute a histogram of measurement values based on the bucket_size.
    // This is a method of reducing the values, since there's one measurement every 100ms.
    // If we only want a resolution of 1 second we can set bucket_size to `1000`.
    if let Some(bucket_size) = bucket_size {
        info!("Bucketing {from}-{to} / {bucket_size}");
        let mut bucketed_measurements: Vec<Measurement> = Vec::new();
        let mut current_bucket: Vec<Measurement> = Vec::new();
        let mut current_bucket_end_time = from + bucket_size;

        for measurement in measurements.iter() {
            if measurement.time < current_bucket_end_time {
                current_bucket.push(measurement.clone())
            } else {
                current_bucket_end_time = measurement.time + bucket_size;

                if current_bucket.len() == 0 {
                    continue;
                }

                current_bucket.sort_by(|a, b| a.boiler_temp_c.total_cmp(&b.boiler_temp_c));

                if let Some(median_measurement) = current_bucket.get(current_bucket.len() / 2) {
                    bucketed_measurements.push(median_measurement.clone());
                }

                current_bucket = vec![measurement.clone()];
            }
        }

        info!(
            "Bucketing {from}-{to} / {bucket_size} - {}",
            bucketed_measurements.len()
        );

        return Ok(bucketed_measurements);
    }

    Ok(measurements)
}

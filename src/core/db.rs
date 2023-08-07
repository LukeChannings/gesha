use std::{collections::VecDeque, fs::OpenOptions, path::Path};

use anyhow::{anyhow, Result};
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use sqlx::{migrate, query, query_as, Pool, QueryBuilder, Sqlite, SqlitePool};
use tokio::task;

#[derive(Clone)]
pub struct Db {
    handle: Pool<Sqlite>,
    measurement_write_queue: VecDeque<Measurement>,
}

impl Db {
    pub async fn new(path: &str) -> Result<Db> {
        if !Path::new(path).is_file() {
            if let Err(err) = OpenOptions::new().write(true).create_new(true).open(path) {
                panic!("Failed to create a DB file at {path}, failed with: {err}");
            }
        }

        let pool = SqlitePool::connect(path).await?;

        migrate!().run(&pool).await?;

        Ok(Db {
            handle: pool,
            measurement_write_queue: VecDeque::new(),
        })
    }

    pub async fn read_config(&self) -> Result<Vec<ConfigItem>> {
        let result = query_as!(ConfigItem, "SELECT key, value FROM config")
            .fetch_all(&self.handle)
            .await?;

        Ok(result)
    }

    pub async fn write_config(&self, config_item: &ConfigItem) -> Result<()> {
        query!(
            r#"
        INSERT INTO config VALUES (?1, ?2)
        ON CONFLICT(key) DO
        UPDATE SET value = ?2 WHERE key = ?1;
        "#,
            config_item.key,
            config_item.value
        )
        .execute(&self.handle)
        .await?;

        info!("Wrote {}={} to the DB", config_item.key, config_item.value);

        Ok(())
    }

    pub fn write_measurement_queue(&mut self, measurement: Measurement) -> Result<()> {
        self.measurement_write_queue.push_front(measurement);

        Ok(())
    }

    pub fn write_measurements(&self, measurements: Vec<Measurement>) -> Result<()> {
        if measurements.len() > 0 {
            let pool = self.handle.clone();

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
        &self,
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
        .fetch_all(&self.handle)
        .await?;

        measurements.extend(
            self.measurement_write_queue
                .clone()
                .into_iter()
                .filter(|measurement| measurement.time >= from && measurement.time <= to),
        );

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

    pub async fn write_shot(&self, start_time: i64, end_time: i64) -> Result<()> {
        let measurements = self
            .read_measurements(start_time, end_time, None, None)
            .await?;

        let measurement_count = measurements.len() as f32;

        if measurement_count == 0.0 {
            return Err(anyhow!("Could not write shot because there were no measurements between {start_time} and {end_time}"));
        }

        let (brew_temp_sum_c, grouphead_temp_sum_c) =
            measurements
                .into_iter()
                .fold((0.0, 0.0), |(brew, grouphead), measurement| {
                    (
                        brew + measurement.grouphead_temp_c,
                        grouphead + measurement.grouphead_temp_c,
                    )
                });

        let shot = Shot {
            start_time,
            end_time,
            total_time: end_time - start_time,
            brew_temp_average_c: brew_temp_sum_c / measurement_count,
            grouphead_temp_avg_c: grouphead_temp_sum_c / measurement_count,
        };

        query!(
            "INSERT INTO shot (start_time, end_time, total_time, brew_temp_average_c, grouphead_temp_avg_c) VALUES (?, ?, ?, ?, ?)",
            shot.start_time,
            shot.end_time,
            shot.total_time,
            shot.brew_temp_average_c,
            shot.grouphead_temp_avg_c,
        ).execute(&self.handle).await?;

        Ok(())
    }

    pub async fn read_shots(&self, from: i64, to: i64, limit: Option<i64>) -> Result<Vec<Shot>> {
        let limit = limit.unwrap_or(-1);

        let shots: Vec<Shot> = query_as!(
            Shot,
            r#"
            SELECT start_time, end_time, total_time, brew_temp_average_c as "brew_temp_average_c: f32", grouphead_temp_avg_c as "grouphead_temp_avg_c: f32"
            FROM shot
            WHERE start_time > ? AND start_time < ?
            ORDER BY start_time DESC
            LIMIT ?"#,
            from,
            to,
            limit
        )
        .fetch_all(&self.handle)
        .await?;

        Ok(shots)
    }

    pub fn flush_measurements(&mut self) -> Result<()> {
        let measurements: Vec<Measurement> = self
            .measurement_write_queue
            .drain(..)
            .collect::<VecDeque<_>>()
            .into();

        self.write_measurements(measurements)?;

        Ok(())
    }
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

#[derive(Serialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub struct Shot {
    pub start_time: i64,
    pub end_time: i64,
    pub total_time: i64,
    pub brew_temp_average_c: f32,
    pub grouphead_temp_avg_c: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ConfigItem {
    pub key: String,
    pub value: String,
}

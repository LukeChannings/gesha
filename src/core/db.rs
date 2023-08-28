use std::{
    collections::{HashMap, VecDeque},
    fs::OpenOptions,
    path::Path,
    sync::Arc,
    time::Duration,
};

use anyhow::{anyhow, Result};
use log::{error, info};
use serde::{Deserialize, Serialize};
use sqlx::{migrate, query, query_as, Pool, QueryBuilder, Sqlite, SqlitePool};
use tokio::{select, sync::RwLock, time};
use tokio_util::sync::CancellationToken;

use super::mqtt::Range;

pub struct Db {
    handle: Pool<Sqlite>,
    measurement_write_queue: Arc<RwLock<VecDeque<Measurement>>>,
    measurement_interval_cancel: CancellationToken,
    measurement_interval_handle: Option<tokio::task::JoinHandle<()>>,
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
            measurement_write_queue: Arc::new(RwLock::new(VecDeque::new())),
            measurement_interval_cancel: CancellationToken::new(),
            measurement_interval_handle: None,
        })
    }

    pub async fn read_config(&self) -> Result<HashMap<String, String>> {
        let results = query_as!(ConfigItem, "SELECT key, value FROM config")
            .fetch_all(&self.handle)
            .await?;

        let mut configs = HashMap::new();

        for result in results {
            configs.insert(result.key, result.value);
        }

        Ok(configs)
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

    pub async fn write_measurement_queue(&mut self, measurement: Measurement) -> Result<()> {
        let mut queue = self.measurement_write_queue.write().await;
        queue.push_back(measurement);

        Ok(())
    }

    pub fn start_measurement_writer_interval(&mut self, duration: Duration) {
        let queue = self.measurement_write_queue.clone();
        let handle = self.handle.clone();

        let abort = self.measurement_interval_cancel.clone();

        self.measurement_interval_handle = Some(tokio::spawn(async move {
            let mut interval = time::interval(duration);
            interval.set_missed_tick_behavior(time::MissedTickBehavior::Skip);

            async fn drain_measurements(
                handle: &Pool<Sqlite>,
                queue: &Arc<RwLock<VecDeque<Measurement>>>,
            ) {
                let mut measurement_queue = queue.write().await;

                let measurements: Vec<Measurement> =
                    measurement_queue.drain(..).collect::<VecDeque<_>>().into();

                if measurements.len() > 0 {
                    if let Err(err) = Db::write_measurements(handle, measurements).await {
                        error!("Failed to write measurements: {}", err);
                    }
                } else {
                    info!("No measurements to write");
                }
            }

            loop {
                select! {
                    _ = interval.tick() => {
                        drain_measurements(&handle, &queue).await;
                    },
                    _ = abort.cancelled() => {
                        info!("Stopping measurement writer interval");
                        drain_measurements(&handle, &queue).await;
                        break;
                    }
                }
            }
        }));
    }

    pub async fn stop_measurement_writer_interval(&mut self) -> Result<()> {
        if let Some(handle) = self.measurement_interval_handle.take() {
            self.measurement_interval_cancel.cancel();
            handle.await?;
        }

        Ok(())
    }

    pub async fn write_measurements(
        pool: &Pool<Sqlite>,
        measurements: Vec<Measurement>,
    ) -> Result<()> {
        info!("Writing {} measurements to the DB", measurements.len());

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

        if let Err(err) = query.execute(pool).await {
            return Err(anyhow!("Error writing measurements to the DB: {err}"));
        }

        Ok(())
    }

    pub async fn read_measurements(&self, range: &Range) -> Result<Vec<Measurement>> {
        let Range {
            from,
            to,
            bucket_size,
            ..
        } = range;

        let limit = range.limit.unwrap_or(-1);

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
            limit,
        )
        .fetch_all(&self.handle)
        .await?;

        let current_measurements = self
            .measurement_write_queue
            .read()
            .await
            .clone()
            .into_iter()
            .filter(|measurement| measurement.time >= *from && measurement.time <= *to);

        measurements.extend(current_measurements);

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
        let range = Range {
            id: "".to_string(),
            from: start_time,
            to: end_time,
            bucket_size: None,
            limit: None,
        };

        let measurements = self.read_measurements(&range).await?;

        let measurement_count = measurements.len() as f32;

        if measurement_count == 0.0 {
            return Err(anyhow!("Could not write shot because there were no measurements between {start_time} and {end_time}"));
        }

        let (brew_temp_sum_c, grouphead_temp_sum_c) =
            measurements
                .into_iter()
                .fold((0.0, 0.0), |(boiler, grouphead), measurement| {
                    (
                        boiler + measurement.boiler_temp_c,
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

    pub async fn read_shots(&self, range: &Range) -> Result<Vec<Shot>> {
        let limit = range.limit.unwrap_or(-1);

        let shots: Vec<Shot> = query_as!(
            Shot,
            r#"
            SELECT start_time, end_time, total_time, brew_temp_average_c as "brew_temp_average_c: f32", grouphead_temp_avg_c as "grouphead_temp_avg_c: f32"
            FROM shot
            WHERE start_time > ? AND start_time < ?
            ORDER BY start_time DESC
            LIMIT ?"#,
            range.from,
            range.to,
            limit
        )
        .fetch_all(&self.handle)
        .await?;

        Ok(shots)
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

pub const DB_KEY_TARGET_TEMPERATURE: &str = "TargetTemperature";
pub const DB_KEY_CONTROL_METHOD: &str = "ControlMethod";

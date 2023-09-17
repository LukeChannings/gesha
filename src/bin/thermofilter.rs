// This is the software for the standalone thermofilter.
// It outputs the temperature to stdout and also publishes it to MQTT.
use anyhow::Result;
use clap::Parser;
use gesha::core::{config::Spi, mqtt::ValueChange, state::Mode, thermocouple::Thermocouple, util};
use rumqttc::v5::{mqttbytes::v5::Packet, AsyncClient, Event as MqttEvent, MqttOptions};
use serde_json;
use std::path::Path;
use std::str;
use std::time::{Duration, SystemTime};
use tokio::{
    io::{self, AsyncWriteExt},
    select, task,
};

#[derive(Parser, Debug, Clone)]
struct Args {
    #[arg(short, long)]
    pub interface: Spi,

    #[arg(short, long, default_value_t = 1000)]
    pub period: u64,
}

struct Brew {
    start_time: i64,
    end_time: i64,
}

enum Event {
    Brew(Brew),
    ClearMeasurements(),
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let mut thermocouple: Thermocouple = args.interface.try_into()?;

    let mut interval = tokio::time::interval(Duration::from_millis(args.period));

    let options = MqttOptions::parse_url(
        "mqtt://luke:5s9zcBneIiIgETZ0FXLKw0frf6GrjrukPIZdYbQc@silvia.iot?client_id=gesha-aux",
    )?;

    let (client, mut event_loop) = AsyncClient::new(options, 10);

    let (tx, mut rx) = tokio::sync::mpsc::channel::<Event>(10);

    let mqtt_listen = task::spawn(async move {
        let mut brew_start_time: Option<i64> = None;

        let mut clear_interval = tokio::time::interval(Duration::from_secs(60 * 10));

        loop {
            select! {
                Ok(notification) = event_loop.poll() => {
                    if let MqttEvent::Incoming(Packet::Publish(publish_event)) = notification {
                        let topic = str::from_utf8(&publish_event.topic).unwrap();
                        match topic {
                            "gesha/mode" => {
                                let mode: Mode = serde_yaml::from_slice(&publish_event.payload).unwrap();
                                match mode {
                                    Mode::Brew => {
                                        brew_start_time = Some(util::get_unix_timestamp(SystemTime::now()).unwrap());

                                    },
                                    Mode::Active => {
                                        if let Some(start_time) = brew_start_time {

                                            tx.send(Event::Brew(Brew  {
                                                start_time,
                                                end_time: util::get_unix_timestamp(SystemTime::now()).unwrap(),
                                            })).await.unwrap();


                                            brew_start_time = None;
                                        }
                                    },
                                    _ => { },
                                }
                            },
                            _ => {}
                        }
                    }
                },
                _ = clear_interval.tick() => {
                    // Clear measurements every 10 minutes
                    if brew_start_time.is_none() {
                        tx.send(Event::ClearMeasurements()).await.unwrap();
                    }
                },
                _ = tokio::signal::ctrl_c() => {
                    break
                }
            }
        }
    });

    client
        .subscribe("gesha/mode", rumqttc::v5::mqttbytes::QoS::AtLeastOnce)
        .await?;

    let mut measurements: Vec<ValueChange> = Vec::new();
    let mut current_temp: Option<f32> = None;

    loop {
        select! {
            _ = interval.tick() => {
                let temp: f32 = thermocouple.read()?;

                let is_changed = match current_temp {
                    Some(current) => current != temp,
                    None => true,
                };

                if is_changed {
                    let time = util::get_unix_timestamp(SystemTime::now())?;

                    let change = ValueChange {
                        timestamp: time,
                        value: temp,
                    };

                    measurements.push(change.clone());

                    println!("{},{}", time, temp);

                    client
                        .publish(
                            "gesha/temperature/thermofilter",
                            rumqttc::v5::mqttbytes::QoS::AtLeastOnce,
                            true,
                            serde_json::to_string(&change)?,
                        )
                        .await?;
                }

                current_temp = Some(temp);
            },
            Some(event) = rx.recv() => {
                match event {
                    Event::Brew(brew) => {
                        write_measurements(&measurements, brew).await?;
                        measurements.clear();
                    }
                    Event::ClearMeasurements() => {
                        measurements.clear();
                    }
                }
            },
            _ = tokio::signal::ctrl_c() => {
                mqtt_listen.abort();
                break
            }
        }
    }

    Ok(())
}

async fn write_measurements(measurements: &Vec<ValueChange>, brew: Brew) -> io::Result<()> {

    let start_time = brew.start_time;
    let end_time = brew.end_time;

    let path = format!("brew-{}-{}.json", start_time, end_time);
    let path = Path::new(&path);

    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .await?;

    let buffer = 5_000;

    let brew_measurements = measurements
        .iter()
        .filter(|m| m.timestamp >= (start_time - buffer) && m.timestamp <= (end_time + buffer))
        .collect::<Vec<&ValueChange>>();

    let json = serde_json::to_string(&brew_measurements)?;

    file.write_all(json.as_bytes()).await?;


    Ok(())
}

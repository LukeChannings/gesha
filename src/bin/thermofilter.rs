// This is the software for the standalone thermofilter.
// It outputs the temperature to stdout and also publishes it to MQTT.
use std::time::{Duration, SystemTime};

use anyhow::Result;
use clap::Parser;
use gesha::core::{config::Spi, thermocouple::Thermocouple, util, mqtt::ValueChange};
use rumqttc::v5::{AsyncClient, MqttOptions};
use tokio::{select, task};
use serde_json;

#[derive(Parser, Debug, Clone)]
struct Args {
    #[arg(short, long)]
    pub interface: Spi,

    #[arg(short, long, default_value_t = 1000)]
    pub period: u64,
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

    let mqtt_listen = task::spawn(async move {
        loop {
            let _ = event_loop.poll().await;
        }
    });

    let mut current_temp: Option<f32> = None;

    loop {
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

            client
                .publish(
                    "gesha/temperature/thermofilter",
                    rumqttc::v5::mqttbytes::QoS::AtLeastOnce,
                    true,
                    serde_json::to_string(&change)?,
                )
                .await?;

            println!("{time},{temp}");
        }

        current_temp = Some(temp);

        select! {
            _ = interval.tick() => { },
            _ = tokio::signal::ctrl_c() => {
                mqtt_listen.abort();
                break
            }
        }
    }

    Ok(())
}

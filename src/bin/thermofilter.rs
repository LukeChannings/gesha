use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::Result;
use clap::Parser;
use gesha::core::{config::Spi, thermocouple::Thermocouple};
use rumqttc::v5::{AsyncClient, MqttOptions};
use tokio::{select, task};

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

    loop {
        let temp = thermocouple.read()?;
        let time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as u64;

        client
            .publish(
                "gesha/temperature/thermofilter",
                rumqttc::v5::mqttbytes::QoS::AtLeastOnce,
                true,
                temp.to_string(),
            )
            .await?;

        println!("{time}, {temp}");

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

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::Result;
use clap::Parser;
use gesha::core::{config::Spi, thermocouple::Thermocouple};
use tokio::select;

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

    loop {
        let temp = thermocouple.read()?;
        let time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as u64;

        println!("{time}, {temp}");

        select! {
            _ = interval.tick() => { },
            _ = tokio::signal::ctrl_c() => {
                break
            }
        }
    }

    Ok(())
}

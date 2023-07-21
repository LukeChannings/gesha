use anyhow::{anyhow, Result};
use clap::{builder::PossibleValue, ValueEnum};
use log::error;
use serde::{Deserialize, Serialize};
use std::io::{self, ErrorKind};

const CONFIG_NAMES: [&str; 2] = ["gesha.config.yaml", "gesha.config.yml"];

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub boiler_spi: Option<Spi>,
    pub grouphead_spi: Option<Spi>,
    pub thermofilter_spi: Option<Spi>,
    pub mqtt_url: Option<String>,
    pub boiler_pin: u8,
}

impl Config {
    pub async fn load(config_path: Option<String>) -> Result<Config> {
        let config_paths: Vec<&str> = if let Some(config_path) = config_path.as_ref() {
            vec![config_path]
        } else {
            CONFIG_NAMES.to_vec()
        };

        for config_path in config_paths.iter() {
            let config = std::fs::read_to_string(config_path)?;
            let config: io::Result<Config> =
                serde_yaml::from_str(&config).map_err(|err| io::Error::new(ErrorKind::Other, err));

            if let Err(error) = config {
                error!("Failed to load {:?}, error: {:?}", config_path, error);
            } else {
                return Ok(config.unwrap());
            }
        }

        Err(anyhow!(
            "No config files could be loaded. Tried: {:?}",
            config_paths
        ))
    }
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub enum Spi {
    Rpi0,
    Rpi0_1,
    Rpi1,
    Rpi1_1,
    Rpi1_2,
}

impl ValueEnum for Spi {
    fn value_variants<'a>() -> &'a [Self] {
        &[
            Self::Rpi0,
            Self::Rpi0_1,
            Self::Rpi1,
            Self::Rpi1_1,
            Self::Rpi1_2,
        ]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        Some(match self {
            Self::Rpi0 => PossibleValue::new("Rpi0"),
            Self::Rpi0_1 => PossibleValue::new("Rpi0_1"),
            Self::Rpi1 => PossibleValue::new("Rpi1"),
            Self::Rpi1_1 => PossibleValue::new("Rpi1_1"),
            Self::Rpi1_2 => PossibleValue::new("Rpi1_2"),
        })
    }
}

use anyhow::Result;
use clap::{builder::PossibleValue, Parser, ValueEnum};
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use std::{
    io::{self, ErrorKind},
    path::Path,
};

const CONFIG_NAMES: [&str; 2] = ["gesha.config.yaml", "gesha.config.yml"];

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub control_method: ControlMethod,
    pub boiler_spi: Option<Spi>,
    pub grouphead_spi: Option<Spi>,
    pub thermofilter_spi: Option<Spi>,
    pub mqtt_url: Option<String>,
    pub boiler_pin: u8,
}

#[derive(Parser, Debug, Clone)]
struct Args {
    #[arg(short, long)]
    pub config_path: Option<String>,

    #[arg(short, long)]
    pub control_method: Option<ControlMethod>,

    #[arg(short, long)]
    pub boiler_spi: Option<Spi>,

    #[arg(short, long)]
    pub grouphead_spi: Option<Spi>,

    #[arg(short, long)]
    pub thermofilter_spi: Option<Spi>,

    #[arg(short, long)]
    pub mqtt_url: Option<String>,

    #[arg(short, long)]
    pub boiler_pin: Option<u8>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            control_method: ControlMethod::Manual,
            boiler_spi: None,
            grouphead_spi: None,
            thermofilter_spi: None,
            mqtt_url: None,
            boiler_pin: 26,
        }
    }
}

impl Config {
    pub async fn load() -> Result<Config> {
        let args = Args::parse();
        let mut config: Config = Self::default();

        let config_paths: Vec<&str> = if let Some(config_path) = &args.config_path {
            vec![config_path]
        } else {
            CONFIG_NAMES.to_vec()
        };

        for name in config_paths.iter() {
            match Self::from_file(Path::new(name)) {
                Ok(config_from_file) => {
                    info!("Loaded {name}");
                    config = config_from_file;
                }
                Err(err) => {
                    if err.kind() != ErrorKind::NotFound {
                        error!("Failed to load {name}. {err}");
                    }
                }
            }
        }

        config.update_with_args(args);

        Ok(config)
    }

    pub fn from_file(path: &Path) -> io::Result<Config> {
        debug!("Trying to load {:?}", path.as_os_str());
        let config = &std::fs::read_to_string(path)?;

        debug!("{}", config);

        serde_yaml::from_str(config).map_err(|err| io::Error::new(ErrorKind::Other, err))
    }

    fn update_with_args(&mut self, args: Args) {
        if let Some(control_method) = args.control_method {
            self.control_method = control_method;
        }
        if let Some(interface_boiler) = args.boiler_spi {
            self.boiler_spi = Some(interface_boiler)
        }
        if let Some(interface_grouphead) = args.grouphead_spi {
            self.grouphead_spi = Some(interface_grouphead)
        }
        if let Some(interface_thermofilter) = args.thermofilter_spi {
            self.thermofilter_spi = Some(interface_thermofilter)
        }
        if let Some(mqtt_url) = args.mqtt_url {
            self.mqtt_url = Some(mqtt_url)
        }
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

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum ControlMethod {
    // If the current temperature is < threshold, turn heat on, otherwise off.
    #[serde(alias = "threshold", alias = "THRESHOLD")]
    Threshold,

    // https://en.wikipedia.org/wiki/PID_controller
    #[serde(alias = "pid", alias = "Pid")]
    PID,

    // https://en.wikipedia.org/wiki/Model_predictive_control
    #[serde(alias = "mpc", alias = "Mpc")]
    MPC,

    // Allow manual control
    #[serde(alias = "manual", alias = "MANUAL")]
    Manual,
}

impl ValueEnum for ControlMethod {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::Threshold, Self::PID, Self::MPC, Self::Manual]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        Some(match self {
            Self::Threshold => PossibleValue::new("Threshold"),
            Self::PID => PossibleValue::new("PID"),
            Self::MPC => PossibleValue::new("MPC"),
            Self::Manual => PossibleValue::new("Manual"),
        })
    }
}

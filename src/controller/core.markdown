```rust
use std::time::{self, SystemTime, UNIX_EPOCH};

use crate::config::{Config, Spi};
use anyhow::{anyhow, Result};
use max31855::{FullResult, Max31855};
use rppal::{gpio, spi};
use tokio::time as tokio_time;
use tokio_util::sync::CancellationToken;

pub struct Controller {
    pub config: Config,
    temp_poll_interval: time::Duration,
    dry_run: bool,
}

impl Controller {
    pub fn new(config: Config, temp_poll_interval: time::Duration, dry_run: bool) -> Controller {
        Controller {
            config,
            temp_poll_interval,
            dry_run,
        }
    }

    fn create_thermocouple_reader(
        &self,
        thermocouple: &Option<Spi>,
    ) -> Result<Box<dyn FnMut() -> Result<FullResult, anyhow::Error>>, anyhow::Error> {
        if let Some(thermocouple) = thermocouple {
            let (mut thermocouple_spi, mut thermocouple_gpio) =
                thermocouple.to_interface().unwrap();

            Ok(Box::new(move || {
                let boiler_temp = thermocouple_spi
                    .read_all(&mut thermocouple_gpio, max31855::Unit::Celsius)
                    .map_err(
                        |err: max31855::Error<spi::Error, std::convert::Infallible>| {
                            anyhow!("Could not read temperature. {:?}", err)
                        },
                    )?;
                Ok(boiler_temp)
            }))
        } else {
            Err(anyhow!("No thermocouple set"))
        }
    }

    pub async fn start(&mut self) -> Result<()> {
        let controller = self
            .config
            .control_method
            .as_mut()
            .map(|control_method| control_method.into_controller());

        if let Some(controller) = controller {
            let mut interval = tokio_time::interval(self.temp_poll_interval);
            interval.set_missed_tick_behavior(tokio_time::MissedTickBehavior::Skip);

            // THIS WILL PANIC IF create_thermocouple_reader CANNOT GET THE INTERFACE
            // UPDATE SO THAT THE FUNCTION IS OPTIONAL
            let mut get_boiler_temp = self.create_thermocouple_reader(&self.config.boiler)?;
            let mut get_grouphead_temp = self.create_thermocouple_reader(&self.config.grouphead)?;
            let mut get_thermofilter_temp =
                self.create_thermocouple_reader(&self.config.thermofilter)?;

            let mut heat = gpio::Gpio::new()?.get(26)?.into_output();

            println!(
                "{:},{:},{:},{:},{:},{:},{:},{:},{:}",
                "time",
                "boiler_thermocouple",
                "boiler_internal",
                "grouphead_thermocouple",
                "grouphead_internal",
                "basket_thermocouple",
                "basket_internal",
                "is_heating",
                "controller",
            );

            let token = CancellationToken::new();
            let cloned_token = token.clone();

            ctrlc::set_handler(move || cloned_token.cancel())?;

            loop {
                if token.is_cancelled() {
                    heat.set_low();
                    break;
                } else {
                    interval.tick().await;
                }

                let now = format!("{:?}", SystemTime::now().duration_since(UNIX_EPOCH)?);

                let boiler_temp = get_boiler_temp().map_or_else(
                    |_| FullResult {
                        thermocouple: -1000.0,
                        internal: -1000.0,
                        unit: max31855::Unit::Celsius,
                    },
                    |result| result,
                );
                let grouphead_temp = get_grouphead_temp().map_or_else(
                    |_| FullResult {
                        thermocouple: -1000.0,
                        internal: -1000.0,
                        unit: max31855::Unit::Celsius,
                    },
                    |result| result,
                );
                let thermofilter_temp = get_thermofilter_temp().map_or_else(
                    |_| FullResult {
                        thermocouple: -1000.0,
                        internal: -1000.0,
                        unit: max31855::Unit::Celsius,
                    },
                    |result| result,
                );

                if boiler_temp.internal == -1000.0
                    || grouphead_temp.internal == -1000.0
                    || thermofilter_temp.internal == -1000.0
                {
                    continue;
                }

                let boiler_heat =
                    controller.sample(boiler_temp.thermocouple, grouphead_temp.thermocouple);

                if !self.dry_run {
                    if boiler_heat == true {
                        heat.set_high();
                    } else {
                        heat.set_low();
                    }
                }

                println!(
                    "{:},{:},{:},{:},{:},{:},{:},{:},{:}",
                    now,
                    boiler_temp.thermocouple,
                    boiler_temp.internal,
                    grouphead_temp.thermocouple,
                    grouphead_temp.internal,
                    thermofilter_temp.thermocouple,
                    thermofilter_temp.internal,
                    boiler_heat,
                    self.config.control_method.as_ref().unwrap()
                );
            }
        }

        Ok(())
    }

    pub fn stop() {}
}

impl Drop for Controller {
    fn drop(&mut self) {
        Controller::stop();
    }
}
```

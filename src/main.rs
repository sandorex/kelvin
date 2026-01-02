mod cli;
mod config;

pub mod prelude {
    pub use anyhow::{Context, Result, anyhow, bail};
}

use clap::Parser;
use prelude::*;
use serde_json::Value as JsonValue;
use crate::config::Config;
use std::fmt::Write as _;

fn get_temps() -> Result<JsonValue> {
    let output = std::process::Command::new("sensors")
        .args(["-j", "--config", "/dev/null"])
        .output()
        .with_context(|| anyhow!("Unable to run sensors command"))?;

    let stdout = String::from_utf8(output.stdout)?;

    serde_json::from_str(&stdout)
        .with_context(|| anyhow!("Unable to parse json from sensors"))
}

fn gen_format(args: &cli::Cli, config: &Config, temps: &JsonValue) -> Result<String> {
    let timestamp = chrono::Local::now().format("%H:%M:%S").to_string();
    let mut output = String::new();

    if args.no_format || config.format.is_none() {
        // simply list all the sensors
        for sensor in &config.sensors {
            let value = sensor.get_value(&temps)?;
            writeln!(&mut output, "{}{}{}", sensor.prefix(), sensor.format_value(value), sensor.suffix())?;
        }
    } else {
        output = config.format.clone().expect("config.format is None");

        // replace each sensor with rust-like format! syntax {sensor_name}
        for sensor in &config.sensors {
            let sensor_pattern = format!("{{{}}}", sensor.name);

            // ignore sensors that are not inside so they dont need to be calculated
            if !output.contains(&sensor_pattern) {
                continue;
            }

            let value = sensor.get_value(&temps)?;
            output = output.replace(&sensor_pattern, &sensor.format_value(value));
        }

        if output.contains("{timestamp}") {
            output = output.replace("{timestamp}", &timestamp);
        }
    }

    Ok(output.trim().to_string())
}

const CLEAR_SEQ: &str = "\x1b[H\x1b[2J";

// TODO warn user of any panic or crash!
fn main() -> Result<()> {
    let args = cli::Cli::parse();

    let config = if let Some(path) = &args.config {
        Config::read_from_file(&path)?
    } else {
        Config::read_config()?
    };

    if args.kill {
        todo!();
    }

    // TODO daemon mode
    if args.daemon {
        todo!();
    }

    // TODO alarms

    if args.once {
        // no screen clearing for the single run
        let temps = get_temps()?;
        let output = gen_format(&args, &config, &temps)?;
        println!("{output}");
    } else {
        loop {
            let temps = get_temps()?;
            let output = gen_format(&args, &config, &temps)?;
            println!("{CLEAR_SEQ}{output}");

            std::thread::sleep(std::time::Duration::from_millis(config.poll_rate as u64))
        }
    }

    Ok(())
}

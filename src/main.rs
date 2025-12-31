mod cli;
mod config;

pub mod prelude {
    pub use anyhow::{Context, Result, anyhow, bail};
}

use clap::Parser;
use prelude::*;
use serde_json::Value as JsonValue;

use crate::config::Config;

fn get_temps() -> Result<JsonValue> {
    let output = std::process::Command::new("sensors")
        .args(["-j", "--config", "/dev/null"])
        .output()
        .with_context(|| anyhow!("Unable to run sensors command"))?;

    let stdout = String::from_utf8(output.stdout)?;

    serde_json::from_str(&stdout)
        .with_context(|| anyhow!("Unable to parse json from sensors"))
}

// TODO warn user of any panic or crash!
fn main() -> Result<()> {
    let args = cli::Cli::parse();
    dbg!(&args);

    let config = if let Some(path) = &args.config {
        Config::read_from_file(&path)?
    } else {
        Config::read_config()?
    };

    let temps = get_temps()?;

    for sensor in config.sensors {
        let value = sensor.get_value(&temps)?;

        println!("{}{}{}", sensor.prefix(), sensor.format_value(value), sensor.suffix());
    }

    Ok(())
}

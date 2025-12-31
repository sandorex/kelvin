mod cli;
mod config;

pub mod prelude {
    pub use anyhow::{Context, Result, anyhow, bail};
}

use std::path::{Path, PathBuf};
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

    // let something = get_by_path(&obj, Path::new("/k10temp-pci-00c3/Tctl/temp1_input"));
    // dbg!(something);
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

        println!("{}: {} {}",
            sensor.label.as_ref().unwrap_or(&sensor.name),
            sensor.format_value(value),
            sensor.unit.unwrap_or("".to_string())
        );
    }


    Ok(())
}

mod cli;
mod config;

pub mod prelude {
    pub use anyhow::{Context, Result, anyhow, bail};
}

use std::path::{Path, PathBuf};
use clap::Parser;
use prelude::*;
use serde_json::Value as JsonValue;

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

fn main() -> Result<()> {
    let args = cli::Cli::parse();
    dbg!(&args);

    // TODO deserialize config

    let temps = get_temps()?;

    let sensor = config::Sensor {
        min: 0.0,
        max: 3500.0,
        map: Some(config::SensorMap {
            min: 0.0,
            max: 100.0,
        }),
        round: Some(0),
        path: PathBuf::new().join("amdgpu-pci-0300").join("fan1").join("fan1_input"),
        ..Default::default()
    };

    dbg!(sensor.get_value(&temps));

    Ok(())
}

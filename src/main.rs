mod cli;
mod config;

pub mod prelude {
    pub use anyhow::{Context, Result, anyhow, bail};
}

use clap::Parser;
use prelude::*;
use serde_json::Value as JsonValue;
use crate::config::Config;
use std::{cell::OnceCell, fmt::Write as _, io::{BufRead, BufReader}};

fn get_temps() -> Result<JsonValue> {
    let output = std::process::Command::new("sensors")
        .args(["-j", "--config", "/dev/null"])
        .output()
        .with_context(|| anyhow!("Unable to run sensors command"))?;

    let stdout = String::from_utf8(output.stdout)?;

    serde_json::from_str(&stdout)
        .with_context(|| anyhow!("Unable to parse json from sensors"))
}

// TODO this function is getting bloated, rework it so all sensors etc are just keys in a hashmap
fn gen_format(args: &cli::Cli, config: &Config, temps: &JsonValue, cpu_usage: &str) -> Result<String> {
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

        if output.contains("{cpu_usage}") {
            output = output.replace("{cpu_usage}", cpu_usage);
        }
    }

    Ok(output.trim().to_string())
}

const CLEAR_SEQ: &str = "\x1b[H\x1b[2J";

#[derive(Debug)]
struct CPUUsage {
    last_idle: u64,
    last_sum: u64,
    cpu_count: OnceCell<u8>,
}

#[allow(dead_code)]
impl CPUUsage {
    fn get_cpu_count() -> u8 {
        let output = std::process::Command::new("nproc")
            .output()
            .expect("Could not run nproc");

        if !output.status.success() {
            panic!("Error running nproc");
        }

        String::from_utf8(output.stdout)
            .unwrap()
            .trim()
            .parse::<u8>()
            .expect("Could not parse nproc output")
    }

    /// This gets current cpu usage since boot, used to calculate actual cpu usage
    fn get_cpu_usage() -> Result<Vec<u64>> {
        let mut buffer = String::with_capacity(128);

        {
            let file = std::fs::File::open("/proc/stat")
                .with_context(|| anyhow!("Could not open /proc/stat"))?;

            let mut reader = BufReader::new(file);
            reader.read_line(&mut buffer)
                .with_context(|| anyhow!("Could not read line from /proc/stat"))?;
        }

        buffer
            .trim()
            .split(" ")
            .skip(1) // remove "cpu"
            .skip_while(|x| x.is_empty()) // remove empty split cause of extra space
            .map(|x|
                x.parse::<u64>()
                    .with_context(|| anyhow!("Unable to parse {x:?} in /proc/stat"))
            )
            .collect::<Result<Vec<_>>>()
    }

    pub fn new() -> Self {
        Self {
            last_idle: 0,
            last_sum: 0,
            cpu_count: OnceCell::new(),
        }
    }

    pub fn refresh(&mut self) -> Result<()> {
        let usage = Self::get_cpu_usage()?;

        self.last_sum = usage.iter().sum();
        self.last_idle = usage[4];

        Ok(())
    }

    pub fn get(&mut self) -> Result<f64> {
        let curr = Self::get_cpu_usage()?;
        let idle = curr[3];
        let sum = curr.iter().sum();

        let delta_total = sum - self.last_sum;
        let delta_idle = idle - self.last_idle;
        let usage = (1000 * (delta_total - delta_idle + 5)) / 10;

        self.last_idle = idle;
        self.last_sum = sum;

        // cpu count wont change so initialize it once
        let cpu_count = *self.cpu_count.get_or_init(|| Self::get_cpu_count());

        // clamp to 0-100
        Ok(((usage as f64 / cpu_count as f64) * 0.01).clamp(0.0, 100.0))
    }
}

// TODO warn user of any panic or crash!
fn main() -> Result<()> {
    let args = cli::Cli::parse();

    let config = if let Some(path) = &args.config {
        Config::read_from_file(&path)?
    } else {
        Config::read_config()?
    };

    if config.poll_rate < 500 {
        bail!("Poll rate must be at least 500ms");
    }

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
        let output = gen_format(&args, &config, &temps, "?")?;
        println!("{output}");
    } else {
        use std::thread::sleep;
        use std::time::Duration;

        let mut cpu_usage_struct = CPUUsage::new();

        loop {
            let cpu_usage = format!("{:.1}", cpu_usage_struct.get()?);

            let temps = get_temps()?;
            let output = gen_format(&args, &config, &temps, &cpu_usage)?;
            println!("{CLEAR_SEQ}{output}");

            // if config.poll_rate > 1_000 {
                // sleep for most of the duration
                // sleep(Duration::from_millis((config.poll_rate - 1_000) as u64));

                // get current cpu usage second before actual poll rate
                // last_cpu = get_cpu_usage()?;
                // last_cpu_sum = last_cpu.iter().sum();
                sleep(Duration::from_millis(1000));
            // } else {
            //     todo!();
            //     // sleep(Duration::from_millis(config.poll_rate as u64))
            // }
        }
    }

    Ok(())
}

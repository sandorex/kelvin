mod cli;
mod config;

pub mod prelude {
    pub use anyhow::{Context as AnyhowContext, Result, anyhow, bail};
}

use clap::Parser;
use prelude::*;
use serde_json::Value as JsonValue;
use crate::config::{Config, Sensor};
use std::{cell::OnceCell, collections::HashMap, io::{BufRead, BufReader}};

fn get_temps() -> Result<JsonValue> {
    let output = std::process::Command::new("sensors")
        .args(["-j", "--config", "/dev/null"])
        .output()
        .with_context(|| anyhow!("Unable to run sensors command"))?;

    let stdout = String::from_utf8(output.stdout)?;

    serde_json::from_str(&stdout)
        .with_context(|| anyhow!("Unable to parse json from sensors"))
}


const CLEAR_SEQ: &str = "\x1b[H\x1b[2J";

#[derive(Debug)]
struct Context {
    args: cli::Cli,
    config: Config,
    sensors_data: JsonValue,
}

trait Widget {
    fn value(&mut self, ctx: &Context) -> Result<String>;

    fn update(&mut self, _ctx: &Context) -> Result<()> {
        // update is not required for all widgets
        Ok(())
    }
}

#[derive(Debug)]
struct CPUUsageWidget {
    last_idle: u64,
    last_sum: u64,
    cpu_count: OnceCell<u8>,
}

#[allow(dead_code)]
impl CPUUsageWidget {
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

        self.last_idle = usage[3];
        self.last_sum = usage.iter().sum();

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

impl Widget for CPUUsageWidget {
    fn value(&mut self, _ctx: &Context) -> Result<String> {
        // limit the decimals to 1
        self.get().map(|x| format!("{x:.1}"))
    }

    fn update(&mut self, _ctx: &Context) -> Result<()> {
        self.refresh()
    }
}

#[derive(Debug)]
struct SensorWidget {
    sensor: Sensor,
}

impl Widget for SensorWidget {
    fn value(&mut self, ctx: &Context) -> Result<String> {
        Ok(self.sensor.format_value(self.sensor.get_value(&ctx.sensors_data)?))
    }
}

#[derive(Debug)]
struct TimeWidget;

impl Widget for TimeWidget {
    fn value(&mut self, _ctx: &Context) -> Result<String> {
        Ok(chrono::Local::now().format("%H:%M:%S").to_string())
    }
}

fn format_var(var: &str) -> String {
    // very simple "{var}" formatter
    format!("{{{var}}}")
}

#[derive(Debug)]
struct DummyWidget(String);

impl Widget for DummyWidget {
    fn value(&mut self, _ctx: &Context) -> Result<String> {
        Ok(self.0.clone())
    }
}

const MINIMAL_POLL_RATE: u16 = 1000;

// TODO warn user of any panic or crash!
fn main() -> Result<()> {
    let args = cli::Cli::parse();

    let config = if let Some(path) = &args.config {
        Config::read_from_file(&path)?
    } else {
        Config::read_config()?
    };

    if config.poll_rate < MINIMAL_POLL_RATE {
        bail!("Poll rate must be at least {}ms", MINIMAL_POLL_RATE);
    }

    if args.kill {
        todo!();
    }

    // TODO daemon mode
    if args.daemon {
        todo!();
    }

    // struct to hold all the data that widgets have access to
    let mut ctx = Context {
        args,
        config,
        sensors_data: get_temps()?,
    };

    // TODO if no format just list all in order
    if ctx.args.no_format || ctx.config.format.is_none() {
        todo!();
    }

    // TODO alarms

    let mut widgets: HashMap<String, Box<dyn Widget>> = HashMap::new();

    // only create widgets that are actually used
    {
        let format = ctx.config.format.as_ref().unwrap();

        // filtering out sensors that are not used
        //
        // NOTE: i am taking the sensors vector to simplify the ownership
        for sensor in std::mem::take(&mut ctx.config.sensors) {
            let var = format_var(&sensor.name);
            if format.contains(&var) {
                widgets.insert(var, Box::new(SensorWidget { sensor: sensor }));
            }
        }

        let var = format_var("time");
        if format.contains(&var) {
            widgets.insert(var, Box::new(TimeWidget));
        }

        let var = format_var("cpu_usage");
        if format.contains(&var) {
            if ctx.args.once {
                // cpu usage cannot be calculated quickly
                widgets.insert(var, Box::new(DummyWidget("??".to_string())));
            } else {
                widgets.insert(var, Box::new(CPUUsageWidget::new()));
            }
        }
    }

    fn update_format(ctx: &Context, format: &mut String, widgets: &mut HashMap<String, Box<dyn Widget>>) -> Result<()> {
        // replace all instances
        for (var, widget) in widgets.iter_mut() {
            *format = format.replace(var, &widget.value(&ctx)?);
        }

        Ok(())
    }

    let mut format = ctx.config.format.as_ref().unwrap().clone();

    if ctx.args.once {
        update_format(&ctx, &mut format, &mut widgets)?;

        println!("{}", format);
    } else {
        use std::thread::sleep;
        use std::time::Duration;

        loop {
            update_format(&ctx, &mut format, &mut widgets)?;
            println!("{CLEAR_SEQ}{format}");

            if ctx.config.poll_rate > MINIMAL_POLL_RATE {
                sleep(Duration::from_millis((ctx.config.poll_rate - MINIMAL_POLL_RATE).into()));
            }

            // update all widgets
            for (_, widget) in widgets.iter_mut() {
                widget.update(&ctx)?;
            }

            sleep(Duration::from_millis(MINIMAL_POLL_RATE.into()));

            // reset format
            format = ctx.config.format.as_ref().unwrap().clone();

            // get fresh sensor data
            ctx.sensors_data = get_temps()?;
        }
    }

    Ok(())
}

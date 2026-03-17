mod cli;
mod config;
mod widget;

pub mod prelude {
    pub use anyhow::{Context as AnyhowContext, Result, anyhow, bail};
}

use clap::Parser;
use prelude::*;
use serde_json::Value as JsonValue;
use crate::{config::{Config, Sensor}, widget::{CPUUsageWidget, ContextIO, DummyWidget, SensorWidget, TimeWidget, Widget}};
use std::collections::HashMap;

const CLEAR_SEQ: &str = "\x1b[H\x1b[2J";

#[derive(Debug)]
struct Context {
    pub args: cli::Cli,
    pub config: Config,
    pub sensors_data: JsonValue,

    /// Data used by I/O widget
    pub io: Option<ContextIO>,
}

impl Context {
    pub fn new(args: cli::Cli, config: Config, sensors_data: JsonValue) -> Self {
        Self {
            args,
            config,
            sensors_data,
            io: None,
        }
    }
}

fn get_temps() -> Result<JsonValue> {
    let output = std::process::Command::new("sensors")
        .args(["-j", "--config", "/dev/null"])
        .output()
        .with_context(|| anyhow!("Unable to run sensors command"))?;

    let stdout = String::from_utf8(output.stdout)?;

    serde_json::from_str(&stdout)
        .with_context(|| anyhow!("Unable to parse json from sensors"))
}

fn format_var(var: &str) -> String {
    // very simple "{var}" formatter
    format!("{{{var}}}")
}

fn update_format(ctx: &Context, format: &mut String, widgets: &mut HashMap<String, Box<dyn Widget>>) -> Result<()> {
    // replace all instances
    for (var, widget) in widgets.iter_mut() {
        *format = format.replace(var, &widget.value(&ctx)?);
    }

    Ok(())
}

/// Filters widgets and adds only those present in format string
fn get_used_widgets(ctx: &mut Context) -> HashMap<String, Box<dyn Widget>> {
    let mut widgets: HashMap<String, Box<dyn Widget>> = HashMap::new();
    let format = ctx.config.format.as_ref().unwrap();

    // filtering out sensors that are not used
    for sensor in std::mem::take(&mut ctx.config.sensors) {
        let var = format_var(&sensor.name);
        if format.contains(&var) {
            widgets.insert(var, Box::new(SensorWidget::new(sensor)));
        }
    }

    let var = format_var(WIDGET_TIME);
    if format.contains(&var) {
        widgets.insert(var, Box::new(TimeWidget));
    }

    let var = format_var(WIDGET_CPU_USAGE);
    if format.contains(&var) {
        // cpu usage cannot be calculated at once
        widgets.insert(var, if ctx.args.once {
            Box::new(DummyWidget)
        } else {
            Box::new(CPUUsageWidget::new())
        });
    }

    widgets
}

const MINIMAL_POLL_RATE: u16 = 1000;

const WIDGET_CPU_USAGE: &str = "cpu_usage";
const WIDGET_TIME: &str = "time";
const WIDGET_IO_READ: &str = "io_read";
const WIDGET_IO_WRITE: &str = "io_write";

/// Contains names of all widgets
const WIDGETS: [&str; 4] = [ WIDGET_TIME, WIDGET_CPU_USAGE, WIDGET_IO_READ, WIDGET_IO_WRITE ];

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
    let mut ctx = Context::new(args, config, get_temps()?);

    // TODO alarms

    // list all widgets when no format is used
    if ctx.args.no_format || ctx.config.format.is_none() {
        let mut new_format = String::new();

        // add all builtin widgets
        for widget in WIDGETS {
            new_format += &format!("{widget}: {0:>20}\n", format_var(widget));
        }

        for sensor in &ctx.config.sensors {
            new_format += &format!("{}: {:>20}\n", sensor.name, format_var(&sensor.name));
        }

        // set the new format
        ctx.config.format = Some(new_format.trim().to_string());
    }

    let mut widgets: HashMap<String, Box<dyn Widget>> = get_used_widgets(&mut ctx);

    if widgets.contains_key(WIDGET_IO_READ) || widgets.contains_key(WIDGET_IO_WRITE) {
        ctx.io = Some(ContextIO::default());
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

mod cpu_usage;

pub use cpu_usage::CPUUsageWidget;

mod io_usage;
pub use io_usage::{IOReadWidget, IOWriteWidget, ContextIO};

use crate::prelude::*;
use crate::{Context, Sensor};

// TODO implement widget for memory usage and IO

pub trait Widget {
    /// Shown value of the widget
    fn value(&mut self, _ctx: &Context) -> Result<String> {
        Ok("".to_string())
    }

    /// Update method called one second before value is shown
    fn update(&mut self, _ctx: &Context) -> Result<()> {
        // update is not required for all widgets
        Ok(())
    }
}

/// Widget that shows current value of sensor
#[derive(Debug)]
pub struct SensorWidget {
    sensor: Sensor,
}

impl SensorWidget {
    pub fn new(sensor: Sensor) -> Self {
        Self {
            sensor,
        }
    }
}

impl Widget for SensorWidget {
    fn value(&mut self, ctx: &Context) -> Result<String> {
        Ok(self.sensor.format_value(self.sensor.get_value(&ctx.sensors_data)?))
    }
}

/// Widget that shows current time
#[derive(Debug)]
pub struct TimeWidget;

impl Widget for TimeWidget {
    fn value(&mut self, _ctx: &Context) -> Result<String> {
        Ok(chrono::Local::now().format("%H:%M:%S").to_string())
    }
}

/// Dummy widget that shows same value each time
#[derive(Debug)]
pub struct DummyWidget;

impl Widget for DummyWidget {
    fn value(&mut self, _ctx: &Context) -> Result<String> {
        Ok("N/A".to_string())
    }
}


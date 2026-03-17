use std::{cell::OnceCell, io::{BufRead, BufReader}};
use super::Widget;
use crate::{Context, prelude::*};

#[derive(Debug)]
pub struct CPUUsageWidget {
    last_idle: u64,
    last_sum: u64,
    cpu_count: OnceCell<u8>,
}

impl CPUUsageWidget {
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

    fn get(&mut self) -> Result<f64> {
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


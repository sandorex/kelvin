use super::Widget;
use crate::{Context, prelude::*};

fn get_io_usage() -> Result<(u64, u64)> {
    // TODO does sector size ever change?
    const SECTOR_SIZE: u64 = 512;

    // sector size is 512
    // NOTE i need second and fourth field (read and write)
    // 7 7 loop7 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0

    let contents = std::fs::read_to_string("/proc/diskstats")
        .with_context(|| anyhow!("Could not open /proc/diskstats"))?;

    let mut sum_read = 0;
    let mut sum_write = 0;

    for line in contents.lines() {
        let split = line.split_whitespace().collect::<Vec<_>>();

        // sectors read
        let read = split[5].parse::<u64>()
            .with_context(|| anyhow!("Could not parse read sectors in diskstats"))?;

        // sectors written
        let write = split[9] .parse::<u64>()
            .with_context(|| anyhow!("Could not parse written sectors in diskstats"))?;

        sum_read += read;
        sum_write += write;
    }

    Ok((
        sum_read * SECTOR_SIZE,
        sum_write * SECTOR_SIZE,
    ))
}

#[derive(Debug, Default)]
pub struct ContextIO {
    pub last_read: u64,
    pub last_write: u64,
    pub read: u64,
    pub write: u64,
}

impl ContextIO {
    pub fn update(&mut self) -> Result<()> {
        let usage = get_io_usage()?;

        self.read = (usage.0 - self.last_read) / 1_000 / 1_000;
        self.write = usage.1 - self.last_write;

        // let usage = (1000 * (delta_total - delta_idle + 5)) / 10;

        self.last_read = usage.0;
        self.last_write = usage.1;

        Ok(())
    }
}

/// Widget that shows I/O read
#[derive(Debug)]
pub struct IOReadWidget;

impl IOReadWidget {
    pub fn new(ctx: &mut Context) -> Self {
        if ctx.io.is_none() {
            ctx.io = Some(ContextIO::default());
        }

        Self
    }
}

impl Widget for IOReadWidget {
    fn value(&mut self, ctx: &Context) -> Result<String> {
        Ok(format!("{}", ctx.io.as_ref().unwrap().read))
    }
}

/// Widget that shows I/O write
#[derive(Debug)]
pub struct IOWriteWidget;

impl IOWriteWidget {
    pub fn new(ctx: &mut Context) -> Self {
        if ctx.io.is_none() {
            ctx.io = Some(ContextIO::default());
        }

        Self
    }
}

impl Widget for IOWriteWidget {
    fn value(&mut self, ctx: &Context) -> Result<String> {
        Ok(format!("{}", ctx.io.as_ref().unwrap().write))
    }
}

// impl Widget for IOWriteWidget {
//     fn value(&mut self, _ctx: &Context) -> Result<String> {
//         // let (curr_read, curr_write) = get_io_usage()?;
//         //
//         // unsafe {
//         //     let delta_read = curr_read - LAST_READ;
//         //     let delta_write = curr_write - LAST_WRITE;
//         // }
//         unsafe {
//             Ok(format!("{}", CURR_WRITE))
//         }
//     }
// }

// /// Widget that ONLY updates the I/O usage
// #[derive(Debug)]
// pub struct IOWidget {
//     last_read: u64,
//     last_write: u64,
//     read: u64,
//     write: u64,
// }
//
// impl IOWidget {
//     pub fn new() -> Self {
//         Self {
//             last_read: 0,
//             last_write: 0,
//             read: 0,
//             write: 0,
//         }
//     }
//
//     pub fn io_read<'a>(&'a self) -> IOReadWidget<'a> {
//         IOReadWidget(self)
//     }
//
//     pub fn io_write<'a>(&'a self) -> IOWriteWidget<'a> {
//         IOWriteWidget(self)
//     }
// }
//
// // TODO add 'visible() -> bool' function to widget so widget like this can be skipped
// impl Widget for IOWidget {
//     fn update(&mut self, _ctx: &Context) -> Result<()> {
//         let usage = get_io_usage()?;
//
//         self.read = usage.0 - self.last_read;
//         self.write = usage.1 - self.last_write;
//
//         self.last_read = usage.0;
//         self.last_write = usage.1;
//
//         Ok(())
//     }
// }


// impl IOUsageWidget {
//     pub fn new_read() -> Self {
//         Self(true)
//     }
//
//     pub fn new_write() -> Self {
//         Self(true)
//     }
//
//     pub fn refresh(&mut self) -> Result<()> {
//         let usage = Self::get_io_usage()?;
//
//         self.last_read = usage.0;
//         self.last_write = usage.1;
//
//         Ok(())
//     }
//
//     /// This gets current cpu usage since boot, used to calculate actual cpu usage
//     fn get_io_usage() -> Result<(u64, u64)> {
//         // TODO does sector size ever change?
//         const SECTOR_SIZE: u64 = 512;
//
//         // sector size is 512
//         // NOTE i need second and fourth field (read and write)
//         // 7 7 loop7 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0
//
//         let contents = std::fs::read_to_string("/proc/diskstats")
//             .with_context(|| anyhow!("Could not open /proc/diskstats"))?;
//
//         let mut sum_read = 0;
//         let mut sum_write = 0;
//
//         for line in contents.lines() {
//             let split = line.split_whitespace().collect::<Vec<_>>();
//
//             // sectors read
//             let read = split[5].parse::<u64>()
//                 .with_context(|| anyhow!("Could not parse read sectors in diskstats"))?;
//
//             // sectors written
//             let write = split[9] .parse::<u64>()
//                 .with_context(|| anyhow!("Could not parse written sectors in diskstats"))?;
//
//             sum_read += read;
//             sum_write += write;
//         }
//
//         Ok((
//             sum_read * SECTOR_SIZE,
//             sum_write * SECTOR_SIZE,
//         ))
//     }
//
//     fn get(&mut self) -> Result<f64> {
//         let curr = Self::get_io_usage()?;
//
//         let delta_read = self.last_read - curr.0;
//         let delta_write = self.last_write - curr.1;
//
//         Ok(1.0)
//     }
// }

// impl Widget for IOUsageWidget {
//     fn value(&mut self, _ctx: &Context) -> Result<String> {
//         // limit the decimals to 1
//         self.get().map(|x| format!("{x:.1}"))
//     }
//
//     fn update(&mut self, _ctx: &Context) -> Result<()> {
//         self.refresh()
//     }
// }


use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

/// Your friendly temperature monitor
#[derive(Parser, Debug, Clone)]
#[command(author, version, about)]
pub struct Cli {
    /// Load config from file instead of default paths
    ///
    /// By default looks for config in this order
    ///   ~/.config/kelvin/<hostname>.toml
    ///   ~/.config/kelvin/default.toml
    ///   /etc/kelvin/<hostname>.toml
    ///   /etc/kelvin/default.toml
    #[clap(short, long, verbatim_doc_comment)]
    pub config: Option<PathBuf>,

    // /// Target block device, leave empty for prompt
    // // hide the flag on windows cause its useless
    // #[cfg_attr(target_os = "windows", clap(skip))]
    // pub target: Option<String>,
    //
    // /// Shows all disk devices instead of only removable ones (SD, Flash drive..)
    // #[clap(long)]
    // pub show_all_disks: bool,

    // #[command(subcommand)]
    // pub cmd: CliCommands,
}

// #[derive(Args, Debug, Clone)]
// pub struct CmdShuffle {
//     /// Repeats all songs until they fill up at minimum this amount of time
//     ///
//     /// This is a hack to implement quasi-shuffle by repeating everything but
//     /// in different predefined order
//     ///
//     /// This feature can create A LOT of links so beware it can take a while
//     #[clap(long)]
//     pub repeat_fill: Option<Duration>,
// }
//
// #[derive(Args, Debug, Clone)]
// pub struct CmdClean {
//     /// Remove songs as well as links
//     #[clap(long, short)]
//     pub songs: bool,
// }
//
// #[derive(Args, Debug, Clone)]
// pub struct CmdImport {
//     /// Files or directories to recursively scan for MP3 files to import
//     #[clap(required = true, num_args = 1..)]
//     pub paths: Vec<PathBuf>,
// }
//
// #[derive(Args, Debug, Clone)]
// pub struct CmdProcess {
//     /// Overwrite existing files
//     #[clap(short, long)]
//     pub overwrite: bool,
//
//     /// Adjust volume of files (in decibels)
//     ///
//     /// +/-10dB => doubles or halves the volume
//     #[clap(short, long, allow_negative_numbers = true)]
//     pub volume_adjustment: Option<f64>,
//
//     /// Output path
//     #[clap(required = true)]
//     pub output: PathBuf,
//
//     /// Files or directories to recursively scan for MP3 files to fix
//     #[clap(required = true, num_args = 1..)]
//     pub paths: Vec<PathBuf>,
// }
//
// #[derive(Subcommand, Debug, Clone)]
// pub enum CliCommands {
//     /// Formats device/partition (ERASES ALL DATA!)
//     ///
//     /// In case target is a device block file then it formats it to contain a
//     /// single FAT32 partition with MBR/BIOS partition table
//     #[cfg_attr(target_os = "windows", clap(skip))]
//     Format,
//
//     /// Shuffle music
//     Shuffle(CmdShuffle),
//
//     /// Cleans up the links making it editable directly
//     Clean(CmdClean),
//
//     /// Imports file into the filesystem without mounting it, will not overwrite files
//     Import(CmdImport),
//
//     /// Processes files using ffmpeg to apply some adjustments (recommended)
//     ///
//     /// All options have a description but always test if the files are playable on a computer!
//     Process(CmdProcess),
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        Cli::command().debug_assert()
    }
}

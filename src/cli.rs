use std::path::PathBuf;
use clap::{Parser};

const HELP_DAEMON: &str = "Daemon Related";

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

    /// Run in the background, alarm is always enabled in this mode
    ///
    /// Uses systemd if available to show logs in systemctl, if there is a
    /// process running it will be restarted
    #[clap(short, long, help_heading = HELP_DAEMON)]
    pub daemon: bool,

    /// Enable alarm
    ///
    /// Note that if you have a daemon process running this will won't do
    /// anything, as two processes triggering alarms is jarring
    #[clap(short, long)]
    pub alarm: bool,

    /// Kill existing daemon process
    #[clap(long, help_heading = HELP_DAEMON)]
    pub kill: bool,

    /// Print the output once and quit
    #[clap(long)]
    pub once: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        Cli::command().debug_assert()
    }
}

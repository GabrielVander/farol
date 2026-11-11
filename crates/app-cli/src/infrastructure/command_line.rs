use std::time::Duration;

use clap::{Args, Parser, Subcommand, ValueEnum};

use crate::adapters::controllers::records_controller::{Action, Provider, RecordsRequest};

/// Keeps DNS A records pointed at this machine's public IP.
#[derive(Parser)]
#[command(name = "farol", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

impl Cli {
    pub fn into_request(self) -> RecordsRequest {
        let (action, options) = match self.command {
            Command::Check(options) => (Action::Check, options),
            Command::Sync(options) => (options.action(), options.records),
        };

        RecordsRequest {
            action,
            provider: options.provider.into(),
            records: options.records,
            ttl: options.ttl,
        }
    }
}

#[derive(Subcommand)]
pub enum Command {
    /// Show which records have drifted, without changing them
    Check(RecordOptions),
    /// Point every drifted record at the public IP
    Sync(SyncOptions),
}

#[derive(Args)]
pub struct RecordOptions {
    /// DNS provider that hosts the records
    #[arg(long, value_enum)]
    pub provider: ProviderOption,
    /// Record to track, repeatable. Use @ as the name for the domain itself, * for a wildcard
    #[arg(long = "record", value_name = "DOMAIN:NAME", required = true)]
    pub records: Vec<String>,
    /// TTL in seconds for records farol points
    #[arg(long, default_value_t = 300)]
    pub ttl: u32,
}

#[derive(Args)]
pub struct SyncOptions {
    #[command(flatten)]
    pub records: RecordOptions,
    /// Keep running, and sync again whenever the public IP changes. Stop with Ctrl+C
    #[arg(long)]
    pub watch: bool,
    /// Seconds between public IP checks while watching
    #[arg(
        long,
        value_name = "SECONDS",
        default_value_t = 300,
        value_parser = clap::value_parser!(u64).range(1..),
        requires = "watch"
    )]
    pub interval: u64,
}

impl SyncOptions {
    fn action(&self) -> Action {
        if !self.watch {
            return Action::Sync;
        }

        Action::Watch {
            interval: Duration::from_secs(self.interval),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ProviderOption {
    Hostinger,
}

impl From<ProviderOption> for Provider {
    fn from(option: ProviderOption) -> Self {
        match option {
            ProviderOption::Hostinger => Self::Hostinger,
        }
    }
}

#[cfg(test)]
mod test {
    use clap::CommandFactory;

    use crate::infrastructure::command_line::Cli;

    #[test]
    fn should_define_a_valid_command_line() {
        Cli::command().debug_assert();
    }
}

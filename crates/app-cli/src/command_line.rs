use clap::{Args, Parser, Subcommand, ValueEnum};

/// Keeps DNS A records pointed at this machine's public IP.
#[derive(Parser)]
#[command(name = "farol", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Show which records have drifted, without changing them
    Check(RecordOptions),
    /// Point every drifted record at the public IP
    Sync(RecordOptions),
}

impl Command {
    pub fn options(&self) -> &RecordOptions {
        match self {
            Self::Check(options) | Self::Sync(options) => options,
        }
    }
}

#[derive(Args)]
pub struct RecordOptions {
    /// DNS provider that hosts the records
    #[arg(long, value_enum)]
    pub provider: Provider,
    /// Record to track, repeatable. Use @ as the name for the domain itself, * for a wildcard
    #[arg(long = "record", value_name = "DOMAIN:NAME", required = true)]
    pub records: Vec<String>,
    /// TTL in seconds for records farol points
    #[arg(long, default_value_t = 300)]
    pub ttl: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Provider {
    Hostinger,
}

#[cfg(test)]
mod test {
    use clap::CommandFactory;

    use crate::command_line::Cli;

    #[test]
    fn should_define_a_valid_command_line() {
        Cli::command().debug_assert();
    }
}

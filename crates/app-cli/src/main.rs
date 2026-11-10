use std::{env, process::ExitCode, sync::Arc, time::Duration};

use app_cli::{
    record_arguments::tracked_records,
    record_arguments_view::explain_record_argument_error,
    report_view::{explain_public_ip_error, has_failures, render},
    token::{HOSTINGER_TOKEN_VARIABLE, hostinger_token},
};
use clap::{Args, Parser, Subcommand, ValueEnum};
use lib_core::use_cases::{
    check_records::{CheckRecords, CheckRecordsUseCase},
    report::Report,
    sync_records::{SyncRecords, SyncRecordsUseCase},
};
use lib_hostinger::infrastructure::{
    adapters::hostinger_dns_gateway::HostingerDnsGateway, ureq_hostinger_api::UreqHostingerApi,
};
use lib_ipify::infrastructure::{
    adapters::ipify_public_ip_gateway::IpifyPublicIpGateway, ureq_ipify_api::UreqIpifyApi,
};

const IPIFY_URL: &str = "https://api.ipify.org";
const IPIFY_TIMEOUT: Duration = Duration::from_secs(10);
const HOSTINGER_URL: &str = "https://developers.hostinger.com";
const HOSTINGER_TIMEOUT: Duration = Duration::from_secs(30);

type PublicIpGateway = IpifyPublicIpGateway<UreqIpifyApi>;
type DnsGateway = HostingerDnsGateway<UreqHostingerApi>;

/// Keeps DNS A records pointed at this machine's public IP.
#[derive(Parser)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Show which records have drifted, without changing them
    Check(RecordOptions),
    /// Point every drifted record at the public IP
    Sync(RecordOptions),
}

impl Command {
    fn options(&self) -> &RecordOptions {
        match self {
            Self::Check(options) | Self::Sync(options) => options,
        }
    }
}

#[derive(Args)]
struct RecordOptions {
    /// DNS provider that hosts the records
    #[arg(long, value_enum)]
    provider: Provider,
    /// Record to track, repeatable. Use @ as the name for the domain itself, * for a wildcard
    #[arg(long = "record", value_name = "DOMAIN:NAME", required = true)]
    records: Vec<String>,
    /// TTL in seconds for records farol points
    #[arg(long, default_value_t = 300)]
    ttl: u32,
}

#[derive(Clone, Copy, ValueEnum)]
enum Provider {
    Hostinger,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(&cli.command) {
        Ok(report) => print_report(&report),
        Err(message) => {
            eprintln!("farol: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run(command: &Command) -> Result<Report, String> {
    let options = command.options();
    let records = tracked_records(&options.records, options.ttl)
        .map_err(|error| explain_record_argument_error(&error))?;
    let dns_gateway = Arc::new(dns_gateway(options.provider)?);
    let public_ip_gateway = Arc::new(public_ip_gateway());
    let report = match command {
        Command::Check(_) => CheckRecords::new(public_ip_gateway, dns_gateway).execute(&records),
        Command::Sync(_) => SyncRecords::new(public_ip_gateway, dns_gateway).execute(&records),
    };
    report.map_err(|error| explain_public_ip_error(&error))
}

fn print_report(report: &Report) -> ExitCode {
    print!("{}", render(report));
    if has_failures(report) {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn dns_gateway(provider: Provider) -> Result<DnsGateway, String> {
    match provider {
        Provider::Hostinger => {
            let token = hostinger_token(environment)
                .map_err(|_| format!("set {HOSTINGER_TOKEN_VARIABLE} to a Hostinger API token"))?;
            let api = UreqHostingerApi::new(HOSTINGER_URL.to_owned(), token, HOSTINGER_TIMEOUT);
            Ok(HostingerDnsGateway::new(api))
        }
    }
}

fn public_ip_gateway() -> PublicIpGateway {
    IpifyPublicIpGateway::new(UreqIpifyApi::new(IPIFY_URL.to_owned(), IPIFY_TIMEOUT))
}

fn environment(key: &str) -> Option<String> {
    env::var(key).ok()
}

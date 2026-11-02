use std::{env, fs, path::PathBuf, process::ExitCode, sync::Arc, time::Duration};

use app_cli::{
    config::{Config, Provider},
    config_path::config_path,
    config_view::explain_config_error,
    report_view::{explain_public_ip_error, has_failures, render},
    token::{HOSTINGER_TOKEN_VARIABLE, hostinger_token},
};
use clap::{Parser, Subcommand};
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

type PublicIpGateway = IpifyPublicIpGateway<UreqIpifyApi>;
type DnsGateway = HostingerDnsGateway<UreqHostingerApi>;

/// Keeps DNS A records pointed at this machine's public IP.
#[derive(Parser)]
#[command(version)]
struct Cli {
    /// Config file (default: $XDG_CONFIG_HOME/farol/config.toml)
    #[arg(long)]
    config: Option<PathBuf>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Clone, Copy)]
enum Command {
    /// Show which records have drifted, without changing them
    Check,
    /// Point every drifted record at the public IP
    Sync,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(report) => print_report(&report),
        Err(message) => {
            eprintln!("farol: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<Report, String> {
    let config = load_config(cli.config)?;
    let dns_gateway = Arc::new(dns_gateway(config.provider)?);
    let public_ip_gateway = Arc::new(public_ip_gateway());
    let report = match cli.command {
        Command::Check => {
            CheckRecords::new(public_ip_gateway, dns_gateway).execute(&config.records)
        }
        Command::Sync => SyncRecords::new(public_ip_gateway, dns_gateway).execute(&config.records),
    };
    report.map_err(|error| explain_public_ip_error(&error))
}

fn print_report(report: &Report) -> ExitCode {
    print!("{}", render(report));
    match has_failures(report) {
        true => ExitCode::FAILURE,
        false => ExitCode::SUCCESS,
    }
}

fn load_config(explicit: Option<PathBuf>) -> Result<Config, String> {
    let path = config_path(explicit, environment)
        .ok_or("no config found: pass --config, or set XDG_CONFIG_HOME or HOME")?;
    let text = fs::read_to_string(&path)
        .map_err(|error| format!("could not read {}: {error}", path.display()))?;
    Config::parse(&text)
        .map_err(|error| format!("{}: {}", path.display(), explain_config_error(&error)))
}

fn dns_gateway(provider: Provider) -> Result<DnsGateway, String> {
    match provider {
        Provider::Hostinger => {
            let token = hostinger_token(environment)
                .map_err(|_| format!("set {HOSTINGER_TOKEN_VARIABLE} to a Hostinger API token"))?;
            let api = UreqHostingerApi::new(HOSTINGER_URL.to_owned(), token);
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

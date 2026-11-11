use std::{ffi::OsString, sync::Arc};

use clap::Parser;
use lib_core::use_cases::{
    check_records::{CheckRecords, CheckRecordsUseCase},
    ports::{dns_gateway::DnsGateway, public_ip_gateway::PublicIpGateway},
    report::Report,
    sync_records::{SyncRecords, SyncRecordsUseCase},
};

use crate::{
    adapters::{
        controllers::record_arguments::tracked_records,
        presenters::{
            record_arguments_view::explain_record_argument_error,
            report_view::{explain_public_ip_error, has_failures, render},
        },
    },
    infrastructure::command_line::{Cli, Command, Provider},
};

/// The outside world farol talks to. Production reaches real services; tests
/// plug in fakes.
pub trait Infrastructure {
    type PublicIpGateway: PublicIpGateway;
    type DnsGateway: DnsGateway;

    fn public_ip_gateway(&self) -> Arc<Self::PublicIpGateway>;

    /// The gateway to `provider`, or why it cannot be reached.
    fn dns_gateway(&self, provider: Provider) -> Result<Arc<Self::DnsGateway>, String>;
}

/// How a run ended: what it printed and whether it succeeded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Termination {
    pub success: bool,
    pub output: String,
    pub error: String,
}

impl Termination {
    fn reported(report: &Report) -> Self {
        Self {
            success: !has_failures(report),
            output: render(report),
            error: String::new(),
        }
    }

    fn stopped(message: &str) -> Self {
        Self {
            success: false,
            output: String::new(),
            error: format!("farol: {message}\n"),
        }
    }
}

/// Runs farol for `arguments`, which start with the program name. Fails only
/// when the command line itself is wrong, or asks for help or the version.
pub fn run<T: Into<OsString> + Clone>(
    arguments: impl IntoIterator<Item = T>,
    infrastructure: &impl Infrastructure,
) -> Result<Termination, clap::Error> {
    let cli = Cli::try_parse_from(arguments)?;
    let termination = match execute(&cli.command, infrastructure) {
        Ok(report) => Termination::reported(&report),
        Err(message) => Termination::stopped(&message),
    };
    Ok(termination)
}

fn execute(command: &Command, infrastructure: &impl Infrastructure) -> Result<Report, String> {
    let options = command.options();
    let records = tracked_records(&options.records, options.ttl)
        .map_err(|error| explain_record_argument_error(&error))?;
    let dns_gateway = infrastructure.dns_gateway(options.provider)?;
    let public_ip_gateway = infrastructure.public_ip_gateway();
    let report = match command {
        Command::Check(_) => CheckRecords::new(public_ip_gateway, dns_gateway).execute(&records),
        Command::Sync(_) => SyncRecords::new(public_ip_gateway, dns_gateway).execute(&records),
    };
    report.map_err(|error| explain_public_ip_error(&error))
}

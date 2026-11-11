use std::sync::Arc;

use lib_core::use_cases::{
    check_records::{CheckRecords, CheckRecordsUseCase},
    ports::{dns_gateway::DnsGateway, public_ip_gateway::PublicIpGateway},
    report::Report,
    sync_records::{SyncRecords, SyncRecordsUseCase},
};

use crate::adapters::{
    controllers::record_arguments::tracked_records,
    presenters::{
        record_arguments_view::explain_record_argument_error,
        report_view::{explain_public_ip_error, render},
    },
};

/// The outside world farol talks to. Production reaches real services; tests
/// plug in fakes.
pub trait Infrastructure {
    type PublicIpGateway: PublicIpGateway;
    type DnsGateway: DnsGateway;

    fn public_ip_gateway(&self) -> Arc<Self::PublicIpGateway>;

    fn dns_gateway(&self, provider: Provider) -> Result<Arc<Self::DnsGateway>, String>;
}

/// What farol was asked to do, with the records still as the user wrote them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordsRequest {
    pub action: Action,
    pub provider: Provider,
    pub records: Vec<String>,
    pub ttl: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Check,
    Sync,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provider {
    Hostinger,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Termination {
    pub success: bool,
    pub output: String,
    pub error: String,
}

impl Termination {
    fn reported(report: &Report) -> Self {
        Self {
            success: !report.has_failures(),
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

pub fn handle(request: &RecordsRequest, infrastructure: &impl Infrastructure) -> Termination {
    match execute(request, infrastructure) {
        Ok(report) => Termination::reported(&report),
        Err(message) => Termination::stopped(&message),
    }
}

fn execute(
    request: &RecordsRequest,
    infrastructure: &impl Infrastructure,
) -> Result<Report, String> {
    let records = tracked_records(&request.records, request.ttl)
        .map_err(|error| explain_record_argument_error(&error))?;

    let dns_gateway = infrastructure.dns_gateway(request.provider)?;
    let public_ip_gateway = infrastructure.public_ip_gateway();

    let report = match request.action {
        Action::Check => CheckRecords::new(public_ip_gateway, dns_gateway).execute(&records),
        Action::Sync => SyncRecords::new(public_ip_gateway, dns_gateway).execute(&records),
    };

    report.map_err(|error| explain_public_ip_error(&error))
}

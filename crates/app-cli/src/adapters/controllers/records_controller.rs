use std::{iter, sync::Arc, time::Duration};

use lib_core::{
    entities::tracked_record::TrackedRecord,
    use_cases::{
        check_records::{CheckRecords, CheckRecordsUseCase},
        ports::{
            dns_gateway::DnsGateway,
            public_ip_gateway::{PublicIpGateway, PublicIpGatewayError},
        },
        report::Report,
        sync_records::{SyncRecords, SyncRecordsUseCase},
        watch_records::{WatchRecords, WatchRecordsUseCase},
    },
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

    /// One item after each wait of `interval`, for as long as a watch should go on.
    fn ticks(&self, interval: Duration) -> impl Iterator<Item = ()>;

    /// Shows the response of a watch round as soon as it is ready.
    fn show(&self, response: &Response);
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
    /// Sync, then sync again every time the public IP changes.
    Watch {
        interval: Duration,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provider {
    Hostinger,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Response {
    pub success: bool,
    pub output: String,
    pub error: String,
}

impl Response {
    fn reported(report: &Report) -> Self {
        Self {
            success: !report.has_failures(),
            output: render(report),
            error: String::new(),
        }
    }

    /// How a watch ends: every round already showed its own response.
    fn watched() -> Self {
        Self {
            success: true,
            output: String::new(),
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

pub fn handle(request: &RecordsRequest, infrastructure: &impl Infrastructure) -> Response {
    execute(request, infrastructure).unwrap_or_else(|message| Response::stopped(&message))
}

fn execute(
    request: &RecordsRequest,
    infrastructure: &impl Infrastructure,
) -> Result<Response, String> {
    let records = tracked_records(&request.records, request.ttl)
        .map_err(|error| explain_record_argument_error(&error))?;

    let dns_gateway = infrastructure.dns_gateway(request.provider)?;
    let public_ip_gateway = infrastructure.public_ip_gateway();

    let response = match request.action {
        Action::Check => {
            respond(CheckRecords::new(public_ip_gateway, dns_gateway).execute(&records))
        }
        Action::Sync => respond(SyncRecords::new(public_ip_gateway, dns_gateway).execute(&records)),
        Action::Watch { interval } => watch(
            WatchRecords::new(public_ip_gateway, dns_gateway),
            &records,
            interval,
            infrastructure,
        ),
    };

    Ok(response)
}

fn respond(report: Result<Report, PublicIpGatewayError>) -> Response {
    match report {
        Ok(report) => Response::reported(&report),
        Err(error) => Response::stopped(&explain_public_ip_error(&error)),
    }
}

/// Runs a first round at once, then one more round after every tick.
fn watch(
    mut watch_records: impl WatchRecordsUseCase,
    records: &[TrackedRecord],
    interval: Duration,
    infrastructure: &impl Infrastructure,
) -> Response {
    for () in iter::once(()).chain(infrastructure.ticks(interval)) {
        if let Some(round) = watch_records.execute(records).transpose() {
            infrastructure.show(&respond(round));
        }
    }

    Response::watched()
}

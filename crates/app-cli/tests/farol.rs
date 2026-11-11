//! farol as a whole, driven through its command line with fake services.

use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use app_cli::{
    adapters::controllers::records_controller::{Infrastructure, Provider, Response},
    infrastructure::run::run,
};
use clap::error::ErrorKind;
use lib_core::use_cases::ports::{
    dns_gateway::DnsGatewayError, public_ip_gateway::PublicIpGatewayError,
};
use lib_core_testing::{
    fakes::{FakeDnsGateway, FakePublicIpGateway},
    fixtures::{OLD_ADDRESS, PUBLIC_ADDRESS, next_public_ip, nothing_pointed, public_ip, tracked},
};

const CHECK_HOME: &str = "check --provider hostinger --record example.com:home";
const WATCH_HOME: &str = "sync --watch --provider hostinger --record example.com:home";

#[derive(Default)]
struct FakeInfrastructure {
    public_ip_gateway: Arc<FakePublicIpGateway>,
    dns_gateway: Arc<FakeDnsGateway>,
    provider_failure: Option<String>,
    ticks: usize,
    waits: Mutex<Vec<Duration>>,
    shown: Mutex<Vec<Response>>,
}

impl FakeInfrastructure {
    fn publishing(dns_gateway: FakeDnsGateway) -> Self {
        Self {
            dns_gateway: Arc::new(dns_gateway),
            ..Self::default()
        }
    }

    fn waits(&self) -> Vec<Duration> {
        self.waits.lock().expect("unpoisoned").clone()
    }

    fn shown(&self) -> Vec<Response> {
        self.shown.lock().expect("unpoisoned").clone()
    }

    fn run(&self, arguments: &str) -> Result<Response, clap::Error> {
        let arguments = ["farol"].into_iter().chain(arguments.split_whitespace());
        run(arguments, self)
    }
}

impl Infrastructure for FakeInfrastructure {
    type PublicIpGateway = FakePublicIpGateway;
    type DnsGateway = FakeDnsGateway;

    fn public_ip_gateway(&self) -> Arc<FakePublicIpGateway> {
        Arc::clone(&self.public_ip_gateway)
    }

    fn dns_gateway(&self, _provider: Provider) -> Result<Arc<FakeDnsGateway>, String> {
        match &self.provider_failure {
            Some(message) => Err(message.clone()),
            None => Ok(Arc::clone(&self.dns_gateway)),
        }
    }

    fn ticks(&self, interval: Duration) -> impl Iterator<Item = ()> {
        (0..self.ticks).map(move |_| self.waits.lock().expect("unpoisoned").push(interval))
    }

    fn show(&self, response: &Response) {
        self.shown
            .lock()
            .expect("unpoisoned")
            .push(response.clone());
    }
}

fn farol(infrastructure: &FakeInfrastructure, arguments: &str) -> Response {
    infrastructure.run(arguments).expect("valid command line")
}

fn reported(output: &str) -> Response {
    Response {
        success: true,
        output: output.to_owned(),
        error: String::new(),
    }
}

fn stopped(message: &str) -> Response {
    Response {
        success: false,
        output: String::new(),
        error: format!("farol: {message}\n"),
    }
}

#[test]
fn should_check_an_unchanged_record_and_succeed() {
    let infrastructure = FakeInfrastructure::publishing(
        FakeDnsGateway::default().with_addresses("home", &[PUBLIC_ADDRESS]),
    );

    let response = farol(&infrastructure, CHECK_HOME);

    let expected = Response {
        success: true,
        output: "public IP 8.8.8.8\nhome.example.com: unchanged\n".to_owned(),
        error: String::new(),
    };
    assert_eq!(expected, response);
}

#[test]
fn should_check_a_drifted_record_without_pointing_it() {
    let infrastructure = FakeInfrastructure::publishing(
        FakeDnsGateway::default().with_addresses("home", &[OLD_ADDRESS]),
    );

    let response = farol(&infrastructure, CHECK_HOME);

    let expected = "public IP 8.8.8.8\nhome.example.com: drifted (now 1.1.1.1, TTL 300)\n";
    assert_eq!(expected, response.output);
    assert!(response.success);
    assert_eq!(nothing_pointed(), infrastructure.dns_gateway.pointed());
}

#[test]
fn should_sync_a_drifted_record_by_pointing_it() {
    let infrastructure = FakeInfrastructure::publishing(
        FakeDnsGateway::default().with_addresses("home", &[OLD_ADDRESS]),
    );

    let response = farol(
        &infrastructure,
        "sync --provider hostinger --record example.com:home",
    );

    let expected = "public IP 8.8.8.8\nhome.example.com: pointed (was 1.1.1.1, TTL 300)\n";
    assert_eq!(expected, response.output);
    assert!(response.success);
    assert_eq!(
        vec![(tracked("home"), public_ip())],
        infrastructure.dns_gateway.pointed()
    );
}

#[test]
fn should_report_every_record_but_fail_when_one_fails() {
    let error = DnsGatewayError::Unavailable("timeout".to_owned());
    let infrastructure = FakeInfrastructure::publishing(
        FakeDnsGateway::default()
            .with_read_failure("home", error)
            .with_addresses("lab", &[PUBLIC_ADDRESS]),
    );

    let response = farol(
        &infrastructure,
        "check --provider hostinger --record example.com:home --record example.com:lab",
    );

    let expected = "public IP 8.8.8.8\n\
        home.example.com: failed, the DNS provider is unavailable (timeout)\n\
        lab.example.com: unchanged\n";
    assert_eq!(expected, response.output);
    assert!(!response.success);
}

#[test]
fn should_stop_when_the_public_ip_is_unavailable() {
    let error = PublicIpGatewayError::Unavailable("offline".to_owned());
    let infrastructure = FakeInfrastructure {
        public_ip_gateway: Arc::new(FakePublicIpGateway::failing(error)),
        ..FakeInfrastructure::default()
    };

    let response = farol(&infrastructure, CHECK_HOME);

    let message = "could not find the public IP: the public IP service is unavailable (offline)";
    assert_eq!(stopped(message), response);
}

#[test]
fn should_stop_on_an_invalid_record_argument() {
    let response = farol(
        &FakeInfrastructure::default(),
        "check --provider hostinger --record example.com",
    );

    let message =
        "invalid record \"example.com\": expected DOMAIN:NAME, for example example.com:home";
    assert_eq!(stopped(message), response);
}

#[test]
fn should_stop_when_the_dns_provider_cannot_be_reached() {
    let infrastructure = FakeInfrastructure {
        provider_failure: Some("no token".to_owned()),
        ..FakeInfrastructure::default()
    };

    let response = farol(&infrastructure, CHECK_HOME);

    assert_eq!(stopped("no token"), response);
}

#[test]
fn should_watch_and_show_a_report_only_when_the_public_ip_changes() {
    let infrastructure = FakeInfrastructure {
        public_ip_gateway: Arc::new(FakePublicIpGateway::answering(vec![
            Ok(public_ip()),
            Ok(public_ip()),
            Ok(next_public_ip()),
        ])),
        dns_gateway: Arc::new(FakeDnsGateway::default().with_addresses("home", &[OLD_ADDRESS])),
        ticks: 2,
        ..FakeInfrastructure::default()
    };

    let response = farol(&infrastructure, WATCH_HOME);

    let expected = vec![
        reported("public IP 8.8.8.8\nhome.example.com: pointed (was 1.1.1.1, TTL 300)\n"),
        reported("public IP 9.9.9.9\nhome.example.com: pointed (was 1.1.1.1, TTL 300)\n"),
    ];
    assert_eq!(expected, infrastructure.shown());
    assert_eq!(vec![Duration::from_secs(300); 2], infrastructure.waits());
    assert_eq!(reported(""), response);
}

#[test]
fn should_refuse_a_command_line_without_records() {
    let infrastructure = FakeInfrastructure::default();

    let error = infrastructure
        .run("check --provider hostinger")
        .expect_err("invalid command line");

    assert_eq!(ErrorKind::MissingRequiredArgument, error.kind());
}

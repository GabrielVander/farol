//! farol as a whole, driven through its command line with fake services.

use std::sync::Arc;

use app_cli::{
    command_line::Provider,
    run::{Infrastructure, Termination, run},
};
use lib_core::use_cases::dns_gateway::DnsGatewayError;
use lib_core_testing::{
    fakes::{FakeDnsGateway, FakePublicIpGateway},
    fixtures::{OLD_ADDRESS, PUBLIC_ADDRESS, public_ip, tracked},
    scenario::Scenario,
};

struct FakeInfrastructure(Scenario);

impl Infrastructure for FakeInfrastructure {
    type PublicIpGateway = FakePublicIpGateway;
    type DnsGateway = FakeDnsGateway;

    fn public_ip_gateway(&self) -> Arc<FakePublicIpGateway> {
        self.0.public_ip_gateway()
    }

    fn dns_gateway(&self, _provider: Provider) -> Result<Arc<FakeDnsGateway>, String> {
        Ok(self.0.dns_gateway())
    }
}

fn farol(scenario: &Scenario, arguments: &str) -> Termination {
    let arguments = ["farol"].into_iter().chain(arguments.split_whitespace());
    run(arguments, &FakeInfrastructure(scenario.clone())).expect("valid command line")
}

#[test]
fn should_check_an_unchanged_record_and_succeed() {
    let scenario = Scenario::default().publishing("home", &[PUBLIC_ADDRESS]);

    let termination = farol(
        &scenario,
        "check --provider hostinger --record example.com:home",
    );

    let expected = Termination {
        success: true,
        output: "public IP 8.8.8.8\nhome.example.com: unchanged\n".to_owned(),
        error: String::new(),
    };
    assert_eq!(expected, termination);
}

#[test]
fn should_check_a_drifted_record_without_pointing_it() {
    let scenario = Scenario::default().publishing("home", &[OLD_ADDRESS]);

    let termination = farol(
        &scenario,
        "check --provider hostinger --record example.com:home",
    );

    let expected = "public IP 8.8.8.8\nhome.example.com: drifted (now 1.1.1.1, TTL 300)\n";
    assert_eq!(expected, termination.output);
    assert!(termination.success);
    assert!(scenario.pointed().is_empty());
}

#[test]
fn should_sync_a_drifted_record_by_pointing_it() {
    let scenario = Scenario::default().publishing("home", &[OLD_ADDRESS]);

    let termination = farol(
        &scenario,
        "sync --provider hostinger --record example.com:home",
    );

    let expected = "public IP 8.8.8.8\nhome.example.com: pointed (was 1.1.1.1, TTL 300)\n";
    assert_eq!(expected, termination.output);
    assert!(termination.success);
    assert_eq!(vec![(tracked("home"), public_ip())], scenario.pointed());
}

#[test]
fn should_report_every_record_but_fail_when_one_fails() {
    let error = DnsGatewayError::Unavailable("timeout".to_owned());
    let scenario = Scenario::default()
        .failing_to_read("home", error)
        .publishing("lab", &[PUBLIC_ADDRESS]);

    let termination = farol(
        &scenario,
        "check --provider hostinger --record example.com:home --record example.com:lab",
    );

    let expected = "public IP 8.8.8.8\n\
        home.example.com: failed, the DNS provider is unavailable (timeout)\n\
        lab.example.com: unchanged\n";
    assert_eq!(expected, termination.output);
    assert!(!termination.success);
}

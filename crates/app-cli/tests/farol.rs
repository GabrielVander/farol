//! farol as a whole, driven through its command line with fake services.

use std::sync::Arc;

use app_cli::{
    command_line::Provider,
    run::{Infrastructure, Termination, run},
};
use lib_core_testing::{
    fakes::{FakeDnsGateway, FakePublicIpGateway},
    fixtures::PUBLIC_ADDRESS,
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

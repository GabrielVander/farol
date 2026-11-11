use lib_core::use_cases::{
    ports::dns_gateway::DnsGatewayError,
    report::{Outcome, RecordOutcome, Report},
};
use lib_core_testing::fixtures::{public_ip, published, tracked};

fn report(outcomes: Vec<Outcome>) -> Report {
    let records = outcomes
        .into_iter()
        .map(|outcome| RecordOutcome {
            record: tracked("home"),
            outcome,
        })
        .collect();

    Report {
        public_ip: public_ip(),
        records,
    }
}

#[test]
fn should_have_no_failures_when_every_record_succeeded() {
    let report = report(vec![
        Outcome::Unchanged,
        Outcome::Pointed {
            previous: published(&[]),
        },
    ]);

    assert!(!report.has_failures());
}

#[test]
fn should_have_failures_when_any_record_failed() {
    let failure = Outcome::Failed(DnsGatewayError::Unavailable("timeout".to_owned()));

    let report = report(vec![Outcome::Unchanged, failure]);

    assert!(report.has_failures());
}

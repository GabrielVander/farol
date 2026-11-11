use lib_core::use_cases::{
    dns_gateway::{DnsGatewayError, PublishedRecord},
    public_ip_gateway::PublicIpGatewayError,
    report::{Outcome, RecordOutcome},
};
use lib_core_testing::{
    fixtures::{OLD_ADDRESS, PUBLIC_ADDRESS, published, tracked},
    scenario::Scenario,
};

#[test]
fn should_report_unchanged_when_the_record_points_at_the_public_ip() {
    let scenario = Scenario::default().publishing("home", &[PUBLIC_ADDRESS]);

    let report = scenario.check(&["home"]).expect("report");

    let expected = RecordOutcome {
        record: tracked("home"),
        outcome: Outcome::Unchanged,
    };
    assert_eq!(vec![expected], report.records);
}

#[test]
fn should_report_drifted_when_the_record_points_elsewhere() {
    let scenario = Scenario::default().publishing("home", &[OLD_ADDRESS]);

    let report = scenario.check(&["home"]).expect("report");

    let expected = RecordOutcome {
        record: tracked("home"),
        outcome: Outcome::Drifted {
            current: published(&[OLD_ADDRESS]),
        },
    };
    assert_eq!(vec![expected], report.records);
}

#[test]
fn should_report_drifted_when_only_the_ttl_differs() {
    let current = PublishedRecord {
        addresses: vec![PUBLIC_ADDRESS],
        ttl: Some(3600),
    };
    let scenario = Scenario::default().publishing_record("home", current.clone());

    let report = scenario.check(&["home"]).expect("report");

    let expected = RecordOutcome {
        record: tracked("home"),
        outcome: Outcome::Drifted { current },
    };
    assert_eq!(vec![expected], report.records);
}

#[test]
fn should_report_a_missing_record_as_drifted() {
    let report = Scenario::default().check(&["home"]).expect("report");

    let expected = RecordOutcome {
        record: tracked("home"),
        outcome: Outcome::Drifted {
            current: published(&[]),
        },
    };
    assert_eq!(vec![expected], report.records);
}

#[test]
fn should_report_several_addresses_as_drifted() {
    let addresses = [PUBLIC_ADDRESS, OLD_ADDRESS];
    let scenario = Scenario::default().publishing("home", &addresses);

    let report = scenario.check(&["home"]).expect("report");

    let expected = RecordOutcome {
        record: tracked("home"),
        outcome: Outcome::Drifted {
            current: published(&addresses),
        },
    };
    assert_eq!(vec![expected], report.records);
}

#[test]
fn should_report_failed_when_reading_fails_and_check_the_rest() {
    let error = DnsGatewayError::Unavailable("timeout".to_owned());
    let scenario = Scenario::default()
        .failing_to_read("home", error.clone())
        .publishing("lab", &[PUBLIC_ADDRESS]);

    let report = scenario.check(&["home", "lab"]).expect("report");

    let expected = vec![
        RecordOutcome {
            record: tracked("home"),
            outcome: Outcome::Failed(error),
        },
        RecordOutcome {
            record: tracked("lab"),
            outcome: Outcome::Unchanged,
        },
    ];
    assert_eq!(expected, report.records);
}

#[test]
fn should_stop_when_the_public_ip_is_unavailable() {
    let error = PublicIpGatewayError::Unavailable("offline".to_owned());
    let scenario = Scenario::default().without_public_ip(error.clone());

    assert_eq!(Err(error), scenario.check(&["home"]));
}

#[test]
fn should_never_point_a_drifted_record() {
    let scenario = Scenario::default().publishing("home", &[OLD_ADDRESS]);

    scenario.check(&["home"]).expect("report");

    assert!(scenario.pointed().is_empty());
}

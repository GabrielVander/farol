use lib_core::use_cases::{
    ports::{
        dns_gateway::{DnsGatewayError, PublishedRecord},
        public_ip_gateway::PublicIpGatewayError,
    },
    report::{Outcome, RecordOutcome},
};
use lib_core_testing::{
    fixtures::{OLD_ADDRESS, PUBLIC_ADDRESS, public_ip, published, tracked},
    scenario::Scenario,
};

#[test]
fn should_leave_an_unchanged_record_alone() {
    let scenario = Scenario::default().publishing("home", &[PUBLIC_ADDRESS]);

    let report = scenario.sync(&["home"]).expect("report");

    let expected = RecordOutcome {
        record: tracked("home"),
        outcome: Outcome::Unchanged,
    };
    assert_eq!(vec![expected], report.records);
    assert!(scenario.pointed().is_empty());
}

#[test]
fn should_point_a_drifted_record() {
    let scenario = Scenario::default().publishing("home", &[OLD_ADDRESS]);

    let report = scenario.sync(&["home"]).expect("report");

    let expected = RecordOutcome {
        record: tracked("home"),
        outcome: Outcome::Pointed {
            previous: published(&[OLD_ADDRESS]),
        },
    };
    assert_eq!(vec![expected], report.records);
    assert_eq!(vec![(tracked("home"), public_ip())], scenario.pointed());
}

#[test]
fn should_point_a_record_that_drifted_only_in_ttl() {
    let current = PublishedRecord {
        addresses: vec![PUBLIC_ADDRESS],
        ttl: Some(3600),
    };
    let scenario = Scenario::default().publishing_record("home", current.clone());

    let report = scenario.sync(&["home"]).expect("report");

    let expected = RecordOutcome {
        record: tracked("home"),
        outcome: Outcome::Pointed { previous: current },
    };
    assert_eq!(vec![expected], report.records);
    assert_eq!(vec![(tracked("home"), public_ip())], scenario.pointed());
}

#[test]
fn should_report_failed_when_pointing_fails_and_sync_the_rest() {
    let error = DnsGatewayError::Unavailable("rejected".to_owned());
    let scenario = Scenario::default()
        .publishing("home", &[OLD_ADDRESS])
        .failing_to_point("home", error.clone())
        .publishing("lab", &[OLD_ADDRESS]);

    let report = scenario.sync(&["home", "lab"]).expect("report");

    let expected = vec![
        RecordOutcome {
            record: tracked("home"),
            outcome: Outcome::Failed(error),
        },
        RecordOutcome {
            record: tracked("lab"),
            outcome: Outcome::Pointed {
                previous: published(&[OLD_ADDRESS]),
            },
        },
    ];
    assert_eq!(expected, report.records);
}

#[test]
fn should_report_failed_when_reading_fails() {
    let error = DnsGatewayError::Unavailable("timeout".to_owned());
    let scenario = Scenario::default().failing_to_read("home", error.clone());

    let report = scenario.sync(&["home"]).expect("report");

    let expected = RecordOutcome {
        record: tracked("home"),
        outcome: Outcome::Failed(error),
    };
    assert_eq!(vec![expected], report.records);
}

#[test]
fn should_stop_when_the_public_ip_is_unavailable() {
    let error = PublicIpGatewayError::Unavailable("offline".to_owned());
    let scenario = Scenario::default().without_public_ip(error.clone());

    assert_eq!(Err(error), scenario.sync(&["home"]));
    assert!(scenario.pointed().is_empty());
}

use std::sync::Arc;

use lib_core::use_cases::{
    check_records::{CheckRecords, CheckRecordsUseCase},
    ports::{
        dns_gateway::{DnsGatewayError, PublishedRecord},
        public_ip_gateway::PublicIpGatewayError,
    },
    report::{Outcome, RecordOutcome},
};
use lib_core_testing::{
    fakes::{FakeDnsGateway, FakePublicIpGateway},
    fixtures::{OLD_ADDRESS, PUBLIC_ADDRESS, nothing_pointed, published, tracked},
};

#[test]
fn should_report_unchanged_when_the_record_points_at_the_public_ip() {
    let dns_gateway = FakeDnsGateway::default().with_addresses("home", &[PUBLIC_ADDRESS]);
    let check_records = CheckRecords::new(
        Arc::new(FakePublicIpGateway::default()),
        Arc::new(dns_gateway),
    );

    let report = check_records.execute(&[tracked("home")]).expect("report");

    let expected = RecordOutcome {
        record: tracked("home"),
        outcome: Outcome::Unchanged,
    };
    assert_eq!(vec![expected], report.records);
}

#[test]
fn should_report_drifted_when_the_record_points_elsewhere() {
    let dns_gateway = FakeDnsGateway::default().with_addresses("home", &[OLD_ADDRESS]);
    let check_records = CheckRecords::new(
        Arc::new(FakePublicIpGateway::default()),
        Arc::new(dns_gateway),
    );

    let report = check_records.execute(&[tracked("home")]).expect("report");

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
    let dns_gateway = FakeDnsGateway::default().with_published("home", current.clone());
    let check_records = CheckRecords::new(
        Arc::new(FakePublicIpGateway::default()),
        Arc::new(dns_gateway),
    );

    let report = check_records.execute(&[tracked("home")]).expect("report");

    let expected = RecordOutcome {
        record: tracked("home"),
        outcome: Outcome::Drifted { current },
    };
    assert_eq!(vec![expected], report.records);
}

#[test]
fn should_report_a_missing_record_as_drifted() {
    let check_records = CheckRecords::new(
        Arc::new(FakePublicIpGateway::default()),
        Arc::new(FakeDnsGateway::default()),
    );

    let report = check_records.execute(&[tracked("home")]).expect("report");

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
    let dns_gateway = FakeDnsGateway::default().with_addresses("home", &addresses);
    let check_records = CheckRecords::new(
        Arc::new(FakePublicIpGateway::default()),
        Arc::new(dns_gateway),
    );

    let report = check_records.execute(&[tracked("home")]).expect("report");

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
    let dns_gateway = FakeDnsGateway::default()
        .with_read_failure("home", error.clone())
        .with_addresses("lab", &[PUBLIC_ADDRESS]);
    let check_records = CheckRecords::new(
        Arc::new(FakePublicIpGateway::default()),
        Arc::new(dns_gateway),
    );

    let report = check_records
        .execute(&[tracked("home"), tracked("lab")])
        .expect("report");

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
    let check_records = CheckRecords::new(
        Arc::new(FakePublicIpGateway(Err(error.clone()))),
        Arc::new(FakeDnsGateway::default()),
    );

    assert_eq!(Err(error), check_records.execute(&[tracked("home")]));
}

#[test]
fn should_never_point_a_drifted_record() {
    let dns_gateway = Arc::new(FakeDnsGateway::default().with_addresses("home", &[OLD_ADDRESS]));
    let check_records = CheckRecords::new(
        Arc::new(FakePublicIpGateway::default()),
        Arc::clone(&dns_gateway),
    );

    check_records.execute(&[tracked("home")]).expect("report");

    assert_eq!(nothing_pointed(), dns_gateway.pointed());
}

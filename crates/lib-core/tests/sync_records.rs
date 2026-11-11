use std::sync::Arc;

use lib_core::use_cases::{
    ports::{
        dns_gateway::{DnsGatewayError, PublishedRecord},
        public_ip_gateway::PublicIpGatewayError,
    },
    report::{Outcome, RecordOutcome},
    sync_records::{SyncRecords, SyncRecordsUseCase},
};
use lib_core_testing::{
    fakes::{FakeDnsGateway, FakePublicIpGateway},
    fixtures::{OLD_ADDRESS, PUBLIC_ADDRESS, nothing_pointed, public_ip, published, tracked},
};

#[test]
fn should_leave_an_unchanged_record_alone() {
    let dns_gateway = Arc::new(FakeDnsGateway::default().with_addresses("home", &[PUBLIC_ADDRESS]));
    let sync_records = SyncRecords::new(
        Arc::new(FakePublicIpGateway::default()),
        Arc::clone(&dns_gateway),
    );

    let report = sync_records.execute(&[tracked("home")]).expect("report");

    let expected = RecordOutcome {
        record: tracked("home"),
        outcome: Outcome::Unchanged,
    };
    assert_eq!(vec![expected], report.records);
    assert_eq!(nothing_pointed(), dns_gateway.pointed());
}

#[test]
fn should_point_a_drifted_record() {
    let dns_gateway = Arc::new(FakeDnsGateway::default().with_addresses("home", &[OLD_ADDRESS]));
    let sync_records = SyncRecords::new(
        Arc::new(FakePublicIpGateway::default()),
        Arc::clone(&dns_gateway),
    );

    let report = sync_records.execute(&[tracked("home")]).expect("report");

    let expected = RecordOutcome {
        record: tracked("home"),
        outcome: Outcome::Pointed {
            previous: published(&[OLD_ADDRESS]),
        },
    };
    assert_eq!(vec![expected], report.records);
    assert_eq!(vec![(tracked("home"), public_ip())], dns_gateway.pointed());
}

#[test]
fn should_point_a_record_that_drifted_only_in_ttl() {
    let current = PublishedRecord {
        addresses: vec![PUBLIC_ADDRESS],
        ttl: Some(3600),
    };
    let dns_gateway = Arc::new(FakeDnsGateway::default().with_published("home", current.clone()));
    let sync_records = SyncRecords::new(
        Arc::new(FakePublicIpGateway::default()),
        Arc::clone(&dns_gateway),
    );

    let report = sync_records.execute(&[tracked("home")]).expect("report");

    let expected = RecordOutcome {
        record: tracked("home"),
        outcome: Outcome::Pointed { previous: current },
    };
    assert_eq!(vec![expected], report.records);
    assert_eq!(vec![(tracked("home"), public_ip())], dns_gateway.pointed());
}

#[test]
fn should_report_failed_when_pointing_fails_and_sync_the_rest() {
    let error = DnsGatewayError::Unavailable("rejected".to_owned());
    let dns_gateway = FakeDnsGateway::default()
        .with_addresses("home", &[OLD_ADDRESS])
        .with_point_failure("home", error.clone())
        .with_addresses("lab", &[OLD_ADDRESS]);
    let sync_records = SyncRecords::new(
        Arc::new(FakePublicIpGateway::default()),
        Arc::new(dns_gateway),
    );

    let report = sync_records
        .execute(&[tracked("home"), tracked("lab")])
        .expect("report");

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
    let dns_gateway = FakeDnsGateway::default().with_read_failure("home", error.clone());
    let sync_records = SyncRecords::new(
        Arc::new(FakePublicIpGateway::default()),
        Arc::new(dns_gateway),
    );

    let report = sync_records.execute(&[tracked("home")]).expect("report");

    let expected = RecordOutcome {
        record: tracked("home"),
        outcome: Outcome::Failed(error),
    };
    assert_eq!(vec![expected], report.records);
}

#[test]
fn should_stop_when_the_public_ip_is_unavailable() {
    let error = PublicIpGatewayError::Unavailable("offline".to_owned());
    let dns_gateway = Arc::new(FakeDnsGateway::default());
    let sync_records = SyncRecords::new(
        Arc::new(FakePublicIpGateway::failing(error.clone())),
        Arc::clone(&dns_gateway),
    );

    assert_eq!(Err(error), sync_records.execute(&[tracked("home")]));
    assert_eq!(nothing_pointed(), dns_gateway.pointed());
}

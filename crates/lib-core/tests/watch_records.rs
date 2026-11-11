use std::sync::Arc;

use lib_core::use_cases::{
    report::{Outcome, RecordOutcome, Report},
    watch_records::{WatchRecords, WatchRecordsUseCase},
};
use lib_core_testing::{
    fakes::{FakeDnsGateway, FakePublicIpGateway},
    fixtures::{OLD_ADDRESS, next_public_ip, public_ip, published, tracked},
};

#[test]
fn should_sync_and_report_on_the_first_round() {
    let dns_gateway = Arc::new(FakeDnsGateway::default().with_addresses("home", &[OLD_ADDRESS]));
    let mut watch_records = WatchRecords::new(
        Arc::new(FakePublicIpGateway::default()),
        Arc::clone(&dns_gateway),
    );

    let report = watch_records.execute(&[tracked("home")]);

    let expected = Report {
        public_ip: public_ip(),
        records: vec![RecordOutcome {
            record: tracked("home"),
            outcome: Outcome::Pointed {
                previous: published(&[OLD_ADDRESS]),
            },
        }],
    };
    assert_eq!(Ok(Some(expected)), report);
    assert_eq!(vec![(tracked("home"), public_ip())], dns_gateway.pointed());
}

#[test]
fn should_skip_a_round_whose_public_ip_was_already_synced() {
    let dns_gateway = Arc::new(FakeDnsGateway::default().with_addresses("home", &[OLD_ADDRESS]));
    let mut watch_records = WatchRecords::new(
        Arc::new(FakePublicIpGateway::default()),
        Arc::clone(&dns_gateway),
    );
    watch_records
        .execute(&[tracked("home")])
        .expect("first round");

    let report = watch_records.execute(&[tracked("home")]);

    assert_eq!(Ok(None), report);
    assert_eq!(vec![(tracked("home"), public_ip())], dns_gateway.pointed());
}

#[test]
fn should_sync_again_when_the_public_ip_changes() {
    let dns_gateway = Arc::new(FakeDnsGateway::default().with_addresses("home", &[OLD_ADDRESS]));
    let mut watch_records = WatchRecords::new(
        Arc::new(FakePublicIpGateway::answering(vec![
            Ok(public_ip()),
            Ok(next_public_ip()),
        ])),
        Arc::clone(&dns_gateway),
    );
    watch_records
        .execute(&[tracked("home")])
        .expect("first round");

    let report = watch_records.execute(&[tracked("home")]);

    let report = report.expect("round").expect("a report");
    assert_eq!(next_public_ip(), report.public_ip);
    assert_eq!(
        vec![
            (tracked("home"), public_ip()),
            (tracked("home"), next_public_ip())
        ],
        dns_gateway.pointed()
    );
}

use std::sync::Arc;

use crate::{
    entities::public_ip::PublicIp,
    entities::tracked_record::TrackedRecord,
    use_cases::{
        check_records::{CheckRecords, CheckRecordsUseCase},
        dns_gateway::DnsGateway,
        public_ip_gateway::{PublicIpGateway, PublicIpGatewayError},
        report::{Outcome, RecordOutcome, Report},
    },
};

pub trait SyncRecordsUseCase {
    fn execute(&self, records: &[TrackedRecord]) -> Result<Report, PublicIpGatewayError>;
}

pub struct SyncRecords<P: PublicIpGateway, D: DnsGateway> {
    check: CheckRecords<P, D>,
    dns_gateway: Arc<D>,
}

impl<P: PublicIpGateway, D: DnsGateway> SyncRecords<P, D> {
    pub fn new(public_ip_gateway: Arc<P>, dns_gateway: Arc<D>) -> Self {
        Self {
            check: CheckRecords::new(public_ip_gateway, dns_gateway.clone()),
            dns_gateway,
        }
    }
}

impl<P: PublicIpGateway, D: DnsGateway> SyncRecordsUseCase for SyncRecords<P, D> {
    fn execute(&self, records: &[TrackedRecord]) -> Result<Report, PublicIpGatewayError> {
        let checked = self.check.execute(records)?;
        let public_ip = checked.public_ip;
        let records = checked
            .records
            .into_iter()
            .map(|record| self.sync(record, public_ip))
            .collect();
        Ok(Report { public_ip, records })
    }
}

impl<P: PublicIpGateway, D: DnsGateway> SyncRecords<P, D> {
    fn sync(&self, checked: RecordOutcome, public_ip: PublicIp) -> RecordOutcome {
        let outcome = match checked.outcome {
            Outcome::Drifted { current } => self
                .dns_gateway
                .point(&checked.record, public_ip)
                .map_or_else(Outcome::Failed, |()| Outcome::Pointed { previous: current }),
            other => other,
        };
        RecordOutcome {
            record: checked.record,
            outcome,
        }
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use crate::use_cases::{
        dns_gateway::DnsGatewayError,
        public_ip_gateway::PublicIpGatewayError,
        report::{Outcome, RecordOutcome},
        sync_records::{SyncRecords, SyncRecordsUseCase},
        test_support::{
            FakeDnsGateway, FakePublicIpGateway, OLD_ADDRESS, PUBLIC_ADDRESS, public_ip, published,
            tracked,
        },
    };

    #[test]
    fn should_leave_an_unchanged_record_alone() {
        let dns_gateway =
            Arc::new(FakeDnsGateway::default().with_addresses("home", &[PUBLIC_ADDRESS]));
        let sync = SyncRecords::new(
            Arc::new(FakePublicIpGateway(Ok(public_ip()))),
            dns_gateway.clone(),
        );

        let report = sync.execute(&[tracked("home")]).expect("report");

        let expected = RecordOutcome {
            record: tracked("home"),
            outcome: Outcome::Unchanged,
        };
        assert_eq!(vec![expected], report.records);
        assert!(dns_gateway.pointed().is_empty());
    }

    #[test]
    fn should_point_a_drifted_record() {
        let dns_gateway =
            Arc::new(FakeDnsGateway::default().with_addresses("home", &[OLD_ADDRESS]));
        let sync = SyncRecords::new(
            Arc::new(FakePublicIpGateway(Ok(public_ip()))),
            dns_gateway.clone(),
        );

        let report = sync.execute(&[tracked("home")]).expect("report");

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
    fn should_report_failed_when_pointing_fails_and_sync_the_rest() {
        let error = DnsGatewayError::Unavailable("rejected".to_owned());
        let dns_gateway = FakeDnsGateway::default()
            .with_addresses("home", &[OLD_ADDRESS])
            .with_point_failure("home", error.clone())
            .with_addresses("lab", &[OLD_ADDRESS]);
        let sync = SyncRecords::new(
            Arc::new(FakePublicIpGateway(Ok(public_ip()))),
            Arc::new(dns_gateway),
        );

        let report = sync
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
        let sync = SyncRecords::new(
            Arc::new(FakePublicIpGateway(Ok(public_ip()))),
            Arc::new(dns_gateway),
        );

        let report = sync.execute(&[tracked("home")]).expect("report");

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
        let sync = SyncRecords::new(
            Arc::new(FakePublicIpGateway(Err(error.clone()))),
            dns_gateway.clone(),
        );

        assert_eq!(Err(error), sync.execute(&[tracked("home")]));
        assert!(dns_gateway.pointed().is_empty());
    }
}

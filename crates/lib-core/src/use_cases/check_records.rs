use std::net::Ipv4Addr;
use std::sync::Arc;

use crate::{
    entities::{public_ip::PublicIp, tracked_record::TrackedRecord},
    use_cases::{
        dns_gateway::DnsGateway,
        public_ip_gateway::{PublicIpGateway, PublicIpGatewayError},
        report::{Outcome, RecordOutcome, Report},
    },
};

pub trait CheckRecordsUseCase {
    fn execute(&self, records: &[TrackedRecord]) -> Result<Report, PublicIpGatewayError>;
}

pub struct CheckRecords<P: PublicIpGateway, D: DnsGateway> {
    public_ip_gateway: Arc<P>,
    dns_gateway: Arc<D>,
}

impl<P: PublicIpGateway, D: DnsGateway> CheckRecords<P, D> {
    pub fn new(public_ip_gateway: Arc<P>, dns_gateway: Arc<D>) -> Self {
        Self {
            public_ip_gateway,
            dns_gateway,
        }
    }
}

impl<P: PublicIpGateway, D: DnsGateway> CheckRecordsUseCase for CheckRecords<P, D> {
    fn execute(&self, records: &[TrackedRecord]) -> Result<Report, PublicIpGatewayError> {
        let public_ip = self.public_ip_gateway.current_public_ip()?;
        let records = records
            .iter()
            .map(|record| self.check(record, public_ip))
            .collect();
        Ok(Report { records })
    }
}

impl<P: PublicIpGateway, D: DnsGateway> CheckRecords<P, D> {
    fn check(&self, record: &TrackedRecord, public_ip: PublicIp) -> RecordOutcome {
        let outcome = match self.dns_gateway.addresses_of(record) {
            Ok(addresses) => outcome_of(addresses, public_ip),
            Err(error) => Outcome::Failed(error),
        };
        RecordOutcome {
            record: record.clone(),
            outcome,
        }
    }
}

fn outcome_of(addresses: Vec<Ipv4Addr>, public_ip: PublicIp) -> Outcome {
    if addresses == [public_ip.address()] {
        return Outcome::Unchanged;
    }
    Outcome::Drifted { addresses }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use crate::{
        use_cases::report::{Outcome, RecordOutcome},
        use_cases::{
            check_records::{CheckRecords, CheckRecordsUseCase},
            dns_gateway::DnsGatewayError,
            public_ip_gateway::PublicIpGatewayError,
            test_support::{
                FakeDnsGateway, FakePublicIpGateway, OLD_ADDRESS, PUBLIC_ADDRESS, public_ip,
                tracked,
            },
        },
    };

    #[test]
    fn should_report_unchanged_when_the_record_points_at_the_public_ip() {
        let dns_gateway = FakeDnsGateway::default().with_addresses("home", &[PUBLIC_ADDRESS]);
        let check = CheckRecords::new(
            Arc::new(FakePublicIpGateway(Ok(public_ip()))),
            Arc::new(dns_gateway),
        );

        let report = check.execute(&[tracked("home")]).expect("report");

        let expected = RecordOutcome {
            record: tracked("home"),
            outcome: Outcome::Unchanged,
        };
        assert_eq!(vec![expected], report.records);
    }

    #[test]
    fn should_report_drifted_when_the_record_points_elsewhere() {
        let dns_gateway = FakeDnsGateway::default().with_addresses("home", &[OLD_ADDRESS]);
        let check = CheckRecords::new(
            Arc::new(FakePublicIpGateway(Ok(public_ip()))),
            Arc::new(dns_gateway),
        );

        let report = check.execute(&[tracked("home")]).expect("report");

        let expected = RecordOutcome {
            record: tracked("home"),
            outcome: Outcome::Drifted {
                addresses: vec![OLD_ADDRESS],
            },
        };
        assert_eq!(vec![expected], report.records);
    }

    #[test]
    fn should_report_a_missing_record_as_drifted() {
        let check = CheckRecords::new(
            Arc::new(FakePublicIpGateway(Ok(public_ip()))),
            Arc::new(FakeDnsGateway::default()),
        );

        let report = check.execute(&[tracked("home")]).expect("report");

        let expected = RecordOutcome {
            record: tracked("home"),
            outcome: Outcome::Drifted { addresses: vec![] },
        };
        assert_eq!(vec![expected], report.records);
    }

    #[test]
    fn should_report_several_addresses_as_drifted() {
        let addresses = [PUBLIC_ADDRESS, OLD_ADDRESS];
        let dns_gateway = FakeDnsGateway::default().with_addresses("home", &addresses);
        let check = CheckRecords::new(
            Arc::new(FakePublicIpGateway(Ok(public_ip()))),
            Arc::new(dns_gateway),
        );

        let report = check.execute(&[tracked("home")]).expect("report");

        let expected = RecordOutcome {
            record: tracked("home"),
            outcome: Outcome::Drifted {
                addresses: addresses.to_vec(),
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
        let check = CheckRecords::new(
            Arc::new(FakePublicIpGateway(Ok(public_ip()))),
            Arc::new(dns_gateway),
        );

        let report = check
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
        let check = CheckRecords::new(
            Arc::new(FakePublicIpGateway(Err(error.clone()))),
            Arc::new(FakeDnsGateway::default()),
        );

        assert_eq!(Err(error), check.execute(&[tracked("home")]));
    }
}

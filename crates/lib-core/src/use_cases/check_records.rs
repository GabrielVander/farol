use std::net::Ipv4Addr;
use std::sync::Arc;

use crate::{
    entities::{
        public_ip::PublicIp,
        report::{Outcome, RecordOutcome, Report},
        tracked_record::TrackedRecord,
    },
    use_cases::{
        dns_gateway::DnsGateway,
        public_ip_gateway::{PublicIpGateway, PublicIpGatewayError},
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
        let addresses = self.dns_gateway.addresses_of(record).unwrap_or_default();
        RecordOutcome {
            record: record.clone(),
            outcome: outcome_of(addresses, public_ip),
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
        entities::report::{Outcome, RecordOutcome},
        use_cases::{
            check_records::{CheckRecords, CheckRecordsUseCase},
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
}

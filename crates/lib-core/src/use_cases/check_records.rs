use std::sync::Arc;

use crate::{
    entities::{public_ip::PublicIp, tracked_record::TrackedRecord},
    use_cases::{
        dns_gateway::{DnsGateway, PublishedRecord},
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
        Ok(Report { public_ip, records })
    }
}

impl<P: PublicIpGateway, D: DnsGateway> CheckRecords<P, D> {
    fn check(&self, record: &TrackedRecord, public_ip: PublicIp) -> RecordOutcome {
        let outcome = match self.dns_gateway.published(record) {
            Ok(current) => outcome_of(current, record, public_ip),
            Err(error) => Outcome::Failed(error),
        };
        RecordOutcome {
            record: record.clone(),
            outcome,
        }
    }
}

fn outcome_of(current: PublishedRecord, record: &TrackedRecord, public_ip: PublicIp) -> Outcome {
    if is_up_to_date(&current, record, public_ip) {
        return Outcome::Unchanged;
    }
    Outcome::Drifted { current }
}

fn is_up_to_date(current: &PublishedRecord, record: &TrackedRecord, public_ip: PublicIp) -> bool {
    current.addresses == [public_ip.address()] && current.ttl == Some(record.ttl.seconds())
}

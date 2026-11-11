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

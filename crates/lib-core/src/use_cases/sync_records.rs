use std::sync::Arc;

use crate::{
    entities::{public_ip::PublicIp, tracked_record::TrackedRecord},
    use_cases::{
        ports::{
            dns_gateway::{DnsGateway, PublishedRecord},
            public_ip_gateway::{PublicIpGateway, PublicIpGatewayError},
        },
        report::{Outcome, RecordOutcome, Report},
    },
};

pub trait SyncRecordsUseCase {
    fn execute(&self, records: &[TrackedRecord]) -> Result<Report, PublicIpGatewayError>;
}

impl<P: PublicIpGateway, D: DnsGateway> SyncRecordsUseCase for SyncRecords<P, D> {
    fn execute(&self, records: &[TrackedRecord]) -> Result<Report, PublicIpGatewayError> {
        let public_ip = self.public_ip_gateway.current_public_ip()?;

        Ok(self.sync_to(records, public_ip))
    }
}

pub struct SyncRecords<P: PublicIpGateway, D: DnsGateway> {
    public_ip_gateway: Arc<P>,
    dns_gateway: Arc<D>,
}

impl<P: PublicIpGateway, D: DnsGateway> SyncRecords<P, D> {
    pub fn new(public_ip_gateway: Arc<P>, dns_gateway: Arc<D>) -> Self {
        Self {
            public_ip_gateway,
            dns_gateway,
        }
    }

    /// Syncs `records` to a public IP the caller already knows.
    pub(crate) fn sync_to(&self, records: &[TrackedRecord], public_ip: PublicIp) -> Report {
        let records = records
            .iter()
            .map(|record| self.sync(record, public_ip))
            .collect();

        Report { public_ip, records }
    }

    fn sync(&self, record: &TrackedRecord, public_ip: PublicIp) -> RecordOutcome {
        let outcome = match self.dns_gateway.published(record) {
            Ok(current) if current.is_up_to_date(record, public_ip) => Outcome::Unchanged,
            Ok(current) => self.point(record, public_ip, current),
            Err(error) => Outcome::Failed(error),
        };

        RecordOutcome {
            record: record.clone(),
            outcome,
        }
    }

    fn point(
        &self,
        record: &TrackedRecord,
        public_ip: PublicIp,
        current: PublishedRecord,
    ) -> Outcome {
        self.dns_gateway
            .point(record, public_ip)
            .map_or_else(Outcome::Failed, |()| Outcome::Pointed { previous: current })
    }
}

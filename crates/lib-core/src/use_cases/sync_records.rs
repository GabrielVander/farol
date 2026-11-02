use std::sync::Arc;

use crate::{
    entities::tracked_record::TrackedRecord,
    use_cases::{
        check_records::{CheckRecords, CheckRecordsUseCase},
        dns_gateway::DnsGateway,
        public_ip_gateway::{PublicIpGateway, PublicIpGatewayError},
        report::Report,
    },
};

pub trait SyncRecordsUseCase {
    fn execute(&self, records: &[TrackedRecord]) -> Result<Report, PublicIpGatewayError>;
}

pub struct SyncRecords<P: PublicIpGateway, D: DnsGateway> {
    check: CheckRecords<P, D>,
}

impl<P: PublicIpGateway, D: DnsGateway> SyncRecords<P, D> {
    pub fn new(public_ip_gateway: Arc<P>, dns_gateway: Arc<D>) -> Self {
        Self {
            check: CheckRecords::new(public_ip_gateway, dns_gateway),
        }
    }
}

impl<P: PublicIpGateway, D: DnsGateway> SyncRecordsUseCase for SyncRecords<P, D> {
    fn execute(&self, records: &[TrackedRecord]) -> Result<Report, PublicIpGatewayError> {
        self.check.execute(records)
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use crate::use_cases::{
        report::{Outcome, RecordOutcome},
        sync_records::{SyncRecords, SyncRecordsUseCase},
        test_support::{FakeDnsGateway, FakePublicIpGateway, PUBLIC_ADDRESS, public_ip, tracked},
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
}

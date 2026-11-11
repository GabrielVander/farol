use std::sync::Arc;

use crate::{
    entities::{public_ip::PublicIp, tracked_record::TrackedRecord},
    use_cases::{
        ports::{
            dns_gateway::DnsGateway,
            public_ip_gateway::{PublicIpGateway, PublicIpGatewayError},
        },
        report::Report,
        sync_records::SyncRecords,
    },
};

pub trait WatchRecordsUseCase {
    /// Runs one round of the watch: syncs `records` unless the public IP is the
    /// one the last round synced without failures, which gives `None`.
    fn execute(
        &mut self,
        records: &[TrackedRecord],
    ) -> Result<Option<Report>, PublicIpGatewayError>;
}

impl<P: PublicIpGateway, D: DnsGateway> WatchRecordsUseCase for WatchRecords<P, D> {
    fn execute(
        &mut self,
        records: &[TrackedRecord],
    ) -> Result<Option<Report>, PublicIpGatewayError> {
        let public_ip = self.public_ip_gateway.current_public_ip()?;

        if self.last_synced == Some(public_ip) {
            return Ok(None);
        }

        let report = self.sync_records.sync_to(records, public_ip);
        self.last_synced = (!report.has_failures()).then_some(public_ip);

        Ok(Some(report))
    }
}

pub struct WatchRecords<P: PublicIpGateway, D: DnsGateway> {
    public_ip_gateway: Arc<P>,
    sync_records: SyncRecords<P, D>,
    last_synced: Option<PublicIp>,
}

impl<P: PublicIpGateway, D: DnsGateway> WatchRecords<P, D> {
    pub fn new(public_ip_gateway: Arc<P>, dns_gateway: Arc<D>) -> Self {
        Self {
            sync_records: SyncRecords::new(Arc::clone(&public_ip_gateway), dns_gateway),
            public_ip_gateway,
            last_synced: None,
        }
    }
}

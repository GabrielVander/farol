use std::{net::Ipv4Addr, sync::Arc};

use lib_core::{
    entities::{public_ip::PublicIp, tracked_record::TrackedRecord},
    use_cases::{
        check_records::{CheckRecords, CheckRecordsUseCase},
        dns_gateway::{DnsGatewayError, PublishedRecord},
        public_ip_gateway::PublicIpGatewayError,
        report::Report,
        sync_records::{SyncRecords, SyncRecordsUseCase},
    },
};

use crate::{
    fakes::{FakeDnsGateway, FakePublicIpGateway},
    fixtures::{public_ip, tracked},
};

/// A world farol runs in: a public IP and a DNS provider, both fake. Tests say
/// what the world holds and what farol does, never how the use cases are built.
#[derive(Debug, Clone)]
pub struct Scenario {
    public_ip_gateway: FakePublicIpGateway,
    dns_gateway: FakeDnsGateway,
}

impl Default for Scenario {
    fn default() -> Self {
        Self {
            public_ip_gateway: FakePublicIpGateway(Ok(public_ip())),
            dns_gateway: FakeDnsGateway::default(),
        }
    }
}

impl Scenario {
    #[must_use]
    pub fn without_public_ip(mut self, error: PublicIpGatewayError) -> Self {
        self.public_ip_gateway = FakePublicIpGateway(Err(error));
        self
    }

    #[must_use]
    pub fn publishing(mut self, name: &str, addresses: &[Ipv4Addr]) -> Self {
        self.dns_gateway = self.dns_gateway.with_addresses(name, addresses);
        self
    }

    #[must_use]
    pub fn publishing_record(mut self, name: &str, record: PublishedRecord) -> Self {
        self.dns_gateway = self.dns_gateway.with_published(name, record);
        self
    }

    #[must_use]
    pub fn failing_to_read(mut self, name: &str, error: DnsGatewayError) -> Self {
        self.dns_gateway = self.dns_gateway.with_read_failure(name, error);
        self
    }

    #[must_use]
    pub fn failing_to_point(mut self, name: &str, error: DnsGatewayError) -> Self {
        self.dns_gateway = self.dns_gateway.with_point_failure(name, error);
        self
    }

    /// Checks the named records of the fixture domain.
    pub fn check(&self, names: &[&str]) -> Result<Report, PublicIpGatewayError> {
        CheckRecords::new(self.public_ip_gateway(), self.dns_gateway()).execute(&records(names))
    }

    /// Syncs the named records of the fixture domain.
    pub fn sync(&self, names: &[&str]) -> Result<Report, PublicIpGatewayError> {
        SyncRecords::new(self.public_ip_gateway(), self.dns_gateway()).execute(&records(names))
    }

    /// Every record farol pointed, in order, with the IP it pointed it at.
    pub fn pointed(&self) -> Vec<(TrackedRecord, PublicIp)> {
        self.dns_gateway.pointed()
    }

    /// The fake public IP service, for wiring into a whole application.
    pub fn public_ip_gateway(&self) -> Arc<FakePublicIpGateway> {
        Arc::new(self.public_ip_gateway.clone())
    }

    /// The fake DNS provider, for wiring into a whole application. It shares
    /// what was pointed with this scenario.
    pub fn dns_gateway(&self) -> Arc<FakeDnsGateway> {
        Arc::new(self.dns_gateway.clone())
    }
}

fn records(names: &[&str]) -> Vec<TrackedRecord> {
    names.iter().map(|name| tracked(name)).collect()
}

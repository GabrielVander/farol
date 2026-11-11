use std::{collections::HashMap, net::Ipv4Addr, sync::Mutex};

use lib_core::{
    entities::{public_ip::PublicIp, tracked_record::TrackedRecord},
    use_cases::ports::{
        dns_gateway::{DnsGateway, DnsGatewayError, PublishedRecord},
        public_ip_gateway::{PublicIpGateway, PublicIpGatewayError},
    },
};

use crate::fixtures::{public_ip, published};

#[derive(Debug, Clone)]
pub struct FakePublicIpGateway(pub Result<PublicIp, PublicIpGatewayError>);

impl Default for FakePublicIpGateway {
    fn default() -> Self {
        Self(Ok(public_ip()))
    }
}

impl PublicIpGateway for FakePublicIpGateway {
    fn current_public_ip(&self) -> Result<PublicIp, PublicIpGatewayError> {
        self.0.clone()
    }
}

/// A DNS provider holding records by name.
#[derive(Debug, Default)]
pub struct FakeDnsGateway {
    published: HashMap<String, Result<PublishedRecord, DnsGatewayError>>,
    point_failures: HashMap<String, DnsGatewayError>,
    pointed: Mutex<Vec<(TrackedRecord, PublicIp)>>,
}

impl FakeDnsGateway {
    #[must_use]
    pub fn with_addresses(self, name: &str, addresses: &[Ipv4Addr]) -> Self {
        self.with_published(name, published(addresses))
    }

    #[must_use]
    pub fn with_published(mut self, name: &str, record: PublishedRecord) -> Self {
        self.published.insert(name.to_owned(), Ok(record));
        self
    }

    #[must_use]
    pub fn with_read_failure(mut self, name: &str, error: DnsGatewayError) -> Self {
        self.published.insert(name.to_owned(), Err(error));
        self
    }

    #[must_use]
    pub fn with_point_failure(mut self, name: &str, error: DnsGatewayError) -> Self {
        self.point_failures.insert(name.to_owned(), error);
        self
    }

    pub fn pointed(&self) -> Vec<(TrackedRecord, PublicIp)> {
        self.pointed.lock().expect("unpoisoned").clone()
    }
}

impl DnsGateway for FakeDnsGateway {
    fn published(&self, record: &TrackedRecord) -> Result<PublishedRecord, DnsGatewayError> {
        let stored = self.published.get(record.name.as_str()).cloned();
        stored.unwrap_or_else(|| Ok(published(&[])))
    }

    fn point(&self, record: &TrackedRecord, public_ip: PublicIp) -> Result<(), DnsGatewayError> {
        if let Some(error) = self.point_failures.get(record.name.as_str()) {
            return Err(error.clone());
        }

        let mut pointed = self.pointed.lock().expect("unpoisoned");
        pointed.push((record.clone(), public_ip));

        Ok(())
    }
}

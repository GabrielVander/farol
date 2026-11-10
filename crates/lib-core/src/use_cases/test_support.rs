use std::{collections::HashMap, net::Ipv4Addr, sync::Mutex};

use crate::{
    entities::{
        domain_name::DomainName, public_ip::PublicIp, record_name::RecordName,
        tracked_record::TrackedRecord, ttl::Ttl,
    },
    use_cases::{
        dns_gateway::{DnsGateway, DnsGatewayError, PublishedRecord},
        public_ip_gateway::{PublicIpGateway, PublicIpGatewayError},
    },
};

pub(crate) const PUBLIC_ADDRESS: Ipv4Addr = Ipv4Addr::new(8, 8, 8, 8);
pub(crate) const OLD_ADDRESS: Ipv4Addr = Ipv4Addr::new(1, 1, 1, 1);
const TRACKED_TTL: u32 = 300;

pub(crate) fn public_ip() -> PublicIp {
    PublicIp::try_from(PUBLIC_ADDRESS).expect("public ip")
}

pub(crate) fn tracked(name: &str) -> TrackedRecord {
    TrackedRecord {
        domain: DomainName::try_from("example.com").expect("valid domain"),
        name: RecordName::try_from(name).expect("valid record name"),
        ttl: Ttl::try_from(TRACKED_TTL).expect("valid ttl"),
    }
}

/// A published record with the TTL `tracked` asks for, or none without addresses.
pub(crate) fn published(addresses: &[Ipv4Addr]) -> PublishedRecord {
    PublishedRecord {
        addresses: addresses.to_vec(),
        ttl: (!addresses.is_empty()).then_some(TRACKED_TTL),
    }
}

pub(crate) struct FakePublicIpGateway(pub Result<PublicIp, PublicIpGatewayError>);

impl PublicIpGateway for FakePublicIpGateway {
    fn current_public_ip(&self) -> Result<PublicIp, PublicIpGatewayError> {
        self.0.clone()
    }
}

#[derive(Default)]
pub(crate) struct FakeDnsGateway {
    published: HashMap<String, Result<PublishedRecord, DnsGatewayError>>,
    point_failures: HashMap<String, DnsGatewayError>,
    pointed: Mutex<Vec<(TrackedRecord, PublicIp)>>,
}

impl FakeDnsGateway {
    pub(crate) fn with_addresses(mut self, name: &str, addresses: &[Ipv4Addr]) -> Self {
        self.published
            .insert(name.to_owned(), Ok(published(addresses)));
        self
    }

    pub(crate) fn with_read_failure(mut self, name: &str, error: DnsGatewayError) -> Self {
        self.published.insert(name.to_owned(), Err(error));
        self
    }

    pub(crate) fn with_point_failure(mut self, name: &str, error: DnsGatewayError) -> Self {
        self.point_failures.insert(name.to_owned(), error);
        self
    }

    pub(crate) fn pointed(&self) -> Vec<(TrackedRecord, PublicIp)> {
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

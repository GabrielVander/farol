use std::net::Ipv4Addr;

use lib_core::{
    entities::{
        domain_name::DomainName, public_ip::PublicIp, record_name::RecordName,
        tracked_record::TrackedRecord, ttl::Ttl,
    },
    use_cases::ports::dns_gateway::PublishedRecord,
};

pub const PUBLIC_ADDRESS: Ipv4Addr = Ipv4Addr::new(8, 8, 8, 8);
pub const OLD_ADDRESS: Ipv4Addr = Ipv4Addr::new(1, 1, 1, 1);
pub const DOMAIN: &str = "example.com";
pub const TRACKED_TTL: u32 = 300;

pub fn public_ip() -> PublicIp {
    PublicIp::try_from(PUBLIC_ADDRESS).expect("public ip")
}

/// A record of `DOMAIN` tracked with `TRACKED_TTL`.
pub fn tracked(name: &str) -> TrackedRecord {
    TrackedRecord {
        domain: DomainName::try_from(DOMAIN).expect("valid domain"),
        name: RecordName::try_from(name).expect("valid record name"),
        ttl: Ttl::try_from(TRACKED_TTL).expect("valid ttl"),
    }
}

/// A published record with the TTL `tracked` asks for, or none without addresses.
pub fn published(addresses: &[Ipv4Addr]) -> PublishedRecord {
    PublishedRecord {
        addresses: addresses.to_vec(),
        ttl: (!addresses.is_empty()).then_some(TRACKED_TTL),
    }
}

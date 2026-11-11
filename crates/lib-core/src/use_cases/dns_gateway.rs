use std::net::Ipv4Addr;

use crate::entities::{public_ip::PublicIp, tracked_record::TrackedRecord};

pub trait DnsGateway {
    fn published(&self, record: &TrackedRecord) -> Result<PublishedRecord, DnsGatewayError>;

    fn point(&self, record: &TrackedRecord, public_ip: PublicIp) -> Result<(), DnsGatewayError>;
}

/// What a DNS provider currently serves for a record. `ttl` is in seconds, and is
/// `None` when no A record exists. It stays a plain number because the provider
/// may hold a value farol would not accept as a `Ttl`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishedRecord {
    pub addresses: Vec<Ipv4Addr>,
    pub ttl: Option<u32>,
}

impl PublishedRecord {
    /// Whether the provider already serves `record` exactly as farol would point it:
    /// the public IP alone, with the tracked TTL.
    pub fn is_up_to_date(&self, record: &TrackedRecord, public_ip: PublicIp) -> bool {
        self.addresses == [public_ip.address()] && self.ttl == Some(record.ttl.seconds())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DnsGatewayError {
    Unavailable(String),
    InvalidAnswer(String),
    Unauthorized(String),
    DomainNotFound(String),
    Rejected(String),
    RateLimited(String),
}

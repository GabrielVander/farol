use crate::entities::{domain_name::DomainName, record_name::RecordName, ttl::Ttl};

/// A record farol keeps pointed at the public IP.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrackedRecord {
    pub domain: DomainName,
    pub name: RecordName,
    pub ttl: Ttl,
}

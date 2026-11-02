use std::net::Ipv4Addr;

use crate::entities::{public_ip::PublicIp, tracked_record::TrackedRecord};

pub trait DnsGateway {
    fn addresses_of(&self, record: &TrackedRecord) -> Result<Vec<Ipv4Addr>, DnsGatewayError>;

    fn point(&self, record: &TrackedRecord, public_ip: PublicIp) -> Result<(), DnsGatewayError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DnsGatewayError {
    Unavailable(String),
}

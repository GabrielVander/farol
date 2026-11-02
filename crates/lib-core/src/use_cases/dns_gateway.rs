use std::net::Ipv4Addr;

use crate::entities::tracked_record::TrackedRecord;

pub trait DnsGateway {
    fn addresses_of(&self, record: &TrackedRecord) -> Result<Vec<Ipv4Addr>, DnsGatewayError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DnsGatewayError {
    Unavailable(String),
}

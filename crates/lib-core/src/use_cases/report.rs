use std::net::Ipv4Addr;

use crate::{
    entities::{public_ip::PublicIp, tracked_record::TrackedRecord},
    use_cases::dns_gateway::DnsGatewayError,
};

/// What a check or a sync found for every tracked record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    pub public_ip: PublicIp,
    pub records: Vec<RecordOutcome>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordOutcome {
    pub record: TrackedRecord,
    pub outcome: Outcome,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    Unchanged,
    Drifted { addresses: Vec<Ipv4Addr> },
    Failed(DnsGatewayError),
}

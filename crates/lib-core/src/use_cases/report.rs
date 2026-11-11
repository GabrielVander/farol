use crate::{
    entities::{public_ip::PublicIp, tracked_record::TrackedRecord},
    use_cases::ports::dns_gateway::{DnsGatewayError, PublishedRecord},
};

/// What a check or a sync found for every tracked record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    pub public_ip: PublicIp,
    pub records: Vec<RecordOutcome>,
}

impl Report {
    /// Whether any tracked record failed.
    pub fn has_failures(&self) -> bool {
        self.records
            .iter()
            .any(|record_outcome| matches!(record_outcome.outcome, Outcome::Failed(_)))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordOutcome {
    pub record: TrackedRecord,
    pub outcome: Outcome,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    Unchanged,
    Drifted { current: PublishedRecord },
    Pointed { previous: PublishedRecord },
    Failed(DnsGatewayError),
}

use std::net::Ipv4Addr;

use crate::entities::tracked_record::TrackedRecord;

/// What a check or a sync found for every tracked record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
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
}

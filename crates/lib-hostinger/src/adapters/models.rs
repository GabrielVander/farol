use serde::{Deserialize, Serialize};

/// One entry of a Hostinger DNS zone: every record of one name and type.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct RecordSet {
    pub name: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub ttl: u32,
    pub records: Vec<RecordValue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct RecordValue {
    pub content: String,
    #[serde(default, skip_serializing)]
    pub is_disabled: bool,
}

/// Replaces (with `overwrite`) or adds the given record sets in a zone.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UpdateZoneRequest {
    pub overwrite: bool,
    pub zone: Vec<RecordSet>,
}

/// The body Hostinger sends with an error status.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ErrorBody {
    pub message: String,
    pub correlation_id: Option<String>,
}

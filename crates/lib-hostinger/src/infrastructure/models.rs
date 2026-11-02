/// One entry of a Hostinger DNS zone: every record of one name and type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordSet {
    pub name: String,
    pub kind: String,
    pub ttl: u32,
    pub records: Vec<RecordValue>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordValue {
    pub content: String,
    pub is_disabled: bool,
}

/// Replaces (with `overwrite`) or adds the given record sets in a zone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateZoneRequest {
    pub overwrite: bool,
    pub zone: Vec<RecordSet>,
}

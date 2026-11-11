use crate::adapters::models::{RecordSet, UpdateZoneRequest};

/// The parts of Hostinger's DNS API that farol uses.
pub trait HostingerApi {
    fn zone(&self, domain: &str) -> Result<Vec<RecordSet>, HostingerApiError>;

    fn update_zone(
        &self,
        domain: &str,
        request: &UpdateZoneRequest,
    ) -> Result<(), HostingerApiError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostingerApiError {
    Unreachable(String),
    InvalidBody(String),
    Status {
        status: u16,
        message: String,
        correlation_id: Option<String>,
    },
}

/// ipify's plain-text answer to "what is my IP?".
pub trait IpifyApi {
    fn fetch_answer(&self) -> Result<String, IpifyApiError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IpifyApiError {
    Unreachable(String),
}

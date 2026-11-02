use crate::entities::public_ip::PublicIp;

pub trait PublicIpGateway {
    fn current_public_ip(&self) -> Result<PublicIp, PublicIpGatewayError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PublicIpGatewayError {
    Unavailable(String),
    InvalidAnswer(String),
}

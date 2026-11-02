use std::net::Ipv4Addr;

/// The IPv4 address the internet sees for this machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PublicIp(Ipv4Addr);

impl PublicIp {
    pub fn address(&self) -> Ipv4Addr {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PublicIpError {
    NotPublic(Ipv4Addr),
}

impl TryFrom<Ipv4Addr> for PublicIp {
    type Error = PublicIpError;

    fn try_from(address: Ipv4Addr) -> Result<Self, Self::Error> {
        if address.is_private() {
            return Err(PublicIpError::NotPublic(address));
        }
        Ok(Self(address))
    }
}

#[cfg(test)]
mod test {
    use std::net::Ipv4Addr;

    use crate::entities::public_ip::{PublicIp, PublicIpError};

    #[test]
    fn should_accept_a_global_address() {
        let address = Ipv4Addr::new(8, 8, 8, 8);

        let ip = PublicIp::try_from(address).expect("public ip");

        assert_eq!(address, ip.address());
    }

    #[test]
    fn should_reject_a_private_address() {
        let address = Ipv4Addr::new(192, 168, 1, 10);

        assert_eq!(
            Err(PublicIpError::NotPublic(address)),
            PublicIp::try_from(address)
        );
    }
}

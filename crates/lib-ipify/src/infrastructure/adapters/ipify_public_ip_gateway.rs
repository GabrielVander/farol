use std::net::Ipv4Addr;

use lib_core::{
    entities::public_ip::PublicIp,
    use_cases::public_ip_gateway::{PublicIpGateway, PublicIpGatewayError},
};

use crate::infrastructure::adapters::ipify_api::IpifyApi;

pub struct IpifyPublicIpGateway<A: IpifyApi> {
    api: A,
}

impl<A: IpifyApi> IpifyPublicIpGateway<A> {
    pub fn new(api: A) -> Self {
        Self { api }
    }
}

impl<A: IpifyApi> PublicIpGateway for IpifyPublicIpGateway<A> {
    fn current_public_ip(&self) -> Result<PublicIp, PublicIpGatewayError> {
        let answer = self
            .api
            .fetch_answer()
            .map_err(|error| PublicIpGatewayError::Unavailable(format!("{error:?}")))?;
        let address: Ipv4Addr = answer
            .parse()
            .map_err(|_| PublicIpGatewayError::Unavailable(answer.clone()))?;
        PublicIp::try_from(address)
            .map_err(|error| PublicIpGatewayError::Unavailable(format!("{error:?}")))
    }
}

#[cfg(test)]
mod test {
    use std::net::Ipv4Addr;

    use lib_core::use_cases::public_ip_gateway::PublicIpGateway;

    use crate::infrastructure::adapters::{
        ipify_api::{IpifyApi, IpifyApiError},
        ipify_public_ip_gateway::IpifyPublicIpGateway,
    };

    struct FakeIpifyApi(Result<String, IpifyApiError>);

    impl IpifyApi for FakeIpifyApi {
        fn fetch_answer(&self) -> Result<String, IpifyApiError> {
            self.0.clone()
        }
    }

    fn gateway_answering(answer: &str) -> IpifyPublicIpGateway<FakeIpifyApi> {
        IpifyPublicIpGateway::new(FakeIpifyApi(Ok(answer.to_owned())))
    }

    #[test]
    fn should_read_the_public_ip_from_the_answer() {
        let public_ip = gateway_answering("8.8.8.8")
            .current_public_ip()
            .expect("public ip");

        assert_eq!(Ipv4Addr::new(8, 8, 8, 8), public_ip.address());
    }
}

use std::net::Ipv4Addr;

use lib_core::{
    entities::public_ip::{PublicIp, PublicIpError},
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
            .trim()
            .parse()
            .map_err(|_| PublicIpGatewayError::InvalidAnswer(answer.clone()))?;
        PublicIp::try_from(address)
            .map_err(|PublicIpError::NotPublic(address)| PublicIpGatewayError::NotPublic(address))
    }
}

#[cfg(test)]
mod test {
    use std::net::Ipv4Addr;

    use lib_core::use_cases::public_ip_gateway::{PublicIpGateway, PublicIpGatewayError};

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

    #[test]
    fn should_ignore_whitespace_around_the_answer() {
        let public_ip = gateway_answering(" 8.8.8.8\n")
            .current_public_ip()
            .expect("public ip");

        assert_eq!(Ipv4Addr::new(8, 8, 8, 8), public_ip.address());
    }

    #[test]
    fn should_report_an_answer_that_is_not_an_address_as_invalid() {
        let result = gateway_answering("<html>").current_public_ip();

        assert_eq!(
            Err(PublicIpGatewayError::InvalidAnswer("<html>".to_owned())),
            result
        );
    }

    #[test]
    fn should_report_a_private_address_as_not_public() {
        let result = gateway_answering("192.168.1.10").current_public_ip();

        assert_eq!(
            Err(PublicIpGatewayError::NotPublic(Ipv4Addr::new(
                192, 168, 1, 10
            ))),
            result
        );
    }
}

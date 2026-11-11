use std::net::Ipv4Addr;

use lib_core::{
    entities::public_ip::{PublicIp, PublicIpError},
    use_cases::public_ip_gateway::{PublicIpGateway, PublicIpGatewayError},
};

use crate::adapters::ipify_api::{IpifyApi, IpifyApiError};

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
        let answer = self.api.fetch_answer().map_err(unavailable)?;
        let address = parse_address(&answer)?;
        PublicIp::try_from(address).map_err(not_public)
    }
}

fn parse_address(answer: &str) -> Result<Ipv4Addr, PublicIpGatewayError> {
    answer
        .trim()
        .parse()
        .map_err(|_| PublicIpGatewayError::InvalidAnswer(answer.to_owned()))
}

fn unavailable(IpifyApiError::Unreachable(reason): IpifyApiError) -> PublicIpGatewayError {
    PublicIpGatewayError::Unavailable(reason)
}

fn not_public(PublicIpError::NotPublic(address): PublicIpError) -> PublicIpGatewayError {
    PublicIpGatewayError::NotPublic(address)
}

#[cfg(test)]
mod test {
    use std::net::Ipv4Addr;

    use lib_core::use_cases::public_ip_gateway::{PublicIpGateway, PublicIpGatewayError};

    use crate::adapters::{
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

    #[test]
    fn should_report_an_unreachable_ipify_as_unavailable() {
        let unreachable = IpifyApiError::Unreachable("connection refused".to_owned());
        let gateway = IpifyPublicIpGateway::new(FakeIpifyApi(Err(unreachable)));

        assert_eq!(
            Err(PublicIpGatewayError::Unavailable(
                "connection refused".to_owned()
            )),
            gateway.current_public_ip()
        );
    }
}

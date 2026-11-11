use std::{env, process::ExitCode, sync::Arc, time::Duration};

use app_cli::{
    adapters::controllers::run::{Infrastructure, run},
    infrastructure::{
        command_line::Provider,
        token::{HOSTINGER_TOKEN_VARIABLE, hostinger_token},
    },
};
use lib_hostinger::{
    adapters::hostinger_dns_gateway::HostingerDnsGateway,
    infrastructure::ureq_hostinger_api::UreqHostingerApi,
};
use lib_ipify::{
    adapters::ipify_public_ip_gateway::IpifyPublicIpGateway,
    infrastructure::ureq_ipify_api::UreqIpifyApi,
};

const IPIFY_URL: &str = "https://api.ipify.org";
const IPIFY_TIMEOUT: Duration = Duration::from_secs(10);
const HOSTINGER_URL: &str = "https://developers.hostinger.com";
const HOSTINGER_TIMEOUT: Duration = Duration::from_secs(30);

fn main() -> ExitCode {
    let termination = run(env::args_os(), &Internet).unwrap_or_else(|error| error.exit());
    print!("{}", termination.output);
    eprint!("{}", termination.error);
    if termination.success {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

/// The real services farol talks to.
struct Internet;

impl Infrastructure for Internet {
    type PublicIpGateway = IpifyPublicIpGateway<UreqIpifyApi>;
    type DnsGateway = HostingerDnsGateway<UreqHostingerApi>;

    fn public_ip_gateway(&self) -> Arc<Self::PublicIpGateway> {
        let api = UreqIpifyApi::new(IPIFY_URL.to_owned(), IPIFY_TIMEOUT);
        Arc::new(IpifyPublicIpGateway::new(api))
    }

    fn dns_gateway(&self, provider: Provider) -> Result<Arc<Self::DnsGateway>, String> {
        match provider {
            Provider::Hostinger => {
                let token = hostinger_token(environment).map_err(|_| {
                    format!("set {HOSTINGER_TOKEN_VARIABLE} to a Hostinger API token")
                })?;
                let api = UreqHostingerApi::new(HOSTINGER_URL.to_owned(), token, HOSTINGER_TIMEOUT);
                Ok(Arc::new(HostingerDnsGateway::new(api)))
            }
        }
    }
}

fn environment(key: &str) -> Option<String> {
    env::var(key).ok()
}

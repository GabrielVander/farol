use std::time::Duration;

use ureq::Agent;

use crate::infrastructure::adapters::ipify_api::{IpifyApi, IpifyApiError};

/// Asks ipify over HTTP, giving up after `timeout`.
pub struct UreqIpifyApi {
    url: String,
    agent: Agent,
}

impl UreqIpifyApi {
    pub fn new(url: String, timeout: Duration) -> Self {
        let config = Agent::config_builder()
            .timeout_global(Some(timeout))
            .build();
        Self {
            url,
            agent: config.into(),
        }
    }
}

impl IpifyApi for UreqIpifyApi {
    fn fetch_answer(&self) -> Result<String, IpifyApiError> {
        self.agent
            .get(&self.url)
            .call()
            .and_then(|mut response| response.body_mut().read_to_string())
            .map_err(|error| IpifyApiError::Unreachable(error.to_string()))
    }
}

#[cfg(test)]
mod test {
    use std::{
        io::{BufRead, BufReader, Write},
        net::TcpListener,
        thread::{self, JoinHandle},
        time::Duration,
    };

    use crate::infrastructure::{adapters::ipify_api::IpifyApi, ureq_ipify_api::UreqIpifyApi};

    const TIMEOUT: Duration = Duration::from_secs(2);

    fn serve_once(response: &'static str) -> (String, JoinHandle<String>) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let url = format!("http://{}/", listener.local_addr().expect("address"));
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept");
            let mut request_line = String::new();
            BufReader::new(&stream)
                .read_line(&mut request_line)
                .expect("read");
            stream.write_all(response.as_bytes()).expect("write");
            request_line
        });
        (url, server)
    }

    #[test]
    fn should_get_the_answer_from_the_configured_url() {
        let (url, server) =
            serve_once("HTTP/1.1 200 OK\r\nContent-Length: 7\r\nConnection: close\r\n\r\n8.8.8.8");

        let answer = UreqIpifyApi::new(url, TIMEOUT).fetch_answer();

        assert_eq!(Ok("8.8.8.8".to_owned()), answer);
        assert_eq!("GET / HTTP/1.1\r\n", server.join().expect("server"));
    }
}

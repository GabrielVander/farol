use std::time::Duration;

use ureq::Agent;

use crate::infrastructure::{
    adapters::hostinger_api::{HostingerApi, HostingerApiError},
    models::{RecordSet, UpdateZoneRequest},
};

const ZONES_PATH: &str = "/api/dns/v1/zones/";
const TIMEOUT: Duration = Duration::from_secs(30);

/// Talks to Hostinger's DNS API at `base_url`, authenticated by an API token.
pub struct UreqHostingerApi {
    base_url: String,
    token: String,
    agent: Agent,
}

impl UreqHostingerApi {
    pub fn new(base_url: String, token: String) -> Self {
        let config = Agent::config_builder()
            .timeout_global(Some(TIMEOUT))
            .build();
        Self {
            base_url,
            token,
            agent: config.into(),
        }
    }

    fn zone_url(&self, domain: &str) -> String {
        format!("{}{ZONES_PATH}{domain}", self.base_url)
    }

    fn authorization(&self) -> String {
        format!("Bearer {}", self.token)
    }
}

impl HostingerApi for UreqHostingerApi {
    fn zone(&self, domain: &str) -> Result<Vec<RecordSet>, HostingerApiError> {
        let body = self
            .agent
            .get(self.zone_url(domain))
            .header("Authorization", self.authorization())
            .call()
            .and_then(|mut response| response.body_mut().read_to_string())
            .map_err(|error| HostingerApiError::Unreachable(error.to_string()))?;
        serde_json::from_str(&body)
            .map_err(|error| HostingerApiError::Unreachable(error.to_string()))
    }

    fn update_zone(
        &self,
        domain: &str,
        request: &UpdateZoneRequest,
    ) -> Result<(), HostingerApiError> {
        self.agent
            .put(self.zone_url(domain))
            .header("Authorization", self.authorization())
            .send_json(request)
            .map(|_| ())
            .map_err(|error| HostingerApiError::Unreachable(error.to_string()))
    }
}

#[cfg(test)]
mod test {
    use std::{
        io::{BufRead, BufReader, Write},
        net::TcpListener,
        thread::{self, JoinHandle},
    };

    use serde_json::{Value, json};

    use crate::infrastructure::{
        adapters::hostinger_api::HostingerApi,
        models::{RecordSet, RecordValue, UpdateZoneRequest},
        ureq_hostinger_api::UreqHostingerApi,
    };

    const TOKEN: &str = "secret-token";

    fn home_record_set() -> RecordSet {
        RecordSet {
            name: "home".to_owned(),
            kind: "A".to_owned(),
            ttl: 300,
            records: vec![RecordValue {
                content: "8.8.8.8".to_owned(),
                is_disabled: false,
            }],
        }
    }

    fn serve_once(status_line: &'static str, body: &'static str) -> (String, JoinHandle<String>) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let base_url = format!("http://{}", listener.local_addr().expect("address"));
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept");
            let request = read_request(&mut BufReader::new(&stream));
            let response = format!(
                "HTTP/1.1 {status_line}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            stream.write_all(response.as_bytes()).expect("write");
            request
        });
        (base_url, server)
    }

    fn read_request(reader: &mut impl BufRead) -> String {
        let mut head = String::new();
        let mut content_length = 0;
        loop {
            let mut line = String::new();
            reader.read_line(&mut line).expect("read line");
            if let Some(length) = line.to_ascii_lowercase().strip_prefix("content-length:") {
                content_length = length.trim().parse().expect("length");
            }
            head.push_str(&line);
            if line == "\r\n" {
                break;
            }
        }
        let mut body = vec![0; content_length];
        reader.read_exact(&mut body).expect("read body");
        head + &String::from_utf8(body).expect("utf-8")
    }

    #[test]
    fn should_get_the_zone_with_the_bearer_token() {
        let (base_url, server) = serve_once(
            "200 OK",
            r#"[{"name":"home","records":[{"content":"8.8.8.8","is_disabled":false}],"ttl":300,"type":"A"}]"#,
        );

        let zone = UreqHostingerApi::new(base_url, TOKEN.to_owned()).zone("example.com");

        assert_eq!(Ok(vec![home_record_set()]), zone);
        let request = server.join().expect("server").to_ascii_lowercase();
        assert!(request.starts_with("get /api/dns/v1/zones/example.com http/1.1\r\n"));
        assert!(request.contains("\r\nauthorization: bearer secret-token\r\n"));
    }

    #[test]
    fn should_put_the_update_as_json() {
        let (base_url, server) = serve_once("200 OK", r#"{"message":"Request accepted"}"#);
        let request = UpdateZoneRequest {
            overwrite: true,
            zone: vec![home_record_set()],
        };

        let result =
            UreqHostingerApi::new(base_url, TOKEN.to_owned()).update_zone("example.com", &request);

        assert_eq!(Ok(()), result);
        let sent = server.join().expect("server");
        let lowercase = sent.to_ascii_lowercase();
        assert!(lowercase.starts_with("put /api/dns/v1/zones/example.com http/1.1\r\n"));
        assert!(lowercase.contains("\r\nauthorization: bearer secret-token\r\n"));
        assert!(lowercase.contains("\r\ncontent-type: application/json"));
        let (_, body) = sent.split_once("\r\n\r\n").expect("body");
        let expected = json!({
            "overwrite": true,
            "zone": [{"name": "home", "type": "A", "ttl": 300, "records": [{"content": "8.8.8.8"}]}]
        });
        assert_eq!(expected, serde_json::from_str::<Value>(body).expect("json"));
    }
}

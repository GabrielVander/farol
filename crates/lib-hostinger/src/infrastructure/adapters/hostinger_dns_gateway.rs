use std::{
    collections::HashMap,
    net::Ipv4Addr,
    sync::{Mutex, PoisonError},
};

use lib_core::{
    entities::{public_ip::PublicIp, tracked_record::TrackedRecord},
    use_cases::dns_gateway::{DnsGateway, DnsGatewayError},
};

use crate::infrastructure::{
    adapters::hostinger_api::{HostingerApi, HostingerApiError},
    models::{RecordSet, RecordValue, UpdateZoneRequest},
};

pub struct HostingerDnsGateway<A: HostingerApi> {
    api: A,
    zones: Mutex<HashMap<String, Vec<RecordSet>>>,
}

impl<A: HostingerApi> HostingerDnsGateway<A> {
    pub fn new(api: A) -> Self {
        Self {
            api,
            zones: Mutex::default(),
        }
    }

    fn zone(&self, domain: &str) -> Result<Vec<RecordSet>, DnsGatewayError> {
        let mut zones = self.zones.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(record_sets) = zones.get(domain) {
            return Ok(record_sets.clone());
        }
        let record_sets = self.api.zone(domain).map_err(translate_error)?;
        zones.insert(domain.to_owned(), record_sets.clone());
        Ok(record_sets)
    }

    fn forget_zone(&self, domain: &str) {
        let mut zones = self.zones.lock().unwrap_or_else(PoisonError::into_inner);
        zones.remove(domain);
    }
}

impl<A: HostingerApi> DnsGateway for HostingerDnsGateway<A> {
    fn addresses_of(&self, record: &TrackedRecord) -> Result<Vec<Ipv4Addr>, DnsGatewayError> {
        let record_sets = self.zone(record.domain.as_str())?;
        addresses_in(&record_sets, record)
    }

    fn point(&self, record: &TrackedRecord, public_ip: PublicIp) -> Result<(), DnsGatewayError> {
        let request = overwrite_request(record, public_ip);
        self.forget_zone(record.domain.as_str());
        self.api
            .update_zone(record.domain.as_str(), &request)
            .map_err(translate_error)
    }
}

const ADDRESS_RECORD: &str = "A";

fn addresses_in(
    record_sets: &[RecordSet],
    record: &TrackedRecord,
) -> Result<Vec<Ipv4Addr>, DnsGatewayError> {
    record_sets
        .iter()
        .filter(|record_set| holds_addresses_of(record_set, record))
        .flat_map(|record_set| &record_set.records)
        .filter(|value| !value.is_disabled)
        .map(parse_address)
        .collect()
}

fn holds_addresses_of(record_set: &RecordSet, record: &TrackedRecord) -> bool {
    record_set.kind == ADDRESS_RECORD && record_set.name == record.name.as_str()
}

fn parse_address(value: &RecordValue) -> Result<Ipv4Addr, DnsGatewayError> {
    value
        .content
        .parse()
        .map_err(|_| DnsGatewayError::InvalidAnswer(value.content.clone()))
}

fn overwrite_request(record: &TrackedRecord, public_ip: PublicIp) -> UpdateZoneRequest {
    UpdateZoneRequest {
        overwrite: true,
        zone: vec![record_set_pointing(record, public_ip)],
    }
}

fn record_set_pointing(record: &TrackedRecord, public_ip: PublicIp) -> RecordSet {
    RecordSet {
        name: record.name.as_str().to_owned(),
        kind: ADDRESS_RECORD.to_owned(),
        ttl: record.ttl.seconds(),
        records: vec![RecordValue {
            content: public_ip.address().to_string(),
            is_disabled: false,
        }],
    }
}

fn translate_error(error: HostingerApiError) -> DnsGatewayError {
    match error {
        HostingerApiError::Unreachable(reason) => DnsGatewayError::Unavailable(reason),
        HostingerApiError::Status {
            status: 401,
            message,
            ..
        } => DnsGatewayError::Unauthorized(message),
        HostingerApiError::Status {
            status: 404,
            message,
            ..
        } => DnsGatewayError::DomainNotFound(message),
        HostingerApiError::Status { message, .. } => DnsGatewayError::Unavailable(message),
    }
}

#[cfg(test)]
mod test {
    use std::{collections::HashMap, net::Ipv4Addr, sync::Mutex};

    use lib_core::{
        entities::{
            domain_name::DomainName, public_ip::PublicIp, record_name::RecordName,
            tracked_record::TrackedRecord, ttl::Ttl,
        },
        use_cases::dns_gateway::{DnsGateway, DnsGatewayError},
    };

    use crate::infrastructure::{
        adapters::{
            hostinger_api::{HostingerApi, HostingerApiError},
            hostinger_dns_gateway::HostingerDnsGateway,
        },
        models::{RecordSet, RecordValue, UpdateZoneRequest},
    };

    #[derive(Default)]
    struct FakeHostingerApi {
        zones: HashMap<String, Vec<RecordSet>>,
        updates: Mutex<Vec<(String, UpdateZoneRequest)>>,
        zone_reads: Mutex<usize>,
        zone_failure: Option<HostingerApiError>,
    }

    impl FakeHostingerApi {
        fn with_zone(mut self, domain: &str, record_sets: Vec<RecordSet>) -> Self {
            self.zones.insert(domain.to_owned(), record_sets);
            self
        }

        fn failing_with(error: HostingerApiError) -> Self {
            Self {
                zone_failure: Some(error),
                ..Self::default()
            }
        }

        fn zone_reads(&self) -> usize {
            *self.zone_reads.lock().expect("unpoisoned")
        }

        fn updates(&self) -> Vec<(String, UpdateZoneRequest)> {
            self.updates.lock().expect("unpoisoned").clone()
        }
    }

    impl HostingerApi for FakeHostingerApi {
        fn zone(&self, domain: &str) -> Result<Vec<RecordSet>, HostingerApiError> {
            *self.zone_reads.lock().expect("unpoisoned") += 1;
            if let Some(error) = &self.zone_failure {
                return Err(error.clone());
            }
            Ok(self.zones.get(domain).cloned().unwrap_or_default())
        }

        fn update_zone(
            &self,
            domain: &str,
            request: &UpdateZoneRequest,
        ) -> Result<(), HostingerApiError> {
            let mut updates = self.updates.lock().expect("unpoisoned");
            updates.push((domain.to_owned(), request.clone()));
            Ok(())
        }
    }

    fn record_set(name: &str, kind: &str, contents: &[&str]) -> RecordSet {
        RecordSet {
            name: name.to_owned(),
            kind: kind.to_owned(),
            ttl: 300,
            records: contents
                .iter()
                .map(|content| RecordValue {
                    content: (*content).to_owned(),
                    is_disabled: false,
                })
                .collect(),
        }
    }

    fn status(status: u16, message: &str) -> HostingerApiError {
        HostingerApiError::Status {
            status,
            message: message.to_owned(),
            correlation_id: None,
        }
    }

    fn reading_error(error: HostingerApiError) -> DnsGatewayError {
        let gateway = HostingerDnsGateway::new(FakeHostingerApi::failing_with(error));
        gateway
            .addresses_of(&tracked("home"))
            .expect_err("reading should fail")
    }

    fn tracked(name: &str) -> TrackedRecord {
        TrackedRecord {
            domain: DomainName::try_from("example.com").expect("valid domain"),
            name: RecordName::try_from(name).expect("valid record name"),
            ttl: Ttl::try_from(300).expect("valid ttl"),
        }
    }

    #[test]
    fn should_read_the_address_of_a_matching_a_record() {
        let api = FakeHostingerApi::default()
            .with_zone("example.com", vec![record_set("home", "A", &["8.8.8.8"])]);
        let gateway = HostingerDnsGateway::new(api);

        let addresses = gateway.addresses_of(&tracked("home")).expect("addresses");

        assert_eq!(vec![Ipv4Addr::new(8, 8, 8, 8)], addresses);
    }

    #[test]
    fn should_point_a_record_by_overwriting_its_a_records() {
        let gateway = HostingerDnsGateway::new(FakeHostingerApi::default());
        let public_ip = PublicIp::try_from(Ipv4Addr::new(8, 8, 8, 8)).expect("public ip");

        gateway.point(&tracked("home"), public_ip).expect("pointed");

        let expected = UpdateZoneRequest {
            overwrite: true,
            zone: vec![record_set("home", "A", &["8.8.8.8"])],
        };
        assert_eq!(
            vec![("example.com".to_owned(), expected)],
            gateway.api.updates()
        );
    }

    #[test]
    fn should_ignore_other_types_and_names() {
        let api = FakeHostingerApi::default().with_zone(
            "example.com",
            vec![
                record_set("home", "AAAA", &["2001:db8::1"]),
                record_set("lab", "A", &["1.1.1.1"]),
                record_set("home", "TXT", &["9.9.9.9"]),
                record_set("home", "A", &["8.8.8.8"]),
            ],
        );
        let gateway = HostingerDnsGateway::new(api);

        let addresses = gateway.addresses_of(&tracked("home")).expect("addresses");

        assert_eq!(vec![Ipv4Addr::new(8, 8, 8, 8)], addresses);
    }

    #[test]
    fn should_ignore_disabled_records() {
        let mut home = record_set("home", "A", &["1.1.1.1", "8.8.8.8"]);
        home.records[0].is_disabled = true;
        let api = FakeHostingerApi::default().with_zone("example.com", vec![home]);
        let gateway = HostingerDnsGateway::new(api);

        let addresses = gateway.addresses_of(&tracked("home")).expect("addresses");

        assert_eq!(vec![Ipv4Addr::new(8, 8, 8, 8)], addresses);
    }

    #[test]
    fn should_find_no_addresses_for_a_record_missing_from_the_zone() {
        let api = FakeHostingerApi::default()
            .with_zone("example.com", vec![record_set("lab", "A", &["8.8.8.8"])]);
        let gateway = HostingerDnsGateway::new(api);

        let addresses = gateway.addresses_of(&tracked("home")).expect("addresses");

        assert!(addresses.is_empty());
    }

    #[test]
    fn should_report_content_that_is_not_an_address_as_invalid() {
        let api = FakeHostingerApi::default()
            .with_zone("example.com", vec![record_set("home", "A", &["not-an-ip"])]);
        let gateway = HostingerDnsGateway::new(api);

        assert_eq!(
            Err(DnsGatewayError::InvalidAnswer("not-an-ip".to_owned())),
            gateway.addresses_of(&tracked("home"))
        );
    }

    #[test]
    fn should_read_each_zone_once() {
        let api = FakeHostingerApi::default().with_zone(
            "example.com",
            vec![
                record_set("@", "A", &["8.8.8.8"]),
                record_set("home", "A", &["8.8.8.8"]),
            ],
        );
        let gateway = HostingerDnsGateway::new(api);

        gateway.addresses_of(&tracked("@")).expect("addresses");
        gateway.addresses_of(&tracked("home")).expect("addresses");

        assert_eq!(1, gateway.api.zone_reads());
    }

    #[test]
    fn should_read_the_zone_again_after_pointing_a_record() {
        let api = FakeHostingerApi::default()
            .with_zone("example.com", vec![record_set("home", "A", &["1.1.1.1"])]);
        let gateway = HostingerDnsGateway::new(api);
        let public_ip = PublicIp::try_from(Ipv4Addr::new(8, 8, 8, 8)).expect("public ip");

        gateway.addresses_of(&tracked("home")).expect("addresses");
        gateway.point(&tracked("home"), public_ip).expect("pointed");
        gateway.addresses_of(&tracked("home")).expect("addresses");

        assert_eq!(2, gateway.api.zone_reads());
    }

    #[test]
    fn should_report_401_as_unauthorized() {
        assert_eq!(
            DnsGatewayError::Unauthorized("Unauthenticated.".to_owned()),
            reading_error(status(401, "Unauthenticated."))
        );
    }

    #[test]
    fn should_report_404_as_domain_not_found() {
        assert_eq!(
            DnsGatewayError::DomainNotFound("Not found.".to_owned()),
            reading_error(status(404, "Not found."))
        );
    }
}

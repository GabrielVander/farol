use std::net::Ipv4Addr;

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
}

impl<A: HostingerApi> HostingerDnsGateway<A> {
    pub fn new(api: A) -> Self {
        Self { api }
    }
}

impl<A: HostingerApi> DnsGateway for HostingerDnsGateway<A> {
    fn addresses_of(&self, record: &TrackedRecord) -> Result<Vec<Ipv4Addr>, DnsGatewayError> {
        let record_sets = self.api.zone(record.domain.as_str()).map_err(
            |HostingerApiError::Unreachable(reason)| DnsGatewayError::Unavailable(reason),
        )?;
        record_sets
            .iter()
            .filter(|record_set| record_set.kind == ADDRESS_RECORD)
            .filter(|record_set| record_set.name == record.name.as_str())
            .flat_map(|record_set| &record_set.records)
            .filter(|value| !value.is_disabled)
            .map(|value| {
                value
                    .content
                    .parse()
                    .map_err(|_| DnsGatewayError::InvalidAnswer(value.content.clone()))
            })
            .collect()
    }

    fn point(&self, record: &TrackedRecord, public_ip: PublicIp) -> Result<(), DnsGatewayError> {
        let request = overwrite_request(record, public_ip);
        self.api
            .update_zone(record.domain.as_str(), &request)
            .map_err(translate_error)
    }
}

const ADDRESS_RECORD: &str = "A";

fn overwrite_request(record: &TrackedRecord, public_ip: PublicIp) -> UpdateZoneRequest {
    let record_set = RecordSet {
        name: record.name.as_str().to_owned(),
        kind: ADDRESS_RECORD.to_owned(),
        ttl: record.ttl.seconds(),
        records: vec![RecordValue {
            content: public_ip.address().to_string(),
            is_disabled: false,
        }],
    };
    UpdateZoneRequest {
        overwrite: true,
        zone: vec![record_set],
    }
}

fn translate_error(HostingerApiError::Unreachable(reason): HostingerApiError) -> DnsGatewayError {
    DnsGatewayError::Unavailable(reason)
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
    }

    impl FakeHostingerApi {
        fn with_zone(mut self, domain: &str, record_sets: Vec<RecordSet>) -> Self {
            self.zones.insert(domain.to_owned(), record_sets);
            self
        }

        fn updates(&self) -> Vec<(String, UpdateZoneRequest)> {
            self.updates.lock().expect("unpoisoned").clone()
        }
    }

    impl HostingerApi for FakeHostingerApi {
        fn zone(&self, domain: &str) -> Result<Vec<RecordSet>, HostingerApiError> {
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
}

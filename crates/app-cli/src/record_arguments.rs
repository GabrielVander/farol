use lib_core::entities::{
    domain_name::DomainName, record_name::RecordName, tracked_record::TrackedRecord, ttl::Ttl,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordArgumentError {}

/// Turns `DOMAIN:NAME` arguments into tracked records that share one TTL.
pub fn tracked_records(
    arguments: &[String],
    ttl: u32,
) -> Result<Vec<TrackedRecord>, RecordArgumentError> {
    Ok(arguments
        .iter()
        .filter_map(|argument| tracked_record(argument, ttl))
        .collect())
}

fn tracked_record(argument: &str, ttl: u32) -> Option<TrackedRecord> {
    let (domain, name) = argument.split_once(':')?;
    Some(TrackedRecord {
        domain: DomainName::try_from(domain).ok()?,
        name: RecordName::try_from(name).ok()?,
        ttl: Ttl::try_from(ttl).ok()?,
    })
}

#[cfg(test)]
mod test {
    use lib_core::entities::{
        domain_name::DomainName, record_name::RecordName, tracked_record::TrackedRecord, ttl::Ttl,
    };

    use crate::record_arguments::tracked_records;

    fn arguments(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_owned()).collect()
    }

    fn tracked(domain: &str, name: &str, ttl: u32) -> TrackedRecord {
        TrackedRecord {
            domain: DomainName::try_from(domain).expect("valid domain"),
            name: RecordName::try_from(name).expect("valid record name"),
            ttl: Ttl::try_from(ttl).expect("valid ttl"),
        }
    }

    #[test]
    fn should_track_a_record_given_as_domain_and_name() {
        let records = tracked_records(&arguments(&["example.com:home"]), 600);

        assert_eq!(Ok(vec![tracked("example.com", "home", 600)]), records);
    }

    #[test]
    fn should_keep_several_records_in_order_across_domains() {
        let records = tracked_records(&arguments(&["example.org:lab", "example.com:home"]), 300);

        let expected = vec![
            tracked("example.org", "lab", 300),
            tracked("example.com", "home", 300),
        ];
        assert_eq!(Ok(expected), records);
    }
}

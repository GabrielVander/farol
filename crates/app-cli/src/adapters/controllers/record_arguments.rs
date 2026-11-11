use lib_core::entities::{
    domain_name::{DomainName, DomainNameError},
    label::LabelError,
    record_name::RecordName,
    tracked_record::TrackedRecord,
    ttl::{Ttl, TtlError},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordArgumentError {
    MissingSeparator(String),
    InvalidDomain {
        domain: String,
        reason: DomainNameError,
    },
    InvalidRecordName {
        name: String,
        reason: LabelError,
    },
    InvalidTtl(TtlError),
}

/// Turns `DOMAIN:NAME` arguments into tracked records that share one TTL.
pub fn tracked_records(
    arguments: &[String],
    ttl: u32,
) -> Result<Vec<TrackedRecord>, RecordArgumentError> {
    let ttl = Ttl::try_from(ttl).map_err(RecordArgumentError::InvalidTtl)?;

    arguments
        .iter()
        .map(|argument| tracked_record(argument, ttl))
        .collect()
}

fn tracked_record(argument: &str, ttl: Ttl) -> Result<TrackedRecord, RecordArgumentError> {
    let (domain, name) = argument
        .split_once(':')
        .ok_or_else(|| RecordArgumentError::MissingSeparator(argument.to_owned()))?;

    Ok(TrackedRecord {
        domain: parse_domain(domain)?,
        name: parse_record_name(name)?,
        ttl,
    })
}

fn parse_domain(domain: &str) -> Result<DomainName, RecordArgumentError> {
    DomainName::try_from(domain).map_err(|reason| RecordArgumentError::InvalidDomain {
        domain: domain.to_owned(),
        reason,
    })
}

fn parse_record_name(name: &str) -> Result<RecordName, RecordArgumentError> {
    RecordName::try_from(name).map_err(|reason| RecordArgumentError::InvalidRecordName {
        name: name.to_owned(),
        reason,
    })
}

#[cfg(test)]
mod test {
    use lib_core::entities::{
        domain_name::{DomainName, DomainNameError},
        label::LabelError,
        record_name::RecordName,
        tracked_record::TrackedRecord,
        ttl::{Ttl, TtlError},
    };

    use crate::adapters::controllers::record_arguments::{RecordArgumentError, tracked_records};

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
    fn should_reject_an_argument_without_a_separator_and_quote_it() {
        let records = tracked_records(&arguments(&["example.com"]), 300);

        let expected = RecordArgumentError::MissingSeparator("example.com".to_owned());
        assert_eq!(Err(expected), records);
    }

    #[test]
    fn should_reject_an_invalid_domain_and_name_it() {
        let records = tracked_records(&arguments(&["exa_mple.com:home"]), 300);

        let expected = RecordArgumentError::InvalidDomain {
            domain: "exa_mple.com".to_owned(),
            reason: DomainNameError::Label(LabelError::InvalidCharacter('_')),
        };
        assert_eq!(Err(expected), records);
    }

    #[test]
    fn should_reject_an_invalid_record_name_and_name_it() {
        let records = tracked_records(&arguments(&["example.com:-home"]), 300);

        let expected = RecordArgumentError::InvalidRecordName {
            name: "-home".to_owned(),
            reason: LabelError::HyphenAtEdge,
        };
        assert_eq!(Err(expected), records);
    }

    #[test]
    fn should_reject_a_second_separator_as_part_of_the_record_name() {
        let records = tracked_records(&arguments(&["example.com:home:600"]), 300);

        let expected = RecordArgumentError::InvalidRecordName {
            name: "home:600".to_owned(),
            reason: LabelError::InvalidCharacter(':'),
        };
        assert_eq!(Err(expected), records);
    }

    #[test]
    fn should_reject_a_zero_ttl() {
        let records = tracked_records(&arguments(&["example.com:home"]), 0);

        assert_eq!(
            Err(RecordArgumentError::InvalidTtl(TtlError::Zero)),
            records
        );
    }

    #[test]
    fn should_track_a_wildcard_of_a_domain() {
        let records = tracked_records(&arguments(&["example.com:*"]), 300);

        assert_eq!(Ok(vec![tracked("example.com", "*", 300)]), records);
    }

    #[test]
    fn should_track_the_apex_of_a_domain() {
        let records = tracked_records(&arguments(&["example.com:@"]), 300);

        assert_eq!(Ok(vec![tracked("example.com", "@", 300)]), records);
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

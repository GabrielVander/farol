use std::fmt::Debug;

use lib_core::entities::{
    domain_name::DomainName, record_name::RecordName, tracked_record::TrackedRecord, ttl::Ttl,
};
use serde::Deserialize;

/// What farol keeps pointed at the public IP, and through which DNS provider.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub provider: Provider,
    pub records: Vec<TrackedRecord>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provider {
    Hostinger,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigError {
    InvalidToml(String),
    Invalid(String),
}

impl Config {
    pub fn parse(text: &str) -> Result<Self, ConfigError> {
        let raw: RawConfig =
            toml::from_str(text).map_err(|error| ConfigError::InvalidToml(error.to_string()))?;
        let records = raw
            .records
            .iter()
            .map(RawRecord::to_tracked)
            .collect::<Result<_, _>>()?;
        Ok(Self {
            provider: Provider::Hostinger,
            records,
        })
    }
}

#[derive(Deserialize)]
struct RawConfig {
    records: Vec<RawRecord>,
}

#[derive(Deserialize)]
struct RawRecord {
    domain: String,
    name: String,
    ttl: u32,
}

impl RawRecord {
    fn to_tracked(&self) -> Result<TrackedRecord, ConfigError> {
        Ok(TrackedRecord {
            domain: DomainName::try_from(self.domain.as_str()).map_err(invalid)?,
            name: RecordName::try_from(self.name.as_str()).map_err(invalid)?,
            ttl: Ttl::try_from(self.ttl).map_err(invalid)?,
        })
    }
}

fn invalid(error: impl Debug) -> ConfigError {
    ConfigError::Invalid(format!("{error:?}"))
}

#[cfg(test)]
mod test {
    use lib_core::entities::{
        domain_name::DomainName, record_name::RecordName, tracked_record::TrackedRecord, ttl::Ttl,
    };

    use crate::config::{Config, ConfigError, Provider};

    fn tracked(domain: &str, name: &str, ttl: u32) -> TrackedRecord {
        TrackedRecord {
            domain: DomainName::try_from(domain).expect("valid domain"),
            name: RecordName::try_from(name).expect("valid record name"),
            ttl: Ttl::try_from(ttl).expect("valid ttl"),
        }
    }

    #[test]
    fn should_parse_a_hostinger_config_with_several_records() {
        let text = r#"
            provider = "hostinger"

            [[records]]
            domain = "example.com"
            name = "@"
            ttl = 300

            [[records]]
            domain = "example.org"
            name = "home"
            ttl = 600
        "#;

        let expected = Config {
            provider: Provider::Hostinger,
            records: vec![
                tracked("example.com", "@", 300),
                tracked("example.org", "home", 600),
            ],
        };
        assert_eq!(Ok(expected), Config::parse(text));
    }

    #[test]
    fn should_report_toml_that_does_not_parse() {
        assert!(matches!(
            Config::parse("provider = "),
            Err(ConfigError::InvalidToml(_))
        ));
    }
}

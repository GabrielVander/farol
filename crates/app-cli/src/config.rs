use std::fmt::Debug;

use lib_core::entities::{
    domain_name::{DomainName, DomainNameError},
    label::LabelError,
    record_name::RecordName,
    tracked_record::TrackedRecord,
    ttl::{Ttl, TtlError},
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

impl Provider {
    fn named(name: &str) -> Result<Self, ConfigError> {
        match name {
            "hostinger" => Ok(Self::Hostinger),
            unknown => Err(ConfigError::UnknownProvider(unknown.to_owned())),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigError {
    InvalidToml(String),
    UnknownProvider(String),
    InvalidDomain {
        domain: String,
        reason: DomainNameError,
    },
    InvalidRecordName {
        name: String,
        reason: LabelError,
    },
    InvalidTtl {
        domain: String,
        name: String,
        reason: TtlError,
    },
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
            provider: Provider::named(&raw.provider)?,
            records,
        })
    }
}

#[derive(Deserialize)]
struct RawConfig {
    provider: String,
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
            domain: DomainName::try_from(self.domain.as_str()).map_err(|reason| {
                ConfigError::InvalidDomain {
                    domain: self.domain.clone(),
                    reason,
                }
            })?,
            name: RecordName::try_from(self.name.as_str()).map_err(|reason| {
                ConfigError::InvalidRecordName {
                    name: self.name.clone(),
                    reason,
                }
            })?,
            ttl: Ttl::try_from(self.ttl).map_err(|reason| ConfigError::InvalidTtl {
                domain: self.domain.clone(),
                name: self.name.clone(),
                reason,
            })?,
        })
    }
}

fn invalid(error: impl Debug) -> ConfigError {
    ConfigError::Invalid(format!("{error:?}"))
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

    use crate::config::{Config, ConfigError, Provider};

    fn single_record(domain: &str, name: &str, ttl: u32) -> String {
        format!(
            "provider = \"hostinger\"\n[[records]]\ndomain = \"{domain}\"\nname = \"{name}\"\nttl = {ttl}\n"
        )
    }

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

    #[test]
    fn should_reject_an_unknown_provider() {
        let text = r#"
            provider = "cloudflare"

            [[records]]
            domain = "example.com"
            name = "@"
            ttl = 300
        "#;

        assert_eq!(
            Err(ConfigError::UnknownProvider("cloudflare".to_owned())),
            Config::parse(text)
        );
    }

    #[test]
    fn should_reject_an_invalid_domain_and_name_it() {
        let text = single_record("exa_mple.com", "@", 300);

        let expected = ConfigError::InvalidDomain {
            domain: "exa_mple.com".to_owned(),
            reason: DomainNameError::Label(LabelError::InvalidCharacter('_')),
        };
        assert_eq!(Err(expected), Config::parse(&text));
    }

    #[test]
    fn should_reject_an_invalid_record_name_and_name_it() {
        let text = single_record("example.com", "-home", 300);

        let expected = ConfigError::InvalidRecordName {
            name: "-home".to_owned(),
            reason: LabelError::HyphenAtEdge,
        };
        assert_eq!(Err(expected), Config::parse(&text));
    }

    #[test]
    fn should_reject_a_zero_ttl() {
        let text = single_record("example.com", "home", 0);

        let expected = ConfigError::InvalidTtl {
            domain: "example.com".to_owned(),
            name: "home".to_owned(),
            reason: TtlError::Zero,
        };
        assert_eq!(Err(expected), Config::parse(&text));
    }
}

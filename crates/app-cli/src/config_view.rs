use lib_core::entities::{domain_name::DomainNameError, label::LabelError, ttl::TtlError};

use crate::config::ConfigError;

/// What is wrong with the config, in words that point at the line to fix.
pub fn explain_config_error(error: &ConfigError) -> String {
    match error {
        ConfigError::InvalidToml(detail) => format!("the config is not valid TOML: {detail}"),
        ConfigError::NoRecords => "the config has no [[records]]".to_owned(),
        ConfigError::UnknownProvider(provider) => {
            format!("unknown provider \"{provider}\" (supported: hostinger)")
        }
        ConfigError::InvalidDomain { domain, reason } => {
            format!("invalid domain \"{domain}\": {}", domain_problem(reason))
        }
        ConfigError::InvalidRecordName { name, reason } => {
            format!("invalid record name \"{name}\": {}", label_problem(reason))
        }
        ConfigError::InvalidTtl {
            domain,
            name,
            reason,
        } => format!(
            "invalid ttl for \"{name}\" in \"{domain}\": {}",
            ttl_problem(reason)
        ),
    }
}

fn domain_problem(reason: &DomainNameError) -> String {
    match reason {
        DomainNameError::Empty => "it is empty".to_owned(),
        DomainNameError::TooLong => "it is longer than 253 characters".to_owned(),
        DomainNameError::Label(label) => label_problem(label),
    }
}

fn label_problem(reason: &LabelError) -> String {
    match reason {
        LabelError::Empty => "it has an empty label".to_owned(),
        LabelError::TooLong => "a label is longer than 63 characters".to_owned(),
        LabelError::InvalidCharacter(character) => format!("it contains '{character}'"),
        LabelError::HyphenAtEdge => "a label starts or ends with a hyphen".to_owned(),
    }
}

fn ttl_problem(reason: &TtlError) -> &'static str {
    match reason {
        TtlError::Zero => "it must be at least 1 second",
    }
}

#[cfg(test)]
mod test {
    use lib_core::entities::{domain_name::DomainNameError, label::LabelError, ttl::TtlError};

    use crate::{config::ConfigError, config_view::explain_config_error};

    #[test]
    fn should_explain_each_config_error() {
        let explanations = [
            (
                ConfigError::InvalidToml("expected a value".to_owned()),
                "the config is not valid TOML: expected a value",
            ),
            (ConfigError::NoRecords, "the config has no [[records]]"),
            (
                ConfigError::UnknownProvider("cloudflare".to_owned()),
                "unknown provider \"cloudflare\" (supported: hostinger)",
            ),
            (
                ConfigError::InvalidDomain {
                    domain: "".to_owned(),
                    reason: DomainNameError::Empty,
                },
                "invalid domain \"\": it is empty",
            ),
            (
                ConfigError::InvalidRecordName {
                    name: "-home".to_owned(),
                    reason: LabelError::HyphenAtEdge,
                },
                "invalid record name \"-home\": a label starts or ends with a hyphen",
            ),
            (
                ConfigError::InvalidTtl {
                    domain: "example.com".to_owned(),
                    name: "home".to_owned(),
                    reason: TtlError::Zero,
                },
                "invalid ttl for \"home\" in \"example.com\": it must be at least 1 second",
            ),
        ];

        for (error, explanation) in explanations {
            assert_eq!(explanation, explain_config_error(&error));
        }
    }

    #[test]
    fn should_explain_each_domain_problem() {
        let explanations = [
            (DomainNameError::TooLong, "it is longer than 253 characters"),
            (
                DomainNameError::Label(LabelError::Empty),
                "it has an empty label",
            ),
            (
                DomainNameError::Label(LabelError::TooLong),
                "a label is longer than 63 characters",
            ),
            (
                DomainNameError::Label(LabelError::InvalidCharacter('_')),
                "it contains '_'",
            ),
        ];

        for (reason, problem) in explanations {
            let error = ConfigError::InvalidDomain {
                domain: "x".to_owned(),
                reason,
            };
            assert_eq!(
                format!("invalid domain \"x\": {problem}"),
                explain_config_error(&error)
            );
        }
    }
}

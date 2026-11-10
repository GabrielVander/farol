use lib_core::entities::{domain_name::DomainNameError, label::LabelError, ttl::TtlError};

use crate::record_arguments::RecordArgumentError;

/// What is wrong with a `--record` or `--ttl` argument.
pub fn explain_record_argument_error(error: &RecordArgumentError) -> String {
    match error {
        RecordArgumentError::MissingSeparator(argument) => format!(
            "invalid record \"{argument}\": expected DOMAIN:NAME, for example example.com:home"
        ),
        RecordArgumentError::InvalidDomain { domain, reason } => {
            format!("invalid domain \"{domain}\": {}", domain_problem(reason))
        }
        RecordArgumentError::InvalidRecordName { name, reason } => {
            format!("invalid record name \"{name}\": {}", label_problem(reason))
        }
        RecordArgumentError::InvalidTtl(reason) => format!("invalid ttl: {}", ttl_problem(reason)),
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

    use crate::{
        config_view::explain_record_argument_error, record_arguments::RecordArgumentError,
    };

    #[test]
    fn should_explain_each_record_argument_error() {
        let explanations = [
            (
                RecordArgumentError::MissingSeparator("example.com".to_owned()),
                "invalid record \"example.com\": expected DOMAIN:NAME, for example example.com:home",
            ),
            (
                RecordArgumentError::InvalidDomain {
                    domain: "".to_owned(),
                    reason: DomainNameError::Empty,
                },
                "invalid domain \"\": it is empty",
            ),
            (
                RecordArgumentError::InvalidRecordName {
                    name: "-home".to_owned(),
                    reason: LabelError::HyphenAtEdge,
                },
                "invalid record name \"-home\": a label starts or ends with a hyphen",
            ),
            (
                RecordArgumentError::InvalidTtl(TtlError::Zero),
                "invalid ttl: it must be at least 1 second",
            ),
        ];

        for (error, explanation) in explanations {
            assert_eq!(explanation, explain_record_argument_error(&error));
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
            let error = RecordArgumentError::InvalidDomain {
                domain: "x".to_owned(),
                reason,
            };
            assert_eq!(
                format!("invalid domain \"x\": {problem}"),
                explain_record_argument_error(&error)
            );
        }
    }
}

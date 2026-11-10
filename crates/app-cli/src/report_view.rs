use std::net::Ipv4Addr;

use lib_core::{
    entities::tracked_record::TrackedRecord,
    use_cases::{
        dns_gateway::DnsGatewayError,
        public_ip_gateway::PublicIpGatewayError,
        report::{Outcome, RecordOutcome, Report},
    },
};

const APEX: &str = "@";

/// The report as lines of text: the public IP, then one line per tracked record.
pub fn render(report: &Report) -> String {
    let header = format!("public IP {}\n", report.public_ip.address());
    report
        .records
        .iter()
        .map(record_line)
        .fold(header, |text, line| text + &line)
}

/// Whether any tracked record failed, which makes the whole run fail.
pub fn has_failures(report: &Report) -> bool {
    report
        .records
        .iter()
        .any(|record_outcome| matches!(record_outcome.outcome, Outcome::Failed(_)))
}

/// Why no record could be checked: the public IP is unknown.
pub fn explain_public_ip_error(error: &PublicIpGatewayError) -> String {
    let reason = match error {
        PublicIpGatewayError::Unavailable(detail) => {
            format!("the public IP service is unavailable ({detail})")
        }
        PublicIpGatewayError::InvalidAnswer(detail) => {
            format!("the public IP service gave an unexpected answer ({detail})")
        }
        PublicIpGatewayError::NotPublic(address) => {
            format!("the public IP service answered {address}, which is not a public address")
        }
    };
    format!("could not find the public IP: {reason}")
}

fn record_line(record_outcome: &RecordOutcome) -> String {
    let name = full_name(&record_outcome.record);
    format!("{name}: {}\n", describe(&record_outcome.outcome))
}

fn describe(outcome: &Outcome) -> String {
    match outcome {
        Outcome::Unchanged => "unchanged".to_owned(),
        Outcome::Drifted { current } => format!("drifted (now {})", list(&current.addresses)),
        Outcome::Pointed { previous } => format!("pointed (was {})", list(&previous.addresses)),
        Outcome::Failed(error) => format!("failed, {}", explain(error)),
    }
}

fn explain(error: &DnsGatewayError) -> String {
    format!("{} ({})", explanation(error), detail(error))
}

fn explanation(error: &DnsGatewayError) -> &'static str {
    match error {
        DnsGatewayError::Unavailable(_) => "the DNS provider is unavailable",
        DnsGatewayError::InvalidAnswer(_) => "the DNS provider gave an unexpected answer",
        DnsGatewayError::Unauthorized(_) => "the DNS provider refused the API token",
        DnsGatewayError::DomainNotFound(_) => "the domain is not in this DNS provider account",
        DnsGatewayError::Rejected(_) => "the DNS provider rejected the change",
        DnsGatewayError::RateLimited(_) => "too many requests to the DNS provider, try again later",
    }
}

fn detail(error: &DnsGatewayError) -> &str {
    match error {
        DnsGatewayError::Unavailable(detail)
        | DnsGatewayError::InvalidAnswer(detail)
        | DnsGatewayError::Unauthorized(detail)
        | DnsGatewayError::DomainNotFound(detail)
        | DnsGatewayError::Rejected(detail)
        | DnsGatewayError::RateLimited(detail) => detail,
    }
}

fn list(addresses: &[Ipv4Addr]) -> String {
    if addresses.is_empty() {
        return "no A record".to_owned();
    }
    let addresses: Vec<String> = addresses.iter().map(Ipv4Addr::to_string).collect();
    addresses.join(", ")
}

fn full_name(record: &TrackedRecord) -> String {
    match record.name.as_str() {
        APEX => record.domain.as_str().to_owned(),
        name => format!("{name}.{}", record.domain.as_str()),
    }
}

#[cfg(test)]
mod test {
    use std::net::Ipv4Addr;

    use lib_core::{
        entities::{
            domain_name::DomainName, public_ip::PublicIp, record_name::RecordName,
            tracked_record::TrackedRecord, ttl::Ttl,
        },
        use_cases::{
            dns_gateway::{DnsGatewayError, PublishedRecord},
            public_ip_gateway::PublicIpGatewayError,
            report::{Outcome, RecordOutcome, Report},
        },
    };

    use crate::report_view::{explain_public_ip_error, has_failures, render};

    fn public_ip() -> PublicIp {
        PublicIp::try_from(Ipv4Addr::new(8, 8, 8, 8)).expect("public ip")
    }

    fn published(addresses: Vec<Ipv4Addr>) -> PublishedRecord {
        PublishedRecord {
            addresses,
            ttl: None,
        }
    }

    fn report_with(name: &str, outcome: Outcome) -> Report {
        let record = TrackedRecord {
            domain: DomainName::try_from("example.com").expect("valid domain"),
            name: RecordName::try_from(name).expect("valid record name"),
            ttl: Ttl::try_from(300).expect("valid ttl"),
        };
        Report {
            public_ip: public_ip(),
            records: vec![RecordOutcome { record, outcome }],
        }
    }

    fn record_line(report: &Report) -> String {
        render(report).lines().nth(1).unwrap_or_default().to_owned()
    }

    #[test]
    fn should_start_with_the_public_ip() {
        let report = Report {
            public_ip: public_ip(),
            records: vec![],
        };

        assert_eq!("public IP 8.8.8.8\n", render(&report));
    }

    #[test]
    fn should_render_an_unchanged_record_by_its_full_name() {
        assert_eq!(
            "home.example.com: unchanged",
            record_line(&report_with("home", Outcome::Unchanged))
        );
        assert_eq!(
            "example.com: unchanged",
            record_line(&report_with("@", Outcome::Unchanged))
        );
    }

    #[test]
    fn should_render_a_wildcard_by_its_full_name() {
        assert_eq!(
            "*.example.com: unchanged",
            record_line(&report_with("*", Outcome::Unchanged))
        );
    }

    #[test]
    fn should_render_a_drifted_record_with_its_current_addresses() {
        let addresses = vec![Ipv4Addr::new(1, 1, 1, 1), Ipv4Addr::new(2, 2, 2, 2)];

        assert_eq!(
            "home.example.com: drifted (now 1.1.1.1, 2.2.2.2)",
            record_line(&report_with(
                "home",
                Outcome::Drifted {
                    current: published(addresses)
                }
            ))
        );
        assert_eq!(
            "home.example.com: drifted (now no A record)",
            record_line(&report_with(
                "home",
                Outcome::Drifted {
                    current: published(vec![])
                }
            ))
        );
    }

    #[test]
    fn should_render_a_pointed_record_with_its_previous_addresses() {
        let previous = vec![Ipv4Addr::new(1, 1, 1, 1)];

        assert_eq!(
            "home.example.com: pointed (was 1.1.1.1)",
            record_line(&report_with(
                "home",
                Outcome::Pointed {
                    previous: published(previous)
                }
            ))
        );
    }

    #[test]
    fn should_explain_why_a_record_failed() {
        let detail = || "detail".to_owned();
        let explanations = [
            (
                DnsGatewayError::Unavailable(detail()),
                "the DNS provider is unavailable",
            ),
            (
                DnsGatewayError::InvalidAnswer(detail()),
                "the DNS provider gave an unexpected answer",
            ),
            (
                DnsGatewayError::Unauthorized(detail()),
                "the DNS provider refused the API token",
            ),
            (
                DnsGatewayError::DomainNotFound(detail()),
                "the domain is not in this DNS provider account",
            ),
            (
                DnsGatewayError::Rejected(detail()),
                "the DNS provider rejected the change",
            ),
            (
                DnsGatewayError::RateLimited(detail()),
                "too many requests to the DNS provider, try again later",
            ),
        ];

        for (error, explanation) in explanations {
            assert_eq!(
                format!("home.example.com: failed, {explanation} (detail)"),
                record_line(&report_with("home", Outcome::Failed(error)))
            );
        }
    }

    #[test]
    fn should_have_no_failures_when_every_record_succeeded() {
        let report = report_with(
            "home",
            Outcome::Pointed {
                previous: published(vec![]),
            },
        );

        assert!(!has_failures(&report));
    }

    #[test]
    fn should_have_failures_when_any_record_failed() {
        let mut report = report_with("home", Outcome::Unchanged);
        let failed = report_with(
            "lab",
            Outcome::Failed(DnsGatewayError::Unavailable("timeout".to_owned())),
        );
        report.records.extend(failed.records);

        assert!(has_failures(&report));
    }

    #[test]
    fn should_explain_why_the_public_ip_is_unknown() {
        let explanations = [
            (
                PublicIpGatewayError::Unavailable("timeout".to_owned()),
                "could not find the public IP: the public IP service is unavailable (timeout)",
            ),
            (
                PublicIpGatewayError::InvalidAnswer("<html>".to_owned()),
                "could not find the public IP: the public IP service gave an unexpected answer (<html>)",
            ),
            (
                PublicIpGatewayError::NotPublic(Ipv4Addr::new(10, 0, 0, 1)),
                "could not find the public IP: the public IP service answered 10.0.0.1, which is not a public address",
            ),
        ];

        for (error, explanation) in explanations {
            assert_eq!(explanation, explain_public_ip_error(&error));
        }
    }
}

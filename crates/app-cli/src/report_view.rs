use std::net::Ipv4Addr;

use lib_core::{
    entities::tracked_record::TrackedRecord,
    use_cases::report::{Outcome, RecordOutcome, Report},
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

fn record_line(record_outcome: &RecordOutcome) -> String {
    let name = full_name(&record_outcome.record);
    format!("{name}: {}\n", describe(&record_outcome.outcome))
}

fn describe(outcome: &Outcome) -> String {
    match outcome {
        Outcome::Unchanged => "unchanged".to_owned(),
        Outcome::Drifted { addresses } => format!("drifted (now {})", list(addresses)),
        Outcome::Pointed { previous } => format!("pointed (was {})", list(previous)),
        _ => String::new(),
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
        use_cases::report::{Outcome, RecordOutcome, Report},
    };

    use crate::report_view::render;

    fn public_ip() -> PublicIp {
        PublicIp::try_from(Ipv4Addr::new(8, 8, 8, 8)).expect("public ip")
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
    fn should_render_a_drifted_record_with_its_current_addresses() {
        let addresses = vec![Ipv4Addr::new(1, 1, 1, 1), Ipv4Addr::new(2, 2, 2, 2)];

        assert_eq!(
            "home.example.com: drifted (now 1.1.1.1, 2.2.2.2)",
            record_line(&report_with("home", Outcome::Drifted { addresses }))
        );
        assert_eq!(
            "home.example.com: drifted (now no A record)",
            record_line(&report_with("home", Outcome::Drifted { addresses: vec![] }))
        );
    }

    #[test]
    fn should_render_a_pointed_record_with_its_previous_addresses() {
        let previous = vec![Ipv4Addr::new(1, 1, 1, 1)];

        assert_eq!(
            "home.example.com: pointed (was 1.1.1.1)",
            record_line(&report_with("home", Outcome::Pointed { previous }))
        );
    }
}

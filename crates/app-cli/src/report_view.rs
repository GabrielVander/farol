use lib_core::{
    entities::tracked_record::TrackedRecord,
    use_cases::report::{RecordOutcome, Report},
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
    format!("{}: unchanged\n", full_name(&record_outcome.record))
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
}

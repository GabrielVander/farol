use lib_core::use_cases::report::Report;

/// The report as lines of text: the public IP, then one line per tracked record.
pub fn render(report: &Report) -> String {
    format!("public IP {}\n", report.public_ip.address())
}

#[cfg(test)]
mod test {
    use std::net::Ipv4Addr;

    use lib_core::{entities::public_ip::PublicIp, use_cases::report::Report};

    use crate::report_view::render;

    fn public_ip() -> PublicIp {
        PublicIp::try_from(Ipv4Addr::new(8, 8, 8, 8)).expect("public ip")
    }

    #[test]
    fn should_start_with_the_public_ip() {
        let report = Report {
            public_ip: public_ip(),
            records: vec![],
        };

        assert_eq!("public IP 8.8.8.8\n", render(&report));
    }
}

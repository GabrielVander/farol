#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomainName(String);

impl DomainName {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainNameError {
    Empty,
}

impl TryFrom<&str> for DomainName {
    type Error = DomainNameError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(DomainNameError::Empty);
        }
        Ok(Self(value.to_owned()))
    }
}

#[cfg(test)]
mod test {
    use crate::entities::domain_name::{DomainName, DomainNameError};

    #[test]
    fn should_accept_a_valid_domain() {
        let domain = DomainName::try_from("example.com").expect("valid domain");

        assert_eq!("example.com", domain.as_str());
    }

    #[test]
    fn should_reject_an_empty_domain() {
        assert_eq!(Err(DomainNameError::Empty), DomainName::try_from(""));
    }
}

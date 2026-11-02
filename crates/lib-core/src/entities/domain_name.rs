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
    TooLong,
    LabelTooLong,
}

impl TryFrom<&str> for DomainName {
    type Error = DomainNameError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        validate_length(value)?;
        value.split('.').try_for_each(validate_label)?;
        Ok(Self(value.to_owned()))
    }
}

const MAX_DOMAIN_LENGTH: usize = 253;
const MAX_LABEL_LENGTH: usize = 63;

fn validate_length(domain: &str) -> Result<(), DomainNameError> {
    if domain.is_empty() {
        return Err(DomainNameError::Empty);
    }
    if domain.len() > MAX_DOMAIN_LENGTH {
        return Err(DomainNameError::TooLong);
    }
    Ok(())
}

fn validate_label(label: &str) -> Result<(), DomainNameError> {
    if label.len() > MAX_LABEL_LENGTH {
        return Err(DomainNameError::LabelTooLong);
    }
    Ok(())
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

    #[test]
    fn should_reject_a_label_longer_than_63_characters() {
        let domain = format!("{}.com", "a".repeat(64));

        assert_eq!(
            Err(DomainNameError::LabelTooLong),
            DomainName::try_from(domain.as_str())
        );
    }

    #[test]
    fn should_reject_a_domain_longer_than_253_characters() {
        let label = "a".repeat(63);
        let domain = [label.as_str(); 4].join(".");

        assert_eq!(
            Err(DomainNameError::TooLong),
            DomainName::try_from(domain.as_str())
        );
    }
}

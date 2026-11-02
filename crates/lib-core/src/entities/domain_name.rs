use crate::entities::label::{LabelError, validate_labels};

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
    Label(LabelError),
}

impl From<LabelError> for DomainNameError {
    fn from(error: LabelError) -> Self {
        Self::Label(error)
    }
}

impl TryFrom<&str> for DomainName {
    type Error = DomainNameError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let value = value.strip_suffix('.').unwrap_or(value);
        validate_length(value)?;
        validate_labels(value)?;
        Ok(Self(value.to_ascii_lowercase()))
    }
}

const MAX_DOMAIN_LENGTH: usize = 253;

fn validate_length(domain: &str) -> Result<(), DomainNameError> {
    if domain.is_empty() {
        return Err(DomainNameError::Empty);
    }
    if domain.len() > MAX_DOMAIN_LENGTH {
        return Err(DomainNameError::TooLong);
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use crate::entities::{
        domain_name::{DomainName, DomainNameError},
        label::LabelError,
    };

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
            Err(DomainNameError::Label(LabelError::TooLong)),
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

    #[test]
    fn should_reject_invalid_characters() {
        assert_eq!(
            Err(DomainNameError::Label(LabelError::InvalidCharacter('_'))),
            DomainName::try_from("exa_mple.com")
        );
    }

    #[test]
    fn should_reject_an_empty_label() {
        assert_eq!(
            Err(DomainNameError::Label(LabelError::Empty)),
            DomainName::try_from("example..com")
        );
    }

    #[test]
    fn should_reject_a_label_starting_with_a_hyphen() {
        assert_eq!(
            Err(DomainNameError::Label(LabelError::HyphenAtEdge)),
            DomainName::try_from("-example.com")
        );
    }

    #[test]
    fn should_reject_a_label_ending_with_a_hyphen() {
        assert_eq!(
            Err(DomainNameError::Label(LabelError::HyphenAtEdge)),
            DomainName::try_from("example-.com")
        );
    }

    #[test]
    fn should_lowercase_the_domain() {
        let domain = DomainName::try_from("Example.COM").expect("valid domain");

        assert_eq!("example.com", domain.as_str());
    }

    #[test]
    fn should_drop_a_trailing_dot() {
        let domain = DomainName::try_from("example.com.").expect("valid domain");

        assert_eq!("example.com", domain.as_str());
    }
}

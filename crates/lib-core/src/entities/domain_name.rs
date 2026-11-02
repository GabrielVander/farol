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
    EmptyLabel,
    LabelTooLong,
    InvalidCharacter(char),
    HyphenAtLabelEdge,
}

impl TryFrom<&str> for DomainName {
    type Error = DomainNameError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        validate_length(value)?;
        value.split('.').try_for_each(validate_label)?;
        Ok(Self(value.to_ascii_lowercase()))
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
    validate_label_length(label)?;
    validate_label_characters(label)?;
    validate_label_edges(label)
}

fn validate_label_length(label: &str) -> Result<(), DomainNameError> {
    if label.is_empty() {
        return Err(DomainNameError::EmptyLabel);
    }
    if label.len() > MAX_LABEL_LENGTH {
        return Err(DomainNameError::LabelTooLong);
    }
    Ok(())
}

fn validate_label_characters(label: &str) -> Result<(), DomainNameError> {
    match label.chars().find(|c| !is_label_character(*c)) {
        Some(invalid) => Err(DomainNameError::InvalidCharacter(invalid)),
        None => Ok(()),
    }
}

fn validate_label_edges(label: &str) -> Result<(), DomainNameError> {
    if label.starts_with('-') || label.ends_with('-') {
        return Err(DomainNameError::HyphenAtLabelEdge);
    }
    Ok(())
}

fn is_label_character(character: char) -> bool {
    character.is_ascii_alphanumeric() || character == '-'
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

    #[test]
    fn should_reject_invalid_characters() {
        assert_eq!(
            Err(DomainNameError::InvalidCharacter('_')),
            DomainName::try_from("exa_mple.com")
        );
    }

    #[test]
    fn should_reject_an_empty_label() {
        assert_eq!(
            Err(DomainNameError::EmptyLabel),
            DomainName::try_from("example..com")
        );
    }

    #[test]
    fn should_reject_a_label_starting_with_a_hyphen() {
        assert_eq!(
            Err(DomainNameError::HyphenAtLabelEdge),
            DomainName::try_from("-example.com")
        );
    }

    #[test]
    fn should_reject_a_label_ending_with_a_hyphen() {
        assert_eq!(
            Err(DomainNameError::HyphenAtLabelEdge),
            DomainName::try_from("example-.com")
        );
    }

    #[test]
    fn should_lowercase_the_domain() {
        let domain = DomainName::try_from("Example.COM").expect("valid domain");

        assert_eq!("example.com", domain.as_str());
    }
}

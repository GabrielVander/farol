use crate::entities::label::{LabelError, validate_labels};

const APEX: &str = "@";
const WILDCARD: &str = "*";
const WILDCARD_PREFIX: &str = "*.";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordName(String);

impl RecordName {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<&str> for RecordName {
    type Error = LabelError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        validate(value)?;

        Ok(Self(value.to_ascii_lowercase()))
    }
}

fn validate(value: &str) -> Result<(), LabelError> {
    match value {
        APEX | WILDCARD => Ok(()),
        _ => validate_labels(value.strip_prefix(WILDCARD_PREFIX).unwrap_or(value)),
    }
}

#[cfg(test)]
mod test {
    use crate::entities::{label::LabelError, record_name::RecordName};

    #[test]
    fn should_accept_the_apex() {
        let name = RecordName::try_from("@").expect("valid record name");

        assert_eq!("@", name.as_str());
    }

    #[test]
    fn should_accept_a_wildcard() {
        let name = RecordName::try_from("*").expect("valid record name");

        assert_eq!("*", name.as_str());
    }

    #[test]
    fn should_accept_a_wildcard_followed_by_labels() {
        let name = RecordName::try_from("*.home").expect("valid record name");

        assert_eq!("*.home", name.as_str());
    }

    #[test]
    fn should_reject_a_wildcard_inside_a_label() {
        assert_eq!(
            Err(LabelError::InvalidCharacter('*')),
            RecordName::try_from("ho*me")
        );
    }

    #[test]
    fn should_reject_a_wildcard_that_is_not_leftmost() {
        assert_eq!(
            Err(LabelError::InvalidCharacter('*')),
            RecordName::try_from("home.*")
        );
    }

    #[test]
    fn should_reject_two_wildcards() {
        assert_eq!(
            Err(LabelError::InvalidCharacter('*')),
            RecordName::try_from("*.*")
        );
    }

    #[test]
    fn should_reject_a_wildcard_followed_by_nothing() {
        assert_eq!(Err(LabelError::Empty), RecordName::try_from("*."));
    }

    #[test]
    fn should_reject_an_invalid_label() {
        assert_eq!(
            Err(LabelError::InvalidCharacter('_')),
            RecordName::try_from("ho_me")
        );
    }

    #[test]
    fn should_accept_nested_labels() {
        let name = RecordName::try_from("home.lab").expect("valid record name");

        assert_eq!("home.lab", name.as_str());
    }

    #[test]
    fn should_lowercase_the_record_name() {
        let name = RecordName::try_from("Home").expect("valid record name");

        assert_eq!("home", name.as_str());
    }
}

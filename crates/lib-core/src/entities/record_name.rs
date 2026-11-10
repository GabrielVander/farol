use crate::entities::label::{LabelError, validate_labels};

const APEX: &str = "@";
const WILDCARD: &str = "*";

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
        if value != APEX && value != WILDCARD {
            validate_labels(value)?;
        }
        Ok(Self(value.to_ascii_lowercase()))
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

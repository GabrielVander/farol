use std::convert::Infallible;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordName(String);

impl RecordName {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<&str> for RecordName {
    type Error = Infallible;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(Self(value.to_owned()))
    }
}

#[cfg(test)]
mod test {
    use crate::entities::record_name::RecordName;

    #[test]
    fn should_accept_the_apex() {
        let name = RecordName::try_from("@").expect("valid record name");

        assert_eq!("@", name.as_str());
    }
}

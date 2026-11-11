pub const HOSTINGER_TOKEN_VARIABLE: &str = "FAROL_HOSTINGER_TOKEN";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissingToken;

/// The Hostinger API token, read from the environment because other users can read arguments.
pub fn hostinger_token(
    environment: impl Fn(&str) -> Option<String>,
) -> Result<String, MissingToken> {
    environment(HOSTINGER_TOKEN_VARIABLE)
        .filter(|token| !token.trim().is_empty())
        .ok_or(MissingToken)
}

#[cfg(test)]
mod test {
    use crate::infrastructure::token::{MissingToken, hostinger_token};

    #[test]
    fn should_read_the_hostinger_token_from_the_environment() {
        let environment = |key: &str| (key == "FAROL_HOSTINGER_TOKEN").then(|| "secret".to_owned());

        assert_eq!(Ok("secret".to_owned()), hostinger_token(environment));
    }

    #[test]
    fn should_report_a_missing_token() {
        assert_eq!(Err(MissingToken), hostinger_token(|_| None));
    }

    #[test]
    fn should_treat_a_blank_token_as_missing() {
        assert_eq!(
            Err(MissingToken),
            hostinger_token(|_| Some("  ".to_owned()))
        );
    }
}

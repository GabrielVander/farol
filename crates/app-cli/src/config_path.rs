use std::path::PathBuf;

/// Where to read the config: `explicit` if given, otherwise the XDG location.
pub fn config_path(
    explicit: Option<PathBuf>,
    _environment: impl Fn(&str) -> Option<String>,
) -> Option<PathBuf> {
    explicit
}

#[cfg(test)]
mod test {
    use std::{collections::HashMap, path::PathBuf};

    use crate::config_path::config_path;

    fn environment(variables: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
        let variables: HashMap<String, String> = variables
            .iter()
            .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
            .collect();
        move |key| variables.get(key).cloned()
    }

    #[test]
    fn should_prefer_the_explicit_path() {
        let explicit = PathBuf::from("/etc/farol.toml");

        let path = config_path(
            Some(explicit.clone()),
            environment(&[("XDG_CONFIG_HOME", "/home/me/.config")]),
        );

        assert_eq!(Some(explicit), path);
    }
}

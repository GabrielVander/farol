use std::path::PathBuf;

/// Where to read the config: `explicit` if given, otherwise the XDG location.
pub fn config_path(
    explicit: Option<PathBuf>,
    environment: impl Fn(&str) -> Option<String>,
) -> Option<PathBuf> {
    explicit.or_else(|| Some(config_home(environment)?.join(FILE_IN_CONFIG_HOME)))
}

// The XDG Base Directory spec's default when XDG_CONFIG_HOME is unset.
fn config_home(environment: impl Fn(&str) -> Option<String>) -> Option<PathBuf> {
    environment("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| Some(PathBuf::from(environment("HOME")?).join(".config")))
}

const FILE_IN_CONFIG_HOME: &str = "farol/config.toml";

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

    #[test]
    fn should_use_the_xdg_config_home() {
        let path = config_path(
            None,
            environment(&[
                ("XDG_CONFIG_HOME", "/home/me/.config"),
                ("HOME", "/home/other"),
            ]),
        );

        assert_eq!(
            Some(PathBuf::from("/home/me/.config/farol/config.toml")),
            path
        );
    }

    #[test]
    fn should_fall_back_to_the_config_folder_in_home() {
        let path = config_path(None, environment(&[("HOME", "/home/me")]));

        assert_eq!(
            Some(PathBuf::from("/home/me/.config/farol/config.toml")),
            path
        );
    }

    #[test]
    fn should_find_no_path_without_a_home() {
        assert_eq!(None, config_path(None, environment(&[])));
    }
}

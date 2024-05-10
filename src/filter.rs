use semver::Version;
use serde::Deserialize;

use crate::config::Loader;

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct Config {
    loader: Loader,
    game_version: Version,
}

#[cfg(test)]
mod config_deserializer {
    use super::Config;
    use crate::config::{Loaders, Loader};

    #[test]
    fn valid() {
        let config = toml::from_str::<Config>(
            r#"
            loader = "forge"
            version = "1.20.1"
"#,
        );

        assert_eq!(
            Ok(Config {
                loader: Loader::any(Loaders::Forge),
                game_version: semver::Version::new(1, 20, 1)
            }),
            config
        );
    }

    #[test]
    fn missing_param() {
        let config = toml::from_str::<Config>(
            r#"
            loader = "quilt"
"#,
        );

        assert!(config.is_err());

        let config = toml::from_str::<Config>(
            r#"
            version = "1.20"
"#,
        );

        assert!(config.is_err());
    }

    #[test]
    fn invalid_param() {
        let config = toml::from_str::<Config>(
            r#"
            loader = 123
            version = "1.20"
"#,
        );

        assert!(config.is_err());

        let config = toml::from_str::<Config>(
            r#"
            loader = "forge"
            version = "certainly not a version!"
"#,
        );

        assert!(config.is_err());
    }
}

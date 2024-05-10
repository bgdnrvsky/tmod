use semver::Version;
use serde::Deserialize;

use crate::loader::Loader;

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct Config {
    loader: Loader,
    game_version: Version,
}

#[cfg(test)]
mod config_deserializer_tests {
    use semver::VersionReq;

    use super::Config;
    use crate::loader::{Loader, Loaders};

    #[test]
    fn valid() {
        let config = toml::from_str::<Config>(
            r#"
            game_version = "1.20.1"

            [loader]
            kind = "forge"
            version = "47.2.0"
"#,
        );

        assert_eq!(
            config,
            Ok(Config {
                loader: Loader::explicit(Loaders::Forge, VersionReq::parse("47.2.0").unwrap())
                    .unwrap(),
                game_version: semver::Version::new(1, 20, 1)
            })
        );
    }

    #[test]
    fn missing() {
        let config = toml::from_str::<Config>(
            r#"
            [loader]
            kind = "forge"
            version = "47.2.0"
"#,
        );

        assert!(config.is_err());
    }
}

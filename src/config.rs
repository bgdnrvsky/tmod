use crate::version::SingleVersion as Version;
use serde::Deserialize;

use crate::loader::Loader;

/// Basic configuration including: loader (forge or fabric) and the goal game version
///
/// Example configuration:
/// ```toml
/// game_version = "1.20.1"
///
/// [loader]
/// kind = "forge"
/// version = "47.2.0"
/// ```
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct Config {
    /// Minecraft mod loader (forge or fabric)
    loader: Loader,
    /// Minecraft version target
    game_version: Version,
}

#[cfg(test)]
mod config_deserializer_tests {
    use std::str::FromStr;

    use super::Config;
    use crate::loader::{Loader, Loaders};
    use crate::version::maven::Version;
    use crate::version::SingleVersion;

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
                loader: Loader::new(
                    Loaders::Forge,
                    SingleVersion::Forge(Version::from_str("47.2.0").unwrap())
                )
                .unwrap(),
                game_version: SingleVersion::Forge(Version::from_str("1.20.1").unwrap())
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

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
    use super::Config;

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

        assert!(config.is_ok());
    }

    #[test]
    #[should_panic]
    fn missing() {
        toml::from_str::<Config>(
            r#"
            [loader]
            kind = "forge"
            version = "47.2.0"
            "#,
        )
        .expect("Should fail since game version is missing");
    }
}

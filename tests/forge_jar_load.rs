use std::str::FromStr;

use tmod::version::maven::Version;
use tmod::{minecraft_mod::Mod, version::SingleVersion};

#[test]
fn load_forge() -> anyhow::Result<()> {
    let btp = Mod::from_jar("tests/btp.jar")?;

    assert_eq!(btp.id(), "betterthirdperson");

    assert_eq!(
        btp.version(),
        SingleVersion::Forge(Version::from_str("1.9.0").unwrap())
    );

    assert!(btp.dependencies().is_some_and(|deps| deps.len() == 2));
    assert!(btp.incompatibilities().is_none());

    Ok(())
}

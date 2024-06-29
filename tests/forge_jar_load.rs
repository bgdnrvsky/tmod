use std::str::FromStr;

use tmod::version::maven::Version;
use tmod::{
    minecraft_mod::Mod,
    version::{MultiVersion, SingleVersion},
};

#[test]
fn load_forge() -> anyhow::Result<()> {
    let btp = Mod::from_jar("tests/btp.jar")?;

    assert_eq!(btp.id(), "betterthirdperson");

    assert_eq!(
        btp.version(),
        SingleVersion::Forge(Version::from_str("1.9.0").unwrap())
    );

    assert!(btp.dependencies().is_some_and(|deps| deps.is_empty()));
    assert!(btp.incompatibilities().is_none());

    assert_eq!(
        btp.minecraft_version_needed(),
        MultiVersion::Forge(tmod::version::maven::VersionRange::from_str("[1.20,1.21)")?)
    );

    assert_eq!(
        btp.loader_version_needed(),
        MultiVersion::Forge(tmod::version::maven::VersionRange::from_str("[46,)")?)
    );

    Ok(())
}

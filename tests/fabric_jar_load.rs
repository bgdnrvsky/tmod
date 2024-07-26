use std::str::FromStr;

use tmod::{
    minecraft_mod::Mod,
    version::{
        fabric::{Version, VersionReq},
        MultiVersion, SingleVersion,
    },
};

#[test]
fn load_fabric() -> anyhow::Result<()> {
    let sodium = Mod::from_jar("tests/jars/sodium.jar")?;

    assert_eq!(sodium.id(), "sodium");

    assert_eq!(
        sodium.version(),
        SingleVersion::Fabric(Version::from_str("0.5.8+mc1.20.4")?)
    );

    assert!(!sodium.dependencies().is_empty());
    assert!(!sodium.incompatibilities().is_empty());

    assert_eq!(
        sodium.minecraft_version_needed(),
        MultiVersion::Fabric(VersionReq::any())
    );

    assert_eq!(
        sodium.loader_version_needed(),
        MultiVersion::Fabric(VersionReq::from_str(">=0.12.0")?)
    );

    Ok(())
}

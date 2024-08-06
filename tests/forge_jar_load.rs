use std::str::FromStr;

use jars::{jar, JarOptionBuilder};

use tmod::{
    jar::forge::ForgeMod as Mod,
    version::maven::{Version, VersionRange},
};

#[test]
fn load_forge() -> anyhow::Result<()> {
    let jar = jar(
        "tests/jars/btp.jar",
        JarOptionBuilder::builder().keep_meta_info().build(),
    )?;

    let btp = Mod::try_from(jar)?;

    assert_eq!(btp.slug(), "betterthirdperson");

    assert_eq!(btp.version(), &Version::from_str("1.9.0")?);

    assert!(btp.dependencies().is_empty());

    assert_eq!(
        btp.minecraft_version_needed(),
        &VersionRange::from_str("[1.20,1.21)")?
    );

    assert_eq!(
        btp.loader_version_needed(),
        &VersionRange::from_str("[46,)")?
    );

    Ok(())
}

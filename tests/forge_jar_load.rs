use tmod::version::maven::{Version, VersionItem};
use tmod::{minecraft_mod::Mod, version::SingleVersion};

#[test]
fn load_forge() -> anyhow::Result<()> {
    let btp = Mod::from_jar("tests/btp.jar")?;

    assert_eq!(btp.id(), "betterthirdperson");

    assert_eq!(
        btp.version(),
        SingleVersion::Forge(Version::new(vec![
            VersionItem::Numeric(1),
            VersionItem::Numeric(9),
            VersionItem::Numeric(0),
        ]))
    );

    // TODO: Remove minecraft and forge
    // from general dependencies and make
    // it separate fields in `Mod` struct

    assert!(btp.dependencies().is_some_and(|deps| deps.len() == 2));
    assert!(btp.incompatibilities().is_none());

    Ok(())
}

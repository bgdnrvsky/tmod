use jars::{jar, JarOptionBuilder};

use tmod::jar::fabric::FabricMod as Mod;

#[test]
fn load_fabric() -> anyhow::Result<()> {
    let jar = jar(
        "tests/test_pool/locals/sodium.jar",
        JarOptionBuilder::builder().keep_meta_info().build(),
    )?;

    let sodium = Mod::try_from(jar)?;

    assert_eq!(sodium.slug(), "sodium");
    assert_eq!(sodium.version(), "0.5.8+mc1.20.4");
    assert!(sodium.dependencies().is_empty()); // It does contain internal dependencies though
    assert!(!sodium.incompatibilities().is_empty());
    assert_eq!(sodium.minecraft_version_needed(), None);
    assert_eq!(sodium.loader_version_needed(), Some(">=0.12.0"));

    Ok(())
}

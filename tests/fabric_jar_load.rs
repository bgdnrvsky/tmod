use tmod::minecraft_mod::Mod;

#[test]
#[ignore]
fn load_fabric() -> anyhow::Result<()> {
    let _sodium = Mod::from_jar("tests/sodium.jar")?;

    Ok(())
}

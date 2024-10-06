use anyhow::Context;
use tmod::pool::Pool;

#[test]
fn read_pool() -> anyhow::Result<()> {
    let pool = Pool::read("tests/test_pool").context("Reading pool")?;

    let remotes = pool.remotes;
    assert!(remotes.contains_key("foo"));
    assert!(remotes.contains_key("bar"));

    let locals = pool.locals;
    assert!(locals.iter().any(|r#mod| r#mod.name() == "sodium"));
    assert!(locals
        .iter()
        .any(|r#mod| r#mod.name() == "betterthirdperson"));

    Ok(())
}

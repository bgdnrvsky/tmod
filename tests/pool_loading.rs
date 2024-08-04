use anyhow::Context;
use tmod::pool::pool::Pool;

#[test]
fn read_pool() -> anyhow::Result<()> {
    let pool = Pool::new("tests/test_pool").context("Reading pool")?;

    let remotes = pool.remotes();
    assert!(remotes.contains_key("foo"));
    assert!(remotes.contains_key("bar"));

    assert!(pool.locals().is_empty());

    Ok(())
}

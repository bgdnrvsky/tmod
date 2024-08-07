use anyhow::Context;
use tmod::pool::pool::Pool;

#[test]
fn read_pool() -> anyhow::Result<()> {
    let pool = Pool::new("tests/test_pool").context("Reading pool")?;

    let remotes = pool.remotes();
    assert!(remotes.contains("foo"));
    assert!(remotes.contains("bar"));

    assert!(pool.locals().is_empty());

    Ok(())
}

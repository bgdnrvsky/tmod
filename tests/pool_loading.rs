use anyhow::Context;
use tmod::pool::Pool;

#[test]
fn read_pool() -> anyhow::Result<()> {
    let pool = Pool::read("tests/test_pool").context("Reading pool")?;

    let remotes = pool.manually_added;
    assert!(remotes.contains("foo"));
    assert!(remotes.contains("bar"));

    Ok(())
}

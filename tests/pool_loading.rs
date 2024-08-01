use anyhow::Context;
use tmod::pool::pool::Pool;

#[test]
fn read_pool() -> anyhow::Result<()> {
    let _pool = Pool::new("tests/test_pool").context("Reading pool")?;

    Ok(())
}

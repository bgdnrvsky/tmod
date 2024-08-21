use std::ffi::OsStr;

use anyhow::Context;
use tmod::pool::Pool;

#[test]
fn read_pool() -> anyhow::Result<()> {
    let pool = Pool::new("tests/test_pool").context("Reading pool")?;

    let remotes = pool.remotes();
    assert!(remotes.contains("foo"));
    assert!(remotes.contains("bar"));

    let locals = pool.locals();
    assert!(locals.contains_key(OsStr::new("sodium.jar")));
    assert!(locals.contains_key(OsStr::new("btp.jar")));

    Ok(())
}

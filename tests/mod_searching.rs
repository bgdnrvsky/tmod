use tmod::fetcher::searcher::Searcher;

#[test]
fn mod_by_id() -> anyhow::Result<()> {
    let searcher = Searcher::new();

    let alexs_mobs = searcher.search_mod_by_id(426558)?;
    assert_eq!(alexs_mobs.slug(), "alexs-mobs");

    let jei = searcher.search_mod_by_id(238222)?;
    assert_eq!(jei.slug(), "jei");

    Ok(())
}

#[test]
fn mod_by_slug() -> anyhow::Result<()> {
    let searcher = Searcher::new();

    let alexs_mobs = searcher.search_mod_by_name("alexs-mobs")?;
    assert_eq!(alexs_mobs.count(), 1);

    let jei = searcher.search_mod_by_name("jei")?;
    assert_eq!(jei.count(), 1);

    Ok(())
}

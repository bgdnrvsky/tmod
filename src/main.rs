use crate::loader::Loader;

mod config;
mod fetchers;
mod loader;

fn main() -> anyhow::Result<()> {
    println!(
        "{:#?}",
        toml::from_str::<Loader>(
            r#"
            kind = "forge"
            version = "47.2.2"
"#,
        )
    );

    Ok(())
}

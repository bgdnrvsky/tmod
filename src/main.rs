mod fetchers;
mod filter;
mod config;

fn main() -> anyhow::Result<()> {
    println!("{:?}", fetchers::get_minecraft_id());

    Ok(())
}

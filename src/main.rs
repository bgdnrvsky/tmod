mod fetchers;
mod token;
mod filter;
mod loader;

fn main() -> anyhow::Result<()> {
    println!("{:?}", fetchers::get_minecraft_id());

    Ok(())
}

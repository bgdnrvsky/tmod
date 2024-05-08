mod fetchers;
mod token;
mod filter;

fn main() -> anyhow::Result<()> {
    println!("{:?}", fetchers::get_minecraft_id());

    Ok(())
}

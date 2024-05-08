mod fetchers;
mod token;

fn main() -> anyhow::Result<()> {
    println!("{:?}", fetchers::get_minecraft_id());

    Ok(())
}

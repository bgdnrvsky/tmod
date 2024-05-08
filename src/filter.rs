use semver::Version;

#[derive(Debug, Clone)]
pub enum Loaders {
    Forge,
    Fabric,
    Quilt,
    NeoForge,
}

#[derive(Debug, Clone)]
pub struct SearchFilter {
    loader: Loaders,
    game_version: Version,
}

pub mod read;
pub mod write;

use std::path::{Path, PathBuf};

use super::Pool;

impl Pool {
    pub fn root_path(&self) -> &Path {
        &self.path
    }

    pub fn remotes_path(&self) -> PathBuf {
        self.root_path().join("Tmod.json")
    }

    pub fn locks_path(&self) -> PathBuf {
        self.root_path().join("Tmod.lock")
    }
}

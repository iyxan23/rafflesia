use crate::core::manifest;
use crate::core::manifest::Manifest;
use anyhow::Result;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Project {
    pub working_directory: PathBuf,
    pub manifest: Manifest,
}

impl Project {
    pub fn new(working_directory: PathBuf, manifest: Manifest) -> Self {
        Self {
            working_directory,
            manifest,
        }
    }

    pub fn find_project() -> Result<Self> {
        // todo: search for parent directories?
        let current_dir = std::env::current_dir()?;
        let manifest_file = current_dir.join(Path::new("swproj.toml"));
        let manifest = manifest::parse_manifest(manifest_file)?;

        Ok(Self {
            working_directory: current_dir,
            manifest,
        })
    }
}

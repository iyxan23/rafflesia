use std::{fs, collections::HashMap};
use std::path::PathBuf;
use anyhow::{Context, Result};
use serde::{Serialize, Deserialize};
use toml::value::Datetime;

// The structure of a swproj.toml file
#[derive(Serialize, Deserialize)]
pub struct Manifest {
    pub project: ProjectTable,
    pub activity: HashMap<String, ActivityTable>,
    pub library: Option<LibraryTable>
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ProjectTable {
    pub id: Option<u16>,
    pub name: String,
    pub workspace_name: Option<String>,
    pub package: String,
    pub version_code: u16,
    pub version_name: String,
    pub time_created: Datetime,
    pub sw_ver: u16,
    pub colors: Option<ColorsTable>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ColorsTable {
    pub primary: String,
    pub primary_dark: String,
    pub accent: String,
    pub control_normal: String,
    pub control_highlight: String,
}

#[derive(Serialize, Deserialize)]
pub struct ActivityTable {
    pub logic: String,
    pub layout: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct LibraryTable {
    pub compat: Option<CompatLibraryTable>,
    pub firebase: Option<FirebaseLibraryTable>,
    pub admob: Option<AdMobLibraryTable>,
    pub google_map: Option<GoogleMapLibraryTable>,
}

#[derive(Serialize, Deserialize)]
pub struct CompatLibraryTable {
    pub enabled: bool
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct FirebaseLibraryTable {
    pub enabled: bool,
    pub api_key: String
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct AdMobLibraryTable {
    pub enabled: bool,
    pub test_devices: Vec<String>
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct GoogleMapLibraryTable {
    pub enabled: bool,
    pub api_key: String
}

pub fn parse_manifest_str(content: &str) -> Result<Manifest> {
    Ok(toml::from_str::<Manifest>(content)?)
}

pub fn parse_manifest(path: PathBuf) -> Result<Manifest> {
    parse_manifest_str(
        &fs::read_to_string(path.clone())
            .context(format!("Failed to parse manifest at path {}", path.display()))?
    )
}
use std::{fs, collections::HashMap};
use std::num::ParseIntError;
use std::path::PathBuf;
use anyhow::{Context, Result};
use chrono::DateTime;
use serde::{Serialize, Deserialize};
use swrs::api::{Colors, Libraries, Metadata, Resources, SketchwareProject};
use swrs::api::library::{AdMob, Firebase, GoogleMap};
use swrs::color::Color;
use swrs::parser::library::AdUnit;
use toml::value::Datetime;
use thiserror::Error;

// The structure of a swproj.toml file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub project: ProjectTable,
    pub activity: HashMap<String, ActivityTable>,
    pub library: Option<LibraryTable>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ProjectTable {
    pub id: Option<u16>,
    pub name: String,
    pub workspace_name: Option<String>,
    pub package: String,
    pub version_code: u16,
    pub version_name: String,
    pub time_created: Datetime,
    pub sw_ver: u8,
    pub colors: Option<ColorsTable>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ColorsTable {
    pub primary: String,
    pub primary_dark: String,
    pub accent: String,
    pub control_normal: String,
    pub control_highlight: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityTable {
    pub logic: String,
    pub layout: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct LibraryTable {
    pub compat: Option<CompatLibraryTable>,
    pub firebase: Option<FirebaseLibraryTable>,
    pub admob: Option<AdMobLibraryTable>,
    pub google_map: Option<GoogleMapLibraryTable>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatLibraryTable {
    pub enabled: bool
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct FirebaseLibraryTable {
    pub enabled: bool,
    pub api_key: String,
    pub project_id: String,
    pub app_id: String,
    pub storage_bucket: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct AdMobLibraryTable {
    pub enabled: bool,
    // name -> id
    pub ad_units: HashMap<String, String>,
    pub test_devices: Vec<String>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl TryInto<SketchwareProject> for Manifest {
    type Error = ProjectGenerationError;

    fn try_into(self) -> Result<SketchwareProject, Self::Error> {
        Ok(SketchwareProject::new(
            Metadata {
                local_id: self.project.id.unwrap_or(600),
                workspace_name: self.project.workspace_name
                    .unwrap_or_else(|| self.project.name.clone()),
                name: self.project.name,
                package_name: self.project.package,
                time_created: toml_datetime_to_timestamp(self.project.time_created),
                sketchware_version: self.project.sw_ver as u8,
                version_name: self.project.version_name,
                version_code: self.project.version_code
            },
            {
                if let Some(colors) = self.project.colors {
                    Colors {
                        color_primary: Color::parse_hex(&colors.primary)
                            .map_err(|err| ProjectGenerationError::ColorParseError {
                                name: "colorPrimary".to_string(),
                                source: err
                            })?,
                        color_primary_dark: Color::parse_hex(&colors.primary_dark)
                            .map_err(|err| ProjectGenerationError::ColorParseError {
                                name: "colorPrimaryDark".to_string(),
                                source: err
                            })?,
                        color_accent: Color::parse_hex(&colors.accent)
                            .map_err(|err| ProjectGenerationError::ColorParseError {
                                name: "colorAccent".to_string(),
                                source: err
                            })?,
                        color_control_normal: Color::parse_hex(&colors.control_normal)
                            .map_err(|err| ProjectGenerationError::ColorParseError {
                                name: "colorControlNormal".to_string(),
                                source: err
                            })?,
                        color_control_highlight: Color::parse_hex(&colors.control_highlight)
                            .map_err(|err| ProjectGenerationError::ColorParseError {
                                name: "colorControlHighlight".to_string(),
                                source: err
                            })?,
                    }
                } else {
                    // use default colors
                    Colors {
                        color_primary: Color::from(0xff008dcd),
                        color_primary_dark: Color::from(0xff0084c2),
                        color_accent: Color::from(0xff008dcd),
                        color_control_normal: Color::from(0xff57beee),
                        color_control_highlight: Color::from(0x20008dcd)
                    }
                }
            },
            vec![],
            vec![],
            {
                if let Some(libraries) = self.library {
                    Libraries {
                        app_compat_enabled: libraries.compat
                            .map(|c| c.enabled)
                            .unwrap_or(false),

                        firebase: if let Some(firebase) = libraries.firebase {
                            if !firebase.enabled { None }
                            else {
                                Some(Firebase {
                                    project_id: firebase.project_id,
                                    app_id: firebase.app_id,
                                    api_key: firebase.api_key,
                                    storage_bucket: firebase.storage_bucket
                                })
                            }
                        } else { None },

                        ad_mob: if let Some(admob) = libraries.admob {
                            if !admob.enabled { None }
                            else {
                                Some(AdMob {
                                    ad_units: admob.ad_units
                                        .into_iter()
                                        .map(|(name, id)| AdUnit { id, name })
                                        .collect(),
                                    test_devices: admob.test_devices,
                                })
                            }
                        } else { None },

                        google_map: if let Some(google_map) = libraries.google_map {
                            if !google_map.enabled { None }
                            else {
                                Some(GoogleMap { api_key: google_map.api_key })
                            }
                        } else { None }
                    }
                } else {
                    // default
                    Libraries { app_compat_enabled: false, firebase: None, ad_mob: None, google_map: None }
                }
            },
            Resources::new_empty()
        ))
    }
}

#[derive(Debug, Error)]
pub enum ProjectGenerationError {
    #[error("failed to parse color {name}")]
    ColorParseError {
        name: String,
        source: ParseIntError
    }
}

// reference: https://docs.rs/toml/latest/toml/value/struct.Datetime.html
fn toml_datetime_to_timestamp(datetime: Datetime) -> u64 {
    DateTime::parse_from_rfc3339(&datetime.to_string()).unwrap().timestamp() as u64

    // this crap is way too complicated
    /*
    match datetime {
        Datetime { date: Some(date), time: Some(time), offset: Some(offset) } => {
            let offset = match offset {
                Offset::Z => Utc::fix(),
                Offset::Custom { hours, minutes } =>
                    FixedOffset::east_opt((hours as i32 * 60 + minutes as i32) * 60)
                        .unwrap()
            };

            // how to apply this offset aa

            NaiveDate::from_ymd(date.year as i32, date.month as u32, date.day as u32)
                .and_hms_nano(
                    time.hour as u32, time.minute as u32, time.second as u32,
                    time.nanosecond as u32
                )
                .timestamp() as u64
        }

        Datetime { date: Some(date), time: Some(time), offset: None } => {

        }

        Datetime { date: Some(date), time: None, offset: None } => {

        }

        Datetime { date: None, time: Some(time), offset: None } => {

        }

        _ => unreachable!()
    }
     */
}
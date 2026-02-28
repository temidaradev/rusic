use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum MusicSource {
    #[default]
    Local,
    Jellyfin,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SortOrder {
    Title,
    Artist,
    Album,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub server: Option<JellyfinServer>,
    #[serde(default)]
    pub active_source: MusicSource,
    pub music_directory: PathBuf,
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_device_id")]
    pub device_id: String,
    #[serde(default = "default_discord_presence")]
    pub discord_presence: Option<bool>,
    #[serde(default = "default_sort_order")]
    pub sort_order: SortOrder,
    #[serde(default)]
    pub listen_counts: HashMap<String, u64>,
    #[serde(default)]
    pub musicbrainz_token: String,
    #[serde(default)]
    pub lastfm_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JellyfinServer {
    pub name: String,
    pub url: String,
    pub access_token: Option<String>,
    pub user_id: Option<String>,
}

fn default_theme() -> String {
    "default".to_string()
}

fn default_device_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

fn default_discord_presence() -> Option<bool> {
    Some(true)
}

fn default_sort_order() -> SortOrder {
    SortOrder::Title
}

impl Default for AppConfig {
    fn default() -> Self {
        let music_directory = directories::UserDirs::new()
            .and_then(|u| u.audio_dir().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("./assets"));
        Self {
            server: None,
            active_source: MusicSource::Local,
            music_directory,
            theme: default_theme(),
            device_id: default_device_id(),
            discord_presence: Some(true),
            sort_order: default_sort_order(),
            listen_counts: HashMap::new(),
            musicbrainz_token: String::new(),
            lastfm_token: String::new(),
        }
    }
}

impl Default for JellyfinServer {
    fn default() -> Self {
        Self {
            name: String::new(),
            url: String::new(),
            access_token: None,
            user_id: None,
        }
    }
}

impl AppConfig {
    pub fn load(path: &Path) -> Self {
        if !path.exists() {
            return Self::default();
        }
        match fs::read_to_string(path) {
            Ok(data) => match serde_json::from_str(&data) {
                Ok(config) => config,
                Err(e) => {
                    eprintln!("Failed to parse config at {:?}: {}", path, e);
                    Self::default()
                }
            },
            Err(e) => {
                eprintln!("Failed to read config at {:?}: {}", path, e);
                Self::default()
            }
        }
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                eprintln!("Failed to create config directory {:?}: {}", parent, e);
                return Err(e);
            }
        }
        let data = match serde_json::to_string_pretty(self) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Failed to serialize config: {}", e);
                return Err(std::io::Error::new(std::io::ErrorKind::Other, e));
            }
        };
        if let Err(e) = fs::write(path, data) {
            eprintln!("Failed to write config to {:?}: {}", path, e);
            return Err(e);
        }
        Ok(())
    }
}

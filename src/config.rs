use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub music_directory: PathBuf,
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default)]
    pub last_folder_name: Option<String>,
    #[serde(default)]
    pub has_loaded_folder: bool,
}

fn default_theme() -> String {
    "default".to_string()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            music_directory: PathBuf::from("./assets"),
            theme: default_theme(),
            last_folder_name: None,
            has_loaded_folder: false,
        }
    }
}

impl AppConfig {
    const STORAGE_KEY: &'static str = "rusic_config";

    pub fn load(_path: &Path) -> Self {
        use gloo_storage::{LocalStorage, Storage};
        LocalStorage::get(Self::STORAGE_KEY).unwrap_or_default()
    }

    pub fn save(&self, _path: &Path) -> std::io::Result<()> {
        use gloo_storage::{LocalStorage, Storage};
        LocalStorage::set(Self::STORAGE_KEY, self).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::Other, format!("Storage error: {:?}", e))
        })
    }

    pub fn load_from_storage() -> Self {
        use gloo_storage::{LocalStorage, Storage};
        LocalStorage::get(Self::STORAGE_KEY).unwrap_or_default()
    }

    pub fn save_to_storage(&self) -> Result<(), String> {
        use gloo_storage::{LocalStorage, Storage};
        LocalStorage::set(Self::STORAGE_KEY, self).map_err(|e| format!("Storage error: {:?}", e))
    }
}

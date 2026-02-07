use gloo_storage::{LocalStorage, Storage};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Album {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub genre: String,
    pub year: u16,
    pub cover_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Track {
    pub path: PathBuf,
    pub album_id: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration: u64,
    pub track_number: Option<u32>,
    pub disc_number: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Library {
    #[serde(default)]
    pub root_path: PathBuf,
    pub tracks: Vec<Track>,
    pub albums: Vec<Album>,
}

impl Library {
    const LIBRARY_KEY: &'static str = "rusic_library";

    pub fn new(root_path: PathBuf) -> Self {
        Self {
            root_path,
            ..Default::default()
        }
    }

    pub fn load(_path: &Path) -> std::io::Result<Self> {
        LocalStorage::get(Self::LIBRARY_KEY).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Storage error: {:?}", e),
            )
        })
    }

    pub fn save(&self, _path: &Path) -> std::io::Result<()> {
        LocalStorage::set(Self::LIBRARY_KEY, self).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::Other, format!("Storage error: {:?}", e))
        })
    }

    pub fn add_track(&mut self, track: Track) {
        if let Some(index) = self.tracks.iter().position(|t| t.path == track.path) {
            self.tracks[index] = track;
        } else {
            self.tracks.push(track);
        }
    }

    pub fn add_album(&mut self, album: Album) {
        if let Some(index) = self.albums.iter().position(|a| a.id == album.id) {
            let mut new_album = album;
            if new_album.cover_path.is_none() {
                new_album.cover_path = self.albums[index].cover_path.clone();
            }
            self.albums[index] = new_album;
        } else {
            self.albums.push(album);
        }
    }

    pub fn remove_track(&mut self, path: &Path) {
        self.tracks.retain(|t| t.path != path);
    }

    pub fn clear(&mut self) {
        self.tracks.clear();
        self.albums.clear();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Playlist {
    pub id: String,
    pub name: String,
    pub tracks: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct PlaylistStore {
    pub playlists: Vec<Playlist>,
}

impl PlaylistStore {
    const PLAYLIST_KEY: &'static str = "rusic_playlists";

    pub fn load(_path: &Path) -> std::io::Result<Self> {
        LocalStorage::get(Self::PLAYLIST_KEY).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Storage error: {:?}", e),
            )
        })
    }

    pub fn save(&self, _path: &Path) -> std::io::Result<()> {
        LocalStorage::set(Self::PLAYLIST_KEY, self).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::Other, format!("Storage error: {:?}", e))
        })
    }
}

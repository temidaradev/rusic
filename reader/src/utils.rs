use std::fs;
use std::path::{Path, PathBuf};

pub fn find_folder_cover(dir: &Path) -> Option<PathBuf> {
    let candidates = ["cover.jpg", "cover.png", "folder.jpg", "album.jpg"];

    for name in candidates {
        let p = dir.join(name);
        if p.exists() {
            return Some(p);
        }
    }
    None
}

pub fn save_cover(album_id: &str, data: &[u8], cache_dir: &Path) -> std::io::Result<PathBuf> {
    fs::create_dir_all(cache_dir)?;
    let path = cache_dir.join(format!("{album_id}.jpg"));
    fs::write(&path, data)?;
    Ok(path)
}

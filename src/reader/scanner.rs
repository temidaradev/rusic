use super::metadata::read;
use super::models::Library;
use async_recursion::async_recursion;
use std::path::{Path, PathBuf};
use tokio::fs;

#[async_recursion]
pub async fn scan_directory(
    dir: PathBuf,
    cover_cache: PathBuf,
    library: &mut Library,
) -> std::io::Result<()> {
    let mut entries = match fs::read_dir(&dir).await {
        Ok(e) => e,
        Err(_) => return Ok(()),
    };

    while let Ok(Some(entry)) = entries.next_entry().await {
        let path = entry.path();
        if path.is_dir() {
            let _ = scan_directory(path, cover_cache.clone(), library).await;
        } else if is_audio_file(&path) {
            read(&path, &cover_cache, library);
        }
    }
    Ok(())
}

pub fn is_audio_file(path: &Path) -> bool {
    let extensions = ["mp3", "flac", "m4a", "wav", "ogg", "opus", "mp4"];
    path.extension()
        .and_then(|s| s.to_str())
        .map(|s| extensions.contains(&s.to_lowercase().as_str()))
        .unwrap_or(false)
}

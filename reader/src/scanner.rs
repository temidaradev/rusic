use super::metadata::read;
use super::models::Library;
use async_recursion::async_recursion;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use tokio::fs;

pub async fn scan_directory(
    dir: PathBuf,
    cover_cache: PathBuf,
    library: &mut Library,
) -> std::io::Result<()> {
    let existing_paths: HashSet<PathBuf> = library.tracks.iter().map(|t| t.path.clone()).collect();
    scan_directory_internal(dir, cover_cache, library, &existing_paths).await
}

#[async_recursion]
async fn scan_directory_internal(
    dir: PathBuf,
    cover_cache: PathBuf,
    library: &mut Library,
    existing_paths: &HashSet<PathBuf>,
) -> std::io::Result<()> {
    let mut entries = match fs::read_dir(&dir).await {
        Ok(e) => e,
        Err(_) => return Ok(()),
    };

    let mut audio_files = Vec::new();
    let mut sub_dirs = Vec::new();

    while let Ok(Some(entry)) = entries.next_entry().await {
        let path = entry.path();
        if path.is_dir() {
            sub_dirs.push(path);
        } else if is_audio_file(&path) {
            if !existing_paths.contains(&path) {
                audio_files.push(path);
            }
        }
    }

    if !audio_files.is_empty() {
        let mut lib = std::mem::take(library);
        let cover_cache_clone = cover_cache.clone();

        lib = tokio::task::spawn_blocking(move || {
            for path in audio_files {
                read(&path, &cover_cache_clone, &mut lib);
            }
            lib
        })
        .await
        .unwrap();

        *library = lib;
    }

    for sub_dir in sub_dirs {
        let _ =
            scan_directory_internal(sub_dir, cover_cache.clone(), library, existing_paths).await;
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

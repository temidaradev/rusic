use crate::reader::models::{Album, Track};
use crate::reader::Library;
use dioxus::prelude::*;

#[derive(Clone, Copy)]
pub struct SearchData {
    pub genres: Memo<Vec<(String, Option<String>)>>,
    pub search_results: Memo<Option<(Vec<(Track, Option<String>)>, Vec<(Album, Option<String>)>)>>,
    pub search_query: Signal<String>,
}

pub fn use_search_data(library: Signal<Library>, search_query: Signal<String>) -> SearchData {
    let genres = use_memo(move || {
        let lib = library.read();
        let mut genre_covers: std::collections::HashMap<String, Vec<std::path::PathBuf>> =
            std::collections::HashMap::new();

        for album in &lib.albums {
            let genre = album.genre.trim();
            if !genre.is_empty() {
                if let Some(cover) = &album.cover_path {
                    genre_covers
                        .entry(genre.to_string())
                        .or_default()
                        .push(cover.clone());
                } else {
                    genre_covers.entry(genre.to_string()).or_default();
                }
            }
        }

        let mut result: Vec<(String, Option<String>)> = genre_covers
            .into_iter()
            .map(|(g, covers)| {
                let cover_url = if !covers.is_empty() {
                    let idx = (g.len() + covers.len()) % covers.len();
                    let c = &covers[idx];
                    crate::utils::format_artwork_url(Some(c))
                } else {
                    None
                };
                (g, cover_url)
            })
            .collect();

        result.sort_by(|a, b| a.0.cmp(&b.0));
        result
    });

    let search_results = use_memo(move || {
        let query = search_query.read().to_lowercase();
        if query.trim().is_empty() {
            return None;
        }

        let lib = library.read();

        let album_map: std::collections::HashMap<&String, &Album> =
            lib.albums.iter().map(|a| (&a.id, a)).collect();

        let tracks: Vec<_> = lib
            .tracks
            .iter()
            .filter(|t| {
                t.title.to_lowercase().contains(&query)
                    || t.artist.to_lowercase().contains(&query)
                    || t.album.to_lowercase().contains(&query)
                    || album_map
                        .get(&t.album_id)
                        .map(|a| a.genre.to_lowercase().contains(&query))
                        .unwrap_or(false)
            })
            .take(100)
            .map(|t| {
                let cover_url = album_map
                    .get(&t.album_id)
                    .and_then(|a| a.cover_path.as_ref())
                    .and_then(|c| crate::utils::format_artwork_url(Some(c)));
                (t.clone(), cover_url)
            })
            .collect();

        let albums: Vec<_> = lib
            .albums
            .iter()
            .filter(|a| {
                a.title.to_lowercase().contains(&query)
                    || a.artist.to_lowercase().contains(&query)
                    || a.genre.to_lowercase().contains(&query)
            })
            .take(50)
            .map(|a| {
                let cover_url = a
                    .cover_path
                    .as_ref()
                    .and_then(|c| crate::utils::format_artwork_url(Some(c)));
                (a.clone(), cover_url)
            })
            .collect();

        Some((tracks, albums))
    });

    SearchData {
        genres,
        search_results,
        search_query,
    }
}

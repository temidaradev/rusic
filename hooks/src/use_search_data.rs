use config::{AppConfig, MusicSource};
use dioxus::prelude::*;
use reader::Library;
use reader::models::{Album, Track};

// why these, its because code was looking complex and clippy said use type for them to make them look
// good.
type TrackRes = Vec<(Track, Option<String>)>;
type AlbumRes = Vec<(Album, Option<String>)>;

#[derive(Clone, Copy)]
pub struct SearchData {
    pub genres: Memo<Vec<(String, Option<String>)>>,
    pub search_results: Memo<Option<(TrackRes, AlbumRes)>>,
    pub search_query: Signal<String>,
}

pub fn use_search_data(
    library: Signal<Library>,
    search_query: Signal<String>,
    config: Signal<AppConfig>,
) -> SearchData {
    let genres = use_memo(move || {
        if config.read().active_source == MusicSource::Jellyfin {
            return Vec::new();
        }

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
                    utils::format_artwork_url(Some(c))
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
        let active_source = config.read().active_source.clone();

        let album_map: std::collections::HashMap<&String, &Album> =
            lib.albums.iter().map(|a| (&a.id, a)).collect();

        let mut tracks: Vec<(Track, Option<String>)> = Vec::new();
        let mut albums: Vec<(Album, Option<String>)> = Vec::new();

        match active_source {
            MusicSource::Local => {
                tracks = lib
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
                            .and_then(|c| utils::format_artwork_url(Some(c)));
                        (t.clone(), cover_url)
                    })
                    .collect();

                albums = lib
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
                            .and_then(|c| utils::format_artwork_url(Some(c)));
                        (a.clone(), cover_url)
                    })
                    .collect();
            }
            MusicSource::Jellyfin => {
                tracks = lib
                    .jellyfin_tracks
                    .iter()
                    .filter(|t| {
                        t.title.to_lowercase().contains(&query)
                            || t.artist.to_lowercase().contains(&query)
                            || t.album.to_lowercase().contains(&query)
                    })
                    .take(100)
                    .map(|t| {
                        let cover_url = if let Some(server) = &config.read().server {
                            let path_str = t.path.to_string_lossy();
                            let parts: Vec<&str> = path_str.split(':').collect();
                            if parts.len() >= 2 {
                                let id = parts[1];
                                let mut url = format!("{}/Items/{}/Images/Primary", server.url, id);
                                let mut params = Vec::new();

                                if parts.len() >= 3 {
                                    params.push(format!("tag={}", parts[2]));
                                }
                                if let Some(token) = &server.access_token {
                                    params.push(format!("api_key={}", token));
                                }
                                if !params.is_empty() {
                                    url.push('?');
                                    url.push_str(&params.join("&"));
                                }
                                Some(url)
                            } else {
                                None
                            }
                        } else {
                            None
                        };
                        (t.clone(), cover_url)
                    })
                    .collect();
            }
        }

        Some((tracks, albums))
    });

    SearchData {
        genres,
        search_results,
        search_query,
    }
}

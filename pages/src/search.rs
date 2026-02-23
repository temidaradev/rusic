use components::playlist_modal::PlaylistModal;
use components::search_bar::SearchBar;
use components::search_genre_detail::SearchGenreDetail;
use components::search_genres::SearchGenres;
use components::search_results::SearchResults;
use config::AppConfig;
use dioxus::prelude::*;
use hooks::use_search_data::use_search_data;
use player::player;
use reader::Library;

#[component]
pub fn Search(
    library: Signal<Library>,
    config: Signal<AppConfig>,
    playlist_store: Signal<reader::PlaylistStore>,
    search_query: Signal<String>,
    player: Signal<player::Player>,
    is_playing: Signal<bool>,
    current_playing: Signal<u64>,
    current_song_cover_url: Signal<String>,
    current_song_title: Signal<String>,
    current_song_artist: Signal<String>,
    current_song_duration: Signal<u64>,
    current_song_progress: Signal<u64>,
    queue: Signal<Vec<reader::models::Track>>,
    current_queue_index: Signal<usize>,
) -> Element {
    let data = use_search_data(library, search_query, config);
    let mut selected_genre = use_signal(|| None::<String>);

    let mut active_menu_track = use_signal(|| None::<std::path::PathBuf>);
    let mut show_playlist_modal = use_signal(|| false);
    let selected_track_for_playlist = use_signal(|| None::<std::path::PathBuf>);

    let genre_tracks = use_memo(move || {
        let genre = selected_genre.read();
        let active_source = config.read().active_source.clone();

        if let Some(g) = &*genre {
            let lib = library.read();
            let is_jellyfin = active_source == config::MusicSource::Jellyfin;

            let albums = if is_jellyfin {
                &lib.jellyfin_albums
            } else {
                &lib.albums
            };
            let tracks = if is_jellyfin {
                &lib.jellyfin_tracks
            } else {
                &lib.tracks
            };

            let valid_album_ids: std::collections::HashSet<&String> = albums
                .iter()
                .filter(|a| a.genre.to_lowercase().contains(&g.to_lowercase()))
                .map(|a| &a.id)
                .collect();

            let album_map: std::collections::HashMap<&String, &reader::models::Album> =
                albums.iter().map(|a| (&a.id, a)).collect();

            let mut matching_tracks = Vec::new();
            for track in tracks {
                if valid_album_ids.contains(&track.album_id) {
                    let cover = if is_jellyfin {
                        if let Some(server) = &config.read().server {
                            let path_str = track.path.to_string_lossy();
                            let parts: Vec<&str> = path_str.split(':').collect();
                            if parts.len() >= 2 {
                                let id = parts[1];
                                let mut url = format!("{}/Items/{}/Images/Primary", server.url, id);
                                if let Some(token) = &server.access_token {
                                    url.push_str(&format!("?api_key={}", token));
                                }
                                Some(url)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        album_map
                            .get(&track.album_id)
                            .and_then(|a| a.cover_path.as_ref())
                            .and_then(|c| utils::format_artwork_url(Some(c)))
                    };
                    matching_tracks.push((track.clone(), cover));
                }
            }
            matching_tracks
        } else {
            Vec::new()
        }
    });

    rsx! {
        div {
            class: "p-8",

            if *show_playlist_modal.read() {
                PlaylistModal {
                    playlist_store: playlist_store,
                    is_jellyfin: config.read().active_source == config::MusicSource::Jellyfin,
                    on_close: move |_| show_playlist_modal.set(false),
                    on_add_to_playlist: move |playlist_id: String| {
                        if let Some(path) = selected_track_for_playlist.read().clone() {
                            let is_jellyfin = config.read().active_source == config::MusicSource::Jellyfin;
                            if !is_jellyfin {
                                let mut store = playlist_store.write();
                                if let Some(playlist) = store.playlists.iter_mut().find(|p| p.id == playlist_id) {
                                    if !playlist.tracks.contains(&path) {
                                        playlist.tracks.push(path);
                                    }
                                }
                            } else {
                                let path_clone = path.clone();
                                let pid = playlist_id.clone();
                                spawn(async move {
                                    let conf = config.peek();
                                    if let Some(server) = &conf.server {
                                        if let (Some(token), Some(user_id)) = (&server.access_token, &server.user_id) {
                                            let remote = server::jellyfin::JellyfinRemote::new(
                                                &server.url,
                                                Some(token),
                                                &conf.device_id,
                                                Some(user_id),
                                            );
                                            let parts: Vec<&str> = path_clone.to_str().unwrap_or_default().split(':').collect();
                                            if parts.len() >= 2 {
                                                let item_id = parts[1];
                                                let _ = remote.add_to_playlist(&pid, item_id).await;
                                            }
                                        }
                                    }
                                });
                            }
                        }
                        show_playlist_modal.set(false);
                        active_menu_track.set(None);
                    },
                    on_create_playlist: move |name: String| {
                        if let Some(path) = selected_track_for_playlist.read().clone() {
                            let is_jellyfin = config.read().active_source == config::MusicSource::Jellyfin;
                            if !is_jellyfin {
                                let mut store = playlist_store.write();
                                store.playlists.push(reader::models::Playlist {
                                    id: uuid::Uuid::new_v4().to_string(),
                                    name,
                                    tracks: vec![path],
                                });
                            } else {
                                let path_clone = path.clone();
                                let playlist_name = name.clone();
                                spawn(async move {
                                    let conf = config.peek();
                                    if let Some(server) = &conf.server {
                                        if let (Some(token), Some(user_id)) = (&server.access_token, &server.user_id) {
                                            let remote = server::jellyfin::JellyfinRemote::new(
                                                &server.url,
                                                Some(token),
                                                &conf.device_id,
                                                Some(user_id),
                                            );
                                            let parts: Vec<&str> = path_clone.to_str().unwrap_or_default().split(':').collect();
                                            if parts.len() >= 2 {
                                                let item_id = parts[1];
                                                let _ = remote.create_playlist(&playlist_name, &[item_id]).await;
                                            }
                                        }
                                    }
                                });
                            }
                        }
                        show_playlist_modal.set(false);
                        active_menu_track.set(None);
                    }
                }
            }

            if let Some(genre) = selected_genre.read().as_ref() {
                SearchGenreDetail {
                    genre: genre.clone(),
                    genre_tracks: genre_tracks.read().clone(),
                    genres: (data.genres)().clone(),
                    on_back: move |_| selected_genre.set(None),
                    library,
                    playlist_store,
                    player,
                    is_playing,
                    current_song_cover_url,
                    current_song_title,
                    current_song_artist,
                    current_song_duration,
                    current_song_progress,
                    queue,
                    current_queue_index,
                    active_menu_track,
                    show_playlist_modal,
                    selected_track_for_playlist,
                }
            } else {
                SearchBar { search_query: data.search_query }

                if let Some((tracks, albums)) = (data.search_results)() {
                    SearchResults {
                        search_query: data.search_query.read().clone(),
                        tracks: tracks.clone(),
                        albums: albums.clone(),
                        library,
                        playlist_store,
                        player,
                        is_playing,
                        current_song_cover_url,
                        current_song_title,
                        current_song_artist,
                        current_song_duration,
                        current_song_progress,
                        queue,
                        current_queue_index,
                        active_menu_track,
                        show_playlist_modal,
                        selected_track_for_playlist,
                    }
                } else {
                    SearchGenres {
                        genres: (data.genres)().clone(),
                        on_select_genre: move |genre| selected_genre.set(Some(genre)),
                    }
                }
            }
        }
    }
}

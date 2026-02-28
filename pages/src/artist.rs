use components::playlist_modal::PlaylistModal;
use config::{AppConfig, MusicSource};
use dioxus::prelude::*;
use player::player;
use reader::{Library, PlaylistStore};
use server::jellyfin::JellyfinRemote;
use std::collections::HashMap;

#[component]
pub fn Artist(
    library: Signal<Library>,
    config: Signal<AppConfig>,
    artist_name: Signal<String>,
    playlist_store: Signal<PlaylistStore>,
    player: Signal<player::Player>,
    mut is_playing: Signal<bool>,
    mut current_playing: Signal<u64>,
    mut current_song_cover_url: Signal<String>,
    mut current_song_title: Signal<String>,
    mut current_song_artist: Signal<String>,
    mut current_song_duration: Signal<u64>,
    mut current_song_progress: Signal<u64>,
    mut queue: Signal<Vec<reader::models::Track>>,
    mut current_queue_index: Signal<usize>,
    on_close: EventHandler<()>,
) -> Element {
    let is_jellyfin = config.read().active_source == MusicSource::Jellyfin;

    let mut show_playlist_modal = use_signal(|| false);
    let mut active_menu_track = use_signal(|| None::<std::path::PathBuf>);
    let mut selected_track_for_playlist = use_signal(|| None::<std::path::PathBuf>);

    let artist_tracks = use_memo(move || {
        let lib = library.read();
        let artist = artist_name.read();

        if artist.is_empty() {
            return Vec::new();
        }

        if is_jellyfin {
            lib.jellyfin_tracks
                .iter()
                .filter(|t| t.artist.to_lowercase() == artist.to_lowercase())
                .cloned()
                .collect::<Vec<_>>()
        } else {
            lib.tracks
                .iter()
                .filter(|t| t.artist.to_lowercase() == artist.to_lowercase())
                .cloned()
                .collect::<Vec<_>>()
        }
    });

    let local_artists = use_memo(move || {
        let lib = library.read();
        let mut artist_map = HashMap::new();

        for album in &lib.albums {
            if !artist_map.contains_key(&album.artist) {
                artist_map.insert(album.artist.clone(), album.cover_path.clone());
            }
        }

        let mut artists: Vec<_> = artist_map.into_iter().collect();
        artists.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));
        artists
    });

    let jellyfin_artists = use_memo(move || {
        let lib = library.read();
        let mut artist_map = HashMap::new();

        for album in &lib.jellyfin_albums {
            if !artist_map.contains_key(&album.artist) {
                artist_map.insert(album.artist.clone(), album.cover_path.clone());
            }
        }

        let mut artists: Vec<_> = artist_map.into_iter().collect();
        artists.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));
        artists
    });

    let artist_cover = use_memo(move || {
        let lib = library.read();
        let conf = config.read();
        let artist = artist_name.read();

        if artist.is_empty() {
            return None;
        }

        if is_jellyfin {
            lib.jellyfin_albums
                .iter()
                .find(|a| a.artist.to_lowercase() == artist.to_lowercase())
                .and_then(|album| {
                    if let Some(server) = &conf.server {
                        if let Some(cover_path) = &album.cover_path {
                            let path_str = cover_path.to_string_lossy();
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
                                return Some(url);
                            }
                        }
                    }
                    None
                })
        } else {
            lib.albums
                .iter()
                .find(|a| a.artist.to_lowercase() == artist.to_lowercase())
                .and_then(|album| utils::format_artwork_url(album.cover_path.as_ref()))
        }
    });

    let name = artist_name.read().clone();

    rsx! {
        div {
            class: "p-4 md:p-8 pb-24",

            if name.is_empty() {
                div {
                    class: "w-full max-w-[1600px] mx-auto",
                    h1 { class: "text-3xl font-bold text-white mb-8", "Artists" }

                    if is_jellyfin {
                        div { class: "grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-8",
                            for (artist, cover_path) in jellyfin_artists() {
                                {
                                    let cover_url = if let Some(server) = &config.read().server {
                                        if let Some(path) = cover_path {
                                            let path_str = path.to_string_lossy();
                                            let parts: Vec<&str> = path_str.split(':').collect();
                                            if parts.len() >= 2 {
                                                let id = parts[1];
                                                let mut url = format!("{}/Items/{}/Images/Primary", server.url, id);
                                                let mut params = Vec::new();
                                                if parts.len() >= 3 { params.push(format!("tag={}", parts[2])); }
                                                if let Some(token) = &server.access_token { params.push(format!("api_key={}", token)); }
                                                if !params.is_empty() {
                                                    url.push('?');
                                                    url.push_str(&params.join("&"));
                                                }
                                                Some(url)
                                            } else { None }
                                        } else { None }
                                    } else { None };

                                    let art = artist.clone();
                                    rsx! {
                                        div {
                                            key: "{artist}",
                                            class: "group cursor-pointer flex flex-col items-center",
                                            onclick: move |_| artist_name.set(art.clone()),
                                            div { class: "aspect-square w-full rounded-full bg-stone-800 mb-4 overflow-hidden shadow-lg relative group-hover:shadow-indigo-500/10 transition-all",
                                                if let Some(url) = cover_url {
                                                    img { src: "{url}", class: "w-full h-full object-cover group-hover:scale-110 transition-transform duration-500" }
                                                } else {
                                                     div { class: "w-full h-full flex items-center justify-center text-white/20",
                                                        i { class: "fa-solid fa-microphone text-5xl" }
                                                     }
                                                }
                                            }
                                            h3 { class: "text-white font-medium truncate text-center w-full group-hover:text-indigo-400 transition-colors", "{artist}" }
                                            p { class: "text-xs text-slate-500 uppercase tracking-wider mt-1", "Artist" }
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        div { class: "grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-8",
                            for (artist, cover_path) in local_artists() {
                                {
                                    let cover_url = utils::format_artwork_url(cover_path.as_ref());
                                    let art = artist.clone();
                                    rsx! {
                                        div {
                                            key: "{artist}",
                                            class: "group cursor-pointer flex flex-col items-center",
                                            onclick: move |_| artist_name.set(art.clone()),
                                            div { class: "aspect-square w-full rounded-full bg-stone-800 mb-4 overflow-hidden shadow-lg relative group-hover:shadow-indigo-500/10 transition-all",
                                                if let Some(url) = cover_url {
                                                    img { src: "{url}", class: "w-full h-full object-cover group-hover:scale-110 transition-transform duration-500" }
                                                } else {
                                                     div { class: "w-full h-full flex items-center justify-center text-white/20",
                                                        i { class: "fa-solid fa-microphone text-5xl" }
                                                     }
                                                }
                                            }
                                            h3 { class: "text-white font-medium truncate text-center w-full group-hover:text-indigo-400 transition-colors", "{artist}" }
                                            p { class: "text-xs text-slate-500 uppercase tracking-wider mt-1", "Artist" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                div {
                    class: "w-full max-w-[1600px] mx-auto",

                    if *show_playlist_modal.read() {
                        PlaylistModal {
                            playlist_store: playlist_store,
                            is_jellyfin: is_jellyfin,
                            on_close: move |_| show_playlist_modal.set(false),
                            on_add_to_playlist: move |playlist_id: String| {
                                if let Some(path) = selected_track_for_playlist.read().clone() {
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
                                                    let remote = JellyfinRemote::new(
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
                            },
                            on_create_playlist: move |name: String| {
                                if let Some(path) = selected_track_for_playlist.read().clone() {
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
                                                    let remote = JellyfinRemote::new(
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
                            }
                        }
                    }

                    div { class: "flex items-center justify-between mb-8",
                        button {
                            class: "flex items-center gap-2 text-slate-400 hover:text-white transition-colors",
                            onclick: move |_| artist_name.set(String::new()),
                            i { class: "fa-solid fa-arrow-left" }
                            "Back to Artists"
                        }
                    }

                    components::showcase::Showcase {
                        name: name.clone(),
                        description: "Artist".to_string(),
                        cover_url: artist_cover(),
                        tracks: artist_tracks(),
                        library: library,
                        active_track: active_menu_track.read().clone(),
                        on_play: move |idx: usize| {
                            let tracks = artist_tracks();
                            queue.set(tracks.clone());
                            current_queue_index.set(idx);

                            if let Some(t) = tracks.get(idx) {
                                if is_jellyfin {
                                    let mut ctrl = use_context::<hooks::use_player_controller::PlayerController>();
                                    ctrl.play_track(idx);
                                } else {
                                    let file = match std::fs::File::open(&t.path) {
                                        Ok(f) => f,
                                        Err(_) => return,
                                    };
                                    let source = match rodio::Decoder::new(std::io::BufReader::new(file)) {
                                        Ok(s) => s,
                                        Err(_) => return,
                                    };

                                    let lib = library.peek();
                                    let album_info = lib.albums.iter().find(|a| a.id == t.album_id);
                                    let artwork = album_info.and_then(|a| {
                                        a.cover_path.as_ref().map(|p| p.to_string_lossy().into_owned())
                                    });

                                    let meta = player::NowPlayingMeta {
                                        title: t.title.clone(),
                                        artist: t.artist.clone(),
                                        album: t.album.clone(),
                                        duration: std::time::Duration::from_secs(t.duration),
                                        artwork,
                                    };
                                    player.write().play(source, meta);
                                    current_song_title.set(t.title.clone());
                                    current_song_artist.set(t.artist.clone());
                                    current_song_duration.set(t.duration);
                                    current_song_progress.set(0);
                                    is_playing.set(true);

                                    if let Some(album) = album_info {
                                        if let Some(url) = utils::format_artwork_url(album.cover_path.as_ref()) {
                                            current_song_cover_url.set(url);
                                        } else {
                                            current_song_cover_url.set(String::new());
                                        }
                                    } else {
                                        current_song_cover_url.set(String::new());
                                    }
                                }
                            }
                        },
                        on_click_menu: move |idx: usize| {
                            if let Some(track) = artist_tracks().get(idx) {
                                if active_menu_track.read().as_ref() == Some(&track.path) {
                                    active_menu_track.set(None);
                                } else {
                                    active_menu_track.set(Some(track.path.clone()));
                                }
                            }
                        },
                        on_close_menu: move |_| active_menu_track.set(None),
                        on_add_to_playlist: move |idx: usize| {
                            if let Some(track) = artist_tracks().get(idx) {
                                selected_track_for_playlist.set(Some(track.path.clone()));
                                show_playlist_modal.set(true);
                                active_menu_track.set(None);
                            }
                        },
                        on_delete_track: move |_| {
                            active_menu_track.set(None);
                        },
                        actions: None
                    }
                }
            }
        }
    }
}

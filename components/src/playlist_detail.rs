use dioxus::prelude::*;
use player::player;
use reader::{Library, PlaylistStore};
use std::collections::HashSet;
use std::path::PathBuf;

#[component]
pub fn PlaylistDetail(
    playlist_id: String,
    mut playlist_store: Signal<PlaylistStore>,
    mut library: Signal<Library>,
    config: Signal<config::AppConfig>,
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
    let store = playlist_store.read();
    let mut active_menu_track = use_signal(|| None::<PathBuf>);
    let mut show_playlist_modal = use_signal(|| false);
    let mut selected_track_for_playlist = use_signal(|| None::<PathBuf>);

    let mut is_selection_mode = use_signal(|| false);
    let mut selected_tracks = use_signal(|| HashSet::<PathBuf>::new());

    let (playlist_name, local_tracks_paths, is_jellyfin) =
        if let Some(p) = store.playlists.iter().find(|p| p.id == playlist_id) {
            (p.name.clone(), p.tracks.clone(), false)
        } else if let Some(p) = store
            .jellyfin_playlists
            .iter()
            .find(|p| p.id == playlist_id)
        {
            (p.name.clone(), vec![], true)
        } else {
            return rsx! { div { "Playlist not found" } };
        };

    let lib = library.read();
    let mut tracks = use_signal(Vec::<reader::models::Track>::new);
    let mut has_loaded_jellyfin_tracks = use_signal(|| false);

    if !is_jellyfin {
        let local_tracks: Vec<_> = local_tracks_paths
            .iter()
            .filter_map(|path| lib.tracks.iter().find(|t| t.path == *path).cloned())
            .collect();
        let local_tracks_for_effect = local_tracks.clone();
        use_effect(move || {
            tracks.set(local_tracks_for_effect.clone());
        });
    } else {
        let pid = playlist_id.clone();
        use_effect(move || {
            if !*has_loaded_jellyfin_tracks.read() {
                let pid_clone = pid.clone();
                spawn(async move {
                    let conf = config.peek();
                    if let Some(server) = &conf.server {
                        if let (Some(token), Some(user_id)) =
                            (&server.access_token, &server.user_id)
                        {
                            let remote = server::jellyfin::JellyfinClient::new(
                                &server.url,
                                Some(token),
                                &conf.device_id,
                                Some(user_id),
                            );

                            if let Ok(items) = remote.get_playlist_items(&pid_clone).await {
                                let mut new_tracks = Vec::new();
                                for (_, item) in items.into_iter().enumerate() {
                                    let duration_secs =
                                        item.run_time_ticks.unwrap_or(0) / 10_000_000;
                                    let mut path_str = format!("jellyfin:{}", item.id);
                                    if let Some(tags) = &item.image_tags {
                                        if let Some(tag) = tags.get("Primary") {
                                            path_str.push_str(&format!(":{}", tag));
                                        }
                                    }
                                    let bitrate_kbps = item.bitrate.unwrap_or(0) / 1000;
                                    let bitrate_u8 = if bitrate_kbps > 255 {
                                        255
                                    } else {
                                        bitrate_kbps as u8
                                    };

                                    let artist_str = item
                                        .album_artist
                                        .clone()
                                        .or_else(|| item.artists.as_ref().map(|a| a.join(", ")))
                                        .unwrap_or_default();
                                    new_tracks.push(reader::models::Track {
                                        path: PathBuf::from(path_str),
                                        album_id: item
                                            .album_id
                                            .map(|id| format!("jellyfin:{}", id))
                                            .unwrap_or_default(),
                                        title: item.name,
                                        artist: artist_str,
                                        album: item.album.unwrap_or_default(),
                                        duration: duration_secs,
                                        khz: item.sample_rate.unwrap_or(0),
                                        bitrate: bitrate_u8,
                                        track_number: item.index_number,
                                        disc_number: item.parent_index_number,
                                        musicbrainz_release_id: None,
                                        playlist_item_id: item.playlist_item_id,
                                    });
                                }
                                tracks.set(new_tracks);
                                has_loaded_jellyfin_tracks.set(true);
                            }
                        }
                    }
                });
            }
        });
    }

    let tracks_val = tracks.read().clone();
    let playlist_cover = if !is_jellyfin {
        tracks_val.first().and_then(|t| {
            lib.albums
                .iter()
                .find(|a| a.id == t.album_id)
                .and_then(|a| utils::format_artwork_url(a.cover_path.as_ref()))
        })
    } else {
        if let Some(_p) = store
            .jellyfin_playlists
            .iter()
            .find(|p| p.id == playlist_id)
        {
            tracks_val.first().and_then(|t| {
                if let Some(server) = &config.read().server {
                    let path_str = t.path.to_string_lossy();
                    utils::jellyfin_image::jellyfin_image_url_from_path(
                        &path_str,
                        &server.url,
                        server.access_token.as_deref(),
                        512,
                        90,
                    )
                } else {
                    None
                }
            })
        } else {
            None
        }
    };

    let pid_for_delete = playlist_id.clone();
    let pid_for_remove = playlist_id.clone();

    rsx! {
        div {
            class: "w-full max-w-[1600px] mx-auto select-none",

            div { class: "flex items-center justify-between mb-8",
                button {
                    class: "flex items-center gap-2 text-slate-400 hover:text-white transition-colors",
                    onclick: move |_| on_close.call(()),
                    i { class: "fa-solid fa-arrow-left" }
                    "Back to Playlists"
                }
            }

            crate::showcase::Showcase {
                name: playlist_name.clone(),
                description: if is_jellyfin { "Jellyfin Playlist".to_string() } else { String::new() },
                cover_url: playlist_cover,
                tracks: tracks_val.clone(),
                library: library,
                is_selection_mode: is_selection_mode(),
                selected_tracks: selected_tracks.read().clone(),
                on_long_press: move |idx: usize| {
                    if let Some(t) = tracks.read().get(idx) {
                        is_selection_mode.set(true);
                        selected_tracks.write().insert(t.path.clone());
                    }
                },
                on_select: move |(idx, selected): (usize, bool)| {
                    if let Some(t) = tracks.read().get(idx) {
                        if selected {
                            selected_tracks.write().insert(t.path.clone());
                        } else {
                            selected_tracks.write().remove(&t.path);
                            if selected_tracks.read().is_empty() {
                                is_selection_mode.set(false);
                            }
                        }
                    }
                },
                actions: rsx! {
                    if !is_jellyfin {
                        button {
                             class: "px-4 py-2 bg-red-500/10 text-red-500 rounded-lg hover:bg-red-500/20 transition-colors text-sm font-medium flex items-center gap-2",
                             onclick: move |_| {
                                 on_close.call(());
                                 playlist_store.write().playlists.retain(|p| p.id != pid_for_delete);
                             },
                             i { class: "fa-solid fa-trash" }
                             "Delete Playlist"
                        }
                    }
                },
                on_play: {
                    let mut ctrl = use_context::<hooks::use_player_controller::PlayerController>();
                    move |idx: usize| {
                        queue.set(tracks.peek().clone());
                        ctrl.play_track(idx);
                    }
                },
                on_add_to_playlist: {
                    move |idx: usize| {
                        if let Some(t) = tracks.read().get(idx) {
                            selected_track_for_playlist.set(Some(t.path.clone()));
                            show_playlist_modal.set(true);
                            active_menu_track.set(None);
                        }
                    }
                },
                active_track: active_menu_track.read().clone(),
                on_click_menu: {
                    move |idx: usize| {
                        if let Some(t) = tracks.read().get(idx) {
                            if active_menu_track.read().as_ref() == Some(&t.path) {
                                active_menu_track.set(None);
                            } else {
                                active_menu_track.set(Some(t.path.clone()));
                            }
                        }
                    }
                },
                on_close_menu: move |_| active_menu_track.set(None),
                on_delete_track: {
                    move |idx: usize| {
                        if let Some(t) = tracks.read().get(idx) {
                            if !is_jellyfin {
                                if std::fs::remove_file(&t.path).is_ok() {
                                    library.write().remove_track(&t.path);
                                    let cache_dir = std::path::Path::new("./cache").to_path_buf();
                                    let lib_path = cache_dir.join("library.json");
                                    let _ = library.read().save(&lib_path);
                                }
                            }
                            active_menu_track.set(None);
                        }
                    }
                },
                on_remove_from_playlist: {
                    let pid = pid_for_remove.clone();
                    move |idx: usize| {
                        if let Some(t) = tracks.read().get(idx) {
                            if !is_jellyfin {
                                let mut store = playlist_store.write();
                                if let Some(playlist) = store.playlists.iter_mut().find(|p| p.id == pid) {
                                    playlist.tracks.retain(|p| p != &t.path);
                                }
                            } else {
                                let pid_clone = pid.clone();
                                let entry_id_opt = t.playlist_item_id.clone();
                                spawn(async move {
                                    if let Some(entry_id) = entry_id_opt {
                                        let conf = config.peek();
                                        if let Some(server) = &conf.server {
                                            if let (Some(token), Some(user_id)) = (&server.access_token, &server.user_id) {
                                                let remote = server::jellyfin::JellyfinClient::new(
                                                    &server.url,
                                                    Some(token),
                                                    &conf.device_id,
                                                    Some(user_id),
                                                );
                                                let _ = remote.remove_from_playlist(&pid_clone, &entry_id).await;
                                            }
                                        }
                                    }
                                });
                            }
                            active_menu_track.set(None);
                        }
                    }
                }
            }

            if is_selection_mode() {
                crate::selection_bar::SelectionBar {
                    count: selected_tracks.read().len(),
                    on_add_to_playlist: move |_| {
                        show_playlist_modal.set(true);
                    },
                    on_delete: move |_| {
                        let paths: Vec<_> = selected_tracks.read().iter().cloned().collect();
                        if !is_jellyfin {
                            for path in paths {
                                if std::fs::remove_file(&path).is_ok() {
                                    library.write().remove_track(&path);
                                }
                            }
                        }
                        selected_tracks.write().clear();
                        is_selection_mode.set(false);
                    },
                    on_cancel: move |_| {
                        is_selection_mode.set(false);
                        selected_tracks.write().clear();
                    }
                }
            }

            if *show_playlist_modal.read() {
                crate::playlist_modal::PlaylistModal {
                    playlist_store: playlist_store,
                    is_jellyfin: is_jellyfin,
                    on_close: move |_| {
                        show_playlist_modal.set(false);
                        if is_selection_mode() {
                            is_selection_mode.set(false);
                            selected_tracks.write().clear();
                        }
                    },
                    on_add_to_playlist: move |playlist_id: String| {
                        let mut selected_paths = Vec::new();
                        if is_selection_mode() {
                            selected_paths = selected_tracks.read().iter().cloned().collect();
                        } else if let Some(path) = selected_track_for_playlist.read().clone() {
                            selected_paths.push(path);
                        }

                        if !selected_paths.is_empty() {
                            if !is_jellyfin {
                                let mut store = playlist_store.write();
                                if let Some(playlist) = store.playlists.iter_mut().find(|p| p.id == playlist_id) {
                                    for path in selected_paths {
                                        if !playlist.tracks.contains(&path) {
                                            playlist.tracks.push(path);
                                        }
                                    }
                                }
                            } else {
                                let pid = playlist_id.clone();
                                spawn(async move {
                                    let conf = config.peek();
                                    if let Some(server) = &conf.server {
                                        if let (Some(token), Some(user_id)) = (&server.access_token, &server.user_id) {
                                            let remote = server::jellyfin::JellyfinClient::new(
                                                &server.url,
                                                Some(token),
                                                &conf.device_id,
                                                Some(user_id),
                                            );
                                            for path in selected_paths {
                                                let parts: Vec<&str> = path.to_str().unwrap_or_default().split(':').collect();
                                                if parts.len() >= 2 {
                                                    let item_id = parts[1];
                                                    let _ = remote.add_to_playlist(&pid, item_id).await;
                                                }
                                            }
                                        }
                                    }
                                });
                            }
                        }
                        show_playlist_modal.set(false);
                        is_selection_mode.set(false);
                        selected_tracks.write().clear();
                    },
                    on_create_playlist: move |name: String| {
                        let mut selected_paths = Vec::new();
                        if is_selection_mode() {
                            selected_paths = selected_tracks.read().iter().cloned().collect();
                        } else if let Some(path) = selected_track_for_playlist.read().clone() {
                            selected_paths.push(path);
                        }

                        if !selected_paths.is_empty() {
                            if !is_jellyfin {
                                let mut store = playlist_store.write();
                                store.playlists.push(reader::models::Playlist {
                                    id: uuid::Uuid::new_v4().to_string(),
                                    name,
                                    tracks: selected_paths,
                                });
                            } else {
                                let playlist_name = name.clone();
                                spawn(async move {
                                    let conf = config.peek();
                                    if let Some(server) = &conf.server {
                                        if let (Some(token), Some(user_id)) = (&server.access_token, &server.user_id) {
                                            let remote = server::jellyfin::JellyfinClient::new(
                                                &server.url,
                                                Some(token),
                                                &conf.device_id,
                                                Some(user_id),
                                            );
                                            let item_ids: Vec<String> = selected_paths.iter().filter_map(|p| {
                                                let parts: Vec<&str> = p.to_str()?.split(':').collect();
                                                if parts.len() >= 2 { Some(parts[1].to_string()) } else { None }
                                            }).collect();
                                            if !item_ids.is_empty() {
                                                let item_id_refs: Vec<&str> = item_ids.iter().map(|s| s.as_str()).collect();
                                                let _ = remote.create_playlist(&playlist_name, &item_id_refs).await;
                                            }
                                        }
                                    }
                                });
                            }
                        }
                        show_playlist_modal.set(false);
                        is_selection_mode.set(false);
                        selected_tracks.write().clear();
                    }
                }
            }
        }
    }
}

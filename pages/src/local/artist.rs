use components::playlist_modal::PlaylistModal;
use components::selection_bar::SelectionBar;
use config::AppConfig;
use dioxus::prelude::*;
use reader::{Library, PlaylistStore};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

#[component]
pub fn LocalArtist(
    library: Signal<Library>,
    config: Signal<AppConfig>,
    artist_name: Signal<String>,
    playlist_store: Signal<PlaylistStore>,
    mut queue: Signal<Vec<reader::models::Track>>,
    mut current_queue_index: Signal<usize>,
) -> Element {
    let mut ctrl = use_context::<hooks::use_player_controller::PlayerController>();
    let mut show_playlist_modal = use_signal(|| false);
    let mut active_menu_track = use_signal(|| None::<PathBuf>);
    let mut selected_track_for_playlist = use_signal(|| None::<PathBuf>);

    // Multi-selection state
    let mut is_selection_mode = use_signal(|| false);
    let mut selected_tracks = use_signal(|| HashSet::<PathBuf>::new());

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

    let artist_tracks = use_memo(move || {
        let lib = library.read();
        let artist = artist_name.read();
        if artist.is_empty() {
            return Vec::new();
        }
        lib.tracks
            .iter()
            .filter(|t| t.artist.to_lowercase() == artist.to_lowercase())
            .cloned()
            .collect::<Vec<_>>()
    });

    let artist_cover = use_memo(move || {
        let lib = library.read();
        let artist = artist_name.read();
        if artist.is_empty() {
            return None;
        }
        lib.albums
            .iter()
            .find(|a| a.artist.to_lowercase() == artist.to_lowercase())
            .and_then(|album| utils::format_artwork_url(album.cover_path.as_ref()))
    });

    let name = artist_name.read().clone();

    rsx! {
        div {
            if name.is_empty() {
                div { class: "grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-8",
                    for (artist , cover_path) in local_artists() {
                        {
                            let cover_url = utils::format_artwork_url(cover_path.as_ref());
                            let art = artist.clone();
                            rsx! {
                                div {
                                    key: "{artist}",
                                    class: "group cursor-pointer flex flex-col items-center",
                                    onclick: move |_| artist_name.set(art.clone()),
                                    div { class: "aspect-square w-full rounded-full bg-stone-800 mb-4 overflow-hidden relative transition-all",
                                        if let Some(url) = cover_url {
                                            img { src: "{url}", class: "w-full h-full object-cover group-hover:scale-110 transition-transform duration-500" }
                                        } else {
                                            div { class: "w-full h-full flex items-center justify-center text-white/20",
                                                i { class: "fa-solid fa-microphone text-5xl" }
                                            }
                                        }
                                    }
                                    h3 { class: "text-white font-medium truncate text-center w-full group-hover:text-indigo-400 transition-colors", "{artist}" }
                                    p { class: "text-xs text-slate-500 uppercase tracking-wider mt-1", "{rust_i18n::t!(\"artist\")}" }
                                }
                            }
                        }
                    }
                }
            } else {
                div {
                    if *show_playlist_modal.read() {
                        PlaylistModal {
                            playlist_store,
                            is_jellyfin: false,
                            on_close: move |_| {
                                show_playlist_modal.set(false);
                                if is_selection_mode() {
                                    is_selection_mode.set(false);
                                    selected_tracks.write().clear();
                                }
                            },
                            on_add_to_playlist: move |playlist_id: String| {
                                let mut store = playlist_store.write();
                                if let Some(playlist) = store.playlists.iter_mut().find(|p| p.id == playlist_id) {
                                    if is_selection_mode() {
                                        for path in selected_tracks.read().iter() {
                                            if !playlist.tracks.contains(path) {
                                                playlist.tracks.push(path.clone());
                                            }
                                        }
                                    } else if let Some(path) = selected_track_for_playlist.read().clone() {
                                        if !playlist.tracks.contains(&path) {
                                            playlist.tracks.push(path);
                                        }
                                    }
                                }
                                show_playlist_modal.set(false);
                                active_menu_track.set(None);
                                is_selection_mode.set(false);
                                selected_tracks.write().clear();
                            },
                            on_create_playlist: move |name: String| {
                                let mut tracks = Vec::new();
                                if is_selection_mode() {
                                    tracks = selected_tracks.read().iter().cloned().collect();
                                } else if let Some(path) = selected_track_for_playlist.read().clone() {
                                    tracks.push(path);
                                }

                                let mut store = playlist_store.write();
                                store.playlists.push(reader::models::Playlist {
                                    id: uuid::Uuid::new_v4().to_string(),
                                    name,
                                    tracks,
                                });
                                show_playlist_modal.set(false);
                                active_menu_track.set(None);
                                is_selection_mode.set(false);
                                selected_tracks.write().clear();
                            },
                        }
                    }

                    if is_selection_mode() {
                        SelectionBar {
                            count: selected_tracks.read().len(),
                            on_add_to_playlist: move |_| {
                                show_playlist_modal.set(true);
                            },
                            on_delete: move |_| {
                                let paths: Vec<_> = selected_tracks.read().iter().cloned().collect();
                                for path in paths {
                                    if std::fs::remove_file(&path).is_ok() {
                                        library.write().remove_track(&path);
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

                    if artist_tracks().is_empty() {
                        div {
                            class: "flex flex-col items-center justify-center h-64 text-slate-500",
                            i { class: "fa-regular fa-music text-4xl mb-4 opacity-30" }
                            p { class: "text-base", "{rust_i18n::t!(\"no_tracks_found\")}" }
                        }
                    } else {
                        components::showcase::Showcase {
                            name: name.clone(),
                            description: rust_i18n::t!("artist").to_string(),
                            cover_url: artist_cover(),
                            tracks: artist_tracks(),
                            library,
                            active_track: active_menu_track.read().clone(),
                            is_selection_mode: is_selection_mode(),
                            selected_tracks: selected_tracks.read().clone(),
                            on_long_press: move |idx: usize| {
                                if let Some(track) = artist_tracks().get(idx) {
                                    is_selection_mode.set(true);
                                    selected_tracks.write().insert(track.path.clone());
                                }
                            },
                            on_select: move |(idx, selected): (usize, bool)| {
                                if let Some(track) = artist_tracks().get(idx) {
                                    if selected {
                                        selected_tracks.write().insert(track.path.clone());
                                    } else {
                                        selected_tracks.write().remove(&track.path);
                                        if selected_tracks.read().is_empty() {
                                            is_selection_mode.set(false);
                                        }
                                    }
                                }
                            },
                            on_play: move |idx: usize| {
                                let tracks = artist_tracks();
                                queue.set(tracks.clone());
                                current_queue_index.set(idx);
                                ctrl.play_track(idx);
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
                            on_delete_track: move |idx: usize| {
                                if let Some(track) = artist_tracks().get(idx) {
                                    if std::fs::remove_file(&track.path).is_ok() {
                                        library.write().remove_track(&track.path);
                                    }
                                }
                                active_menu_track.set(None)
                            },
                            actions: None,
                        }
                    }
                }
            }
        }
    }
}

use components::playlist_modal::PlaylistModal;
use components::selection_bar::SelectionBar;
use components::stat_card::StatCard;
use components::track_row::TrackRow;
use config::AppConfig;
use dioxus::prelude::*;
use hooks::use_library_items::use_library_items;
use hooks::use_player_controller::PlayerController;
use reader::Library;
use std::collections::HashSet;
use std::path::PathBuf;

#[component]
pub fn LocalLibrary(
    library: Signal<Library>,
    config: Signal<AppConfig>,
    playlist_store: Signal<reader::PlaylistStore>,
    on_rescan: EventHandler,
    mut queue: Signal<Vec<reader::models::Track>>,
) -> Element {
    let items = use_library_items(library);
    let mut sort_order = items.sort_order;

    use_effect(move || {
        let curr = sort_order.read().clone();
        if config.peek().sort_order != curr {
            config.write().sort_order = curr;
        }
    });

    let mut ctrl = use_context::<PlayerController>();
    let mut active_menu_track = use_signal(|| None::<PathBuf>);
    let mut show_playlist_modal = use_signal(|| false);
    let mut selected_track_for_playlist = use_signal(|| None::<PathBuf>);

    // Multi-selection state
    let mut is_selection_mode = use_signal(|| false);
    let mut selected_tracks = use_signal(|| HashSet::<PathBuf>::new());

    let displayed_tracks = use_memo(move || (items.all_tracks)());

    let queue_tracks = use_memo(move || {
        displayed_tracks()
            .iter()
            .map(|(t, _)| t.clone())
            .collect::<Vec<_>>()
    });

    let is_empty = displayed_tracks().is_empty();

    let queue_source = std::sync::Arc::new(queue_tracks());
    let tracks_nodes =
        displayed_tracks()
            .into_iter()
            .enumerate()
            .map(|(idx, (track, cover_url))| {
                let track_menu = track.clone();
                let track_add = track.clone();
                let track_delete = track.clone();
                let track_path = track.path.clone();
                let track_select = track.path.clone();
                let queue_arc = std::sync::Arc::clone(&queue_source);
                let track_key = format!("{}-{}", track.path.display(), idx);
                let is_menu_open = active_menu_track.read().as_ref() == Some(&track.path);
                let is_selected = selected_tracks.read().contains(&track_path);

                rsx! {
                    TrackRow {
                        key: "{track_key}",
                        track: track.clone(),
                        cover_url: cover_url.clone(),
                        is_menu_open,
                        is_selection_mode: is_selection_mode(),
                        is_selected,
                        on_long_press: move |_| {
                            is_selection_mode.set(true);
                            selected_tracks.write().insert(track_path.clone());
                        },
                        on_select: move |selected| {
                            if selected {
                                selected_tracks.write().insert(track_select.clone());
                            } else {
                                selected_tracks.write().remove(&track_select);
                                if selected_tracks.read().is_empty() {
                                    is_selection_mode.set(false);
                                }
                            }
                        },
                        on_click_menu: move |_| {
                            if active_menu_track.read().as_ref() == Some(&track_menu.path) {
                                active_menu_track.set(None);
                            } else {
                                active_menu_track.set(Some(track_menu.path.clone()));
                            }
                        },
                        on_add_to_playlist: move |_| {
                            selected_track_for_playlist.set(Some(track_add.path.clone()));
                            show_playlist_modal.set(true);
                            active_menu_track.set(None);
                        },
                        on_close_menu: move |_| active_menu_track.set(None),
                        on_delete: move |_| {
                            active_menu_track.set(None);
                            if std::fs::remove_file(&track_delete.path).is_ok() {
                                library.write().remove_track(&track_delete.path);
                            }
                        },
                        on_play: move |_| {
                            queue.set((*queue_arc).clone());
                            ctrl.play_track(idx);
                        },
                    }
                }
            });

    rsx! {
        div {
            class: "p-8 relative min-h-full",

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

            div {
                class: "flex items-center justify-between mb-6",
                h1 { class: "text-3xl font-bold text-white", "{rust_i18n::t!(\"your_library\")}" }
                button {
                    class: "text-white/60 hover:text-white transition-colors p-2 rounded-full hover:bg-white/10",
                    title: rust_i18n::t!("rescan_library").to_string(),
                    onclick: move |_| on_rescan.call(()),
                    i { class: "fa-solid fa-rotate" }
                }
            }

            div {
                class: "grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 mb-12",
                {
                    let lib = library.read();
                    let album_count = lib.albums.iter()
                        .map(|a| a.title.to_lowercase())
                        .collect::<std::collections::HashSet<_>>()
                        .len();
                    rsx! {
                        StatCard { label: rust_i18n::t!("tracks").to_string(),    value: "{lib.tracks.len()}",  icon: "fa-music" }
                        StatCard { label: rust_i18n::t!("albums").to_string(),    value: "{album_count}",  icon: "fa-compact-disc" }
                        StatCard { label: rust_i18n::t!("artists").to_string(),   value: "{(items.artist_count)()}", icon: "fa-user" }
                        StatCard { label: rust_i18n::t!("playlists").to_string(), value: "{playlist_store.read().playlists.len()}", icon: "fa-list" }
                    }
                }
            }

            div {
                class: "flex items-center justify-between mb-4",
                h2 { class: "text-xl font-semibold text-white/80", "{rust_i18n::t!(\"tracks\")}" }
                div {
                    class: "flex space-x-1 bg-white/5 border border-white/5 p-1 rounded-lg",
                    button {
                        class: if *sort_order.read() == config::SortOrder::Title {
                            "px-3 py-1 text-xs rounded-md bg-white/10 text-white font-medium transition-all"
                        } else {
                            "px-3 py-1 text-xs rounded-md text-white/40 hover:text-white/80 transition-all"
                        },
                        onclick: move |_| sort_order.set(config::SortOrder::Title),
                        "{rust_i18n::t!(\"title\")}"
                    }
                    button {
                        class: if *sort_order.read() == config::SortOrder::Artist {
                            "px-3 py-1 text-xs rounded-md bg-white/10 text-white font-medium transition-all"
                        } else {
                            "px-3 py-1 text-xs rounded-md text-white/40 hover:text-white/80 transition-all"
                        },
                        onclick: move |_| sort_order.set(config::SortOrder::Artist),
                        "{rust_i18n::t!(\"artist\")}"
                    }
                    button {
                        class: if *sort_order.read() == config::SortOrder::Album {
                            "px-3 py-1 text-xs rounded-md bg-white/10 text-white font-medium transition-all"
                        } else {
                            "px-3 py-1 text-xs rounded-md text-white/40 hover:text-white/80 transition-all"
                        },
                        onclick: move |_| sort_order.set(config::SortOrder::Album),
                        "{rust_i18n::t!(\"album\")}"
                    }
                }
            }

            div {
                class: "space-y-1 pb-20",
                if is_empty {
                    p { class: "text-slate-500 italic", "{rust_i18n::t!(\"no_tracks_found\")}" }
                } else {
                    {tracks_nodes}
                }
            }
        }
    }
}

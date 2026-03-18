use components::playlist_modal::PlaylistModal;
use components::stat_card::StatCard;
use components::track_row::TrackRow;
use config::AppConfig;
use dioxus::prelude::*;
use hooks::use_library_items::use_library_items;
use hooks::use_player_controller::PlayerController;
use reader::Library;
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

    let displayed_tracks = use_memo(move || (items.all_tracks)());

    let queue_tracks = use_memo(move || {
        displayed_tracks()
            .iter()
            .map(|(t, _)| t.clone())
            .collect::<Vec<_>>()
    });

    let is_empty = displayed_tracks().is_empty();

    let tracks_nodes =
        displayed_tracks()
            .into_iter()
            .enumerate()
            .map(|(idx, (track, cover_url))| {
                let track_menu = track.clone();
                let track_add = track.clone();
                let track_delete = track.clone();
                let queue_source = queue_tracks();
                let track_key = format!("{}-{}", track.path.display(), idx);
                let is_menu_open = active_menu_track.read().as_ref() == Some(&track.path);

                rsx! {
                    TrackRow {
                        key: "{track_key}",
                        track: track.clone(),
                        cover_url: cover_url.clone(),
                        is_menu_open,
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
                            queue.set(queue_source.clone());
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
                    on_close: move |_| show_playlist_modal.set(false),
                    on_add_to_playlist: move |playlist_id: String| {
                        if let Some(path) = selected_track_for_playlist.read().clone() {
                            let mut store = playlist_store.write();
                            if let Some(playlist) = store.playlists.iter_mut().find(|p| p.id == playlist_id) {
                                if !playlist.tracks.contains(&path) {
                                    playlist.tracks.push(path);
                                }
                            }
                        }
                        show_playlist_modal.set(false);
                        active_menu_track.set(None);
                    },
                    on_create_playlist: move |name: String| {
                        if let Some(path) = selected_track_for_playlist.read().clone() {
                            let mut store = playlist_store.write();
                            store.playlists.push(reader::models::Playlist {
                                id: uuid::Uuid::new_v4().to_string(),
                                name,
                                tracks: vec![path],
                            });
                        }
                        show_playlist_modal.set(false);
                        active_menu_track.set(None);
                    },
                }
            }

            div {
                class: "flex items-center justify-between mb-6",
                h1 { class: "text-3xl font-bold text-white", "Your Library" }
                button {
                    class: "text-white/60 hover:text-white transition-colors p-2 rounded-full hover:bg-white/10",
                    title: "Rescan Library",
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
                        StatCard { label: "Tracks",    value: "{lib.tracks.len()}",  icon: "fa-music" }
                        StatCard { label: "Albums",    value: "{album_count}",  icon: "fa-compact-disc" }
                        StatCard { label: "Artists",   value: "{(items.artist_count)()}", icon: "fa-user" }
                        StatCard { label: "Playlists", value: "{playlist_store.read().playlists.len()}", icon: "fa-list" }
                    }
                }
            }

            div {
                class: "flex items-center justify-between mb-4",
                h2 { class: "text-xl font-semibold text-white/80", "All Tracks" }
                div {
                    class: "flex space-x-1 bg-white/5 border border-white/5 p-1 rounded-lg",
                    button {
                        class: if *sort_order.read() == config::SortOrder::Title {
                            "px-3 py-1 text-xs rounded-md bg-white/10 text-white font-medium transition-all"
                        } else {
                            "px-3 py-1 text-xs rounded-md text-white/40 hover:text-white/80 transition-all"
                        },
                        onclick: move |_| sort_order.set(config::SortOrder::Title),
                        "Title"
                    }
                    button {
                        class: if *sort_order.read() == config::SortOrder::Artist {
                            "px-3 py-1 text-xs rounded-md bg-white/10 text-white font-medium transition-all"
                        } else {
                            "px-3 py-1 text-xs rounded-md text-white/40 hover:text-white/80 transition-all"
                        },
                        onclick: move |_| sort_order.set(config::SortOrder::Artist),
                        "Artist"
                    }
                    button {
                        class: if *sort_order.read() == config::SortOrder::Album {
                            "px-3 py-1 text-xs rounded-md bg-white/10 text-white font-medium transition-all"
                        } else {
                            "px-3 py-1 text-xs rounded-md text-white/40 hover:text-white/80 transition-all"
                        },
                        onclick: move |_| sort_order.set(config::SortOrder::Album),
                        "Album"
                    }
                }
            }

            div {
                class: "space-y-1 pb-20",
                if is_empty {
                    p { class: "text-slate-500 italic", "No tracks found." }
                } else {
                    {tracks_nodes}
                }
            }
        }
    }
}

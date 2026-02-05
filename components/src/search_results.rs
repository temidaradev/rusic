use crate::track_row::TrackRow;
use dioxus::prelude::*;
use hooks::use_player_controller::PlayerController;
use player::player;
use reader::Library;
use reader::models::{Album, Track};

#[component]
pub fn SearchResults(
    search_query: String,
    tracks: Vec<(Track, Option<String>)>,
    albums: Vec<(Album, Option<String>)>,
    library: Signal<Library>,
    playlist_store: Signal<reader::PlaylistStore>,
    player: Signal<player::Player>,
    mut is_playing: Signal<bool>,
    mut current_song_cover_url: Signal<String>,
    mut current_song_title: Signal<String>,
    mut current_song_artist: Signal<String>,
    mut current_song_duration: Signal<u64>,
    mut current_song_progress: Signal<u64>,
    mut queue: Signal<Vec<Track>>,
    mut current_queue_index: Signal<usize>,
    mut active_menu_track: Signal<Option<std::path::PathBuf>>,
    mut show_playlist_modal: Signal<bool>,
    mut selected_track_for_playlist: Signal<Option<std::path::PathBuf>>,
) -> Element {
    let mut ctrl = use_context::<PlayerController>();

    rsx! {
        div { class: "mt-8 space-y-8",
            if !tracks.is_empty() {
                div {
                    h2 { class: "text-xl font-semibold text-white/80 mb-4", "Tracks" }
                    div { class: "space-y-2",
                        for (idx, (track, cover_url)) in tracks.iter().enumerate() {
                            {
                                let track = track.clone();
                                let track_key = track.path.display().to_string();
                                let track_menu = track.clone();
                                let track_add = track.clone();
                                let track_delete = track.clone();
                                let is_menu_open = active_menu_track.read().as_ref() == Some(&track.path);
                                let search_queue: Vec<Track> = tracks.iter().map(|(t, _)| t.clone()).collect();

                                rsx! {
                                    TrackRow {
                                        key: "{track_key}",
                                        track: track.clone(),
                                        cover_url: cover_url.clone(),
                                        is_menu_open: is_menu_open,
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
                                                let cache_dir = std::path::Path::new("./cache").to_path_buf();
                                                let lib_path = cache_dir.join("library.json");
                                                let _ = library.read().save(&lib_path);
                                            }
                                        },
                                        on_play: move |_| {
                                            queue.set(search_queue.clone());
                                            ctrl.play_track(idx);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if !albums.is_empty() {
                div {
                    h2 { class: "text-xl font-semibold text-white/80 mb-4", "Albums" }
                    div { class: "grid grid-cols-2 md:grid-cols-4 lg:grid-cols-5 gap-4",
                        for (album, cover_url) in &albums {
                            div {
                                key: "{album.id}",
                                class: "p-4 bg-white/5 rounded-xl hover:bg-white/10 transition-colors cursor-pointer group",
                                div {
                                    class: "aspect-square rounded-lg bg-black/40 mb-3 overflow-hidden shadow-lg relative",
                                    if let Some(url) = cover_url {
                                        img {
                                            src: "{url}",
                                            class: "w-full h-full object-cover group-hover:scale-105 transition-transform duration-300",
                                            loading: "lazy",
                                            decoding: "async",
                                        }
                                    } else {
                                        div { class: "w-full h-full flex items-center justify-center",
                                            i { class: "fa-solid fa-compact-disc text-4xl text-white/20" }
                                        }
                                    }
                                }
                                h3 { class: "text-white font-medium truncate", "{album.title}" }
                                p { class: "text-sm text-slate-400 truncate", "{album.artist}" }
                            }
                        }
                    }
                }
            }

            if tracks.is_empty() && albums.is_empty() {
                div { class: "text-center py-12 text-slate-500",
                    p { "No results found for \"{search_query}\"" }
                }
            }
        }
    }
}

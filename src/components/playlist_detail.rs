use crate::hooks::PlayerController;
use crate::player::player;
use crate::reader::{Library, PlaylistStore};
use crate::web_audio_store;
use dioxus::prelude::*;

#[component]
pub fn PlaylistDetail(
    playlist_id: String,
    playlist_store: Signal<PlaylistStore>,
    library: Signal<Library>,
    player: Signal<player::Player>,
    mut is_playing: Signal<bool>,
    mut current_playing: Signal<u64>,
    mut current_song_cover_url: Signal<String>,
    mut current_song_title: Signal<String>,
    mut current_song_artist: Signal<String>,
    mut current_song_duration: Signal<u64>,
    mut current_song_progress: Signal<u64>,
    mut queue: Signal<Vec<crate::reader::models::Track>>,
    mut current_queue_index: Signal<usize>,
    on_close: EventHandler<()>,
) -> Element {
    let mut ctrl = use_context::<PlayerController>();
    let store = playlist_store.read();
    let mut active_menu_track = use_signal(|| None::<std::path::PathBuf>);
    let mut show_playlist_modal = use_signal(|| false);
    let mut selected_track_for_playlist = use_signal(|| None::<std::path::PathBuf>);

    let playlist = match store.playlists.iter().find(|p| p.id == playlist_id) {
        Some(p) => p,
        None => return rsx! { div { "Playlist not found" } },
    };

    let lib = library.read();
    let tracks: Vec<_> = playlist
        .tracks
        .iter()
        .filter_map(|path| lib.tracks.iter().find(|t| t.path == *path).cloned())
        .collect();

    let playlist_cover = tracks.first().and_then(|t| {
        lib.albums
            .iter()
            .find(|a| a.id == t.album_id)
            .and_then(|a| crate::utils::format_artwork_url(a.cover_path.as_ref()))
    });

    rsx! {
        div {
            class: "w-full max-w-[1600px] mx-auto",

            div { class: "flex items-center justify-between mb-8",
                button {
                    class: "flex items-center gap-2 text-slate-400 hover:text-white transition-colors",
                    onclick: move |_| on_close.call(()),
                    i { class: "fa-solid fa-arrow-left" }
                    "Back to Playlists"
                }
            }

            crate::components::showcase::Showcase {
                name: playlist.name.clone(),
                description: String::new(),
                cover_url: playlist_cover,
                tracks: tracks.clone(),
                library: library,
                actions: rsx! {
                    button {
                         class: "px-4 py-2 bg-red-500/10 text-red-500 rounded-lg hover:bg-red-500/20 transition-colors text-sm font-medium flex items-center gap-2",
                         onclick: move |_| {
                             on_close.call(());
                             playlist_store.write().playlists.retain(|p| p.id != playlist_id);
                         },
                         i { class: "fa-solid fa-trash" }
                         "Delete Playlist"
                    }
                },
                on_play: {
                    let q = tracks.clone();
                    move |idx: usize| {
                        ctrl.queue.set(q.clone());
                        ctrl.play_track(idx);
                    }
                },
                on_add_to_playlist: {
                    let q = tracks.clone();
                    move |idx: usize| {
                        if let Some(t) = q.get(idx) {
                            selected_track_for_playlist.set(Some(t.path.clone()));
                            show_playlist_modal.set(true);
                            active_menu_track.set(None);
                        }
                    }
                },
                active_track: active_menu_track.read().clone(),
                on_click_menu: {
                    let q = tracks.clone();
                    move |idx: usize| {
                        if let Some(t) = q.get(idx) {
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
                    let q = tracks.clone();
                    move |idx: usize| {
                        if let Some(t) = q.get(idx) {
                            // Remove from audio store and library
                            let path_key = t.path.to_string_lossy().to_string();
                            web_audio_store::remove_file(&path_key);
                            library.write().remove_track(&t.path);
                            let _ = library.read().save(&std::path::PathBuf::new());
                            active_menu_track.set(None);
                        }
                    }
                }
            }
            if *show_playlist_modal.read() {
                crate::components::playlist_modal::PlaylistModal {
                    playlist_store: playlist_store,
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
                    },
                    on_create_playlist: move |name: String| {
                        if let Some(path) = selected_track_for_playlist.read().clone() {
                            let mut store = playlist_store.write();
                            store.playlists.push(crate::reader::models::Playlist {
                                id: uuid::Uuid::new_v4().to_string(),
                                name,
                                tracks: vec![path],
                            });
                        }
                        show_playlist_modal.set(false);
                    }
                }
            }
        }
    }
}

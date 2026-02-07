use crate::hooks::PlayerController;
use crate::player::player;
use crate::reader::Library;
use crate::web_audio_store;
use dioxus::prelude::*;

#[component]
pub fn AlbumDetails(
    album_id: String,
    library: Signal<Library>,
    playlist_store: Signal<crate::reader::PlaylistStore>,
    player: Signal<player::Player>,
    mut is_playing: Signal<bool>,
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
    let mut active_menu_track = use_signal(|| None::<std::path::PathBuf>);
    let mut show_playlist_modal = use_signal(|| false);
    let mut selected_track_for_playlist = use_signal(|| None::<std::path::PathBuf>);

    let lib = library.read();
    let album = match lib.albums.iter().find(|a| a.id == album_id) {
        Some(a) => a,
        None => return rsx! { div { "Album not found" } },
    };

    let tracks: Vec<_> = lib
        .tracks
        .iter()
        .filter(|t| t.album_id == album_id)
        .cloned()
        .collect();

    let album_cover = crate::utils::format_artwork_url(album.cover_path.as_ref());

    rsx! {
        div {
            class: "w-full max-w-[1600px] mx-auto",

            div { class: "flex items-center justify-between mb-8",
                button {
                    class: "flex items-center gap-2 text-slate-400 hover:text-white transition-colors",
                    onclick: move |_| on_close.call(()),
                    i { class: "fa-solid fa-arrow-left" }
                    "Back to Albums"
                }
            }

            crate::components::showcase::Showcase {
                name: album.title.clone(),
                description: album.artist.clone(),
                cover_url: album_cover,
                tracks: tracks.clone(),
                library: library,
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
                on_delete_track: {
                    let q = tracks.clone();
                    move |idx: usize| {
                        if let Some(t) = q.get(idx) {
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

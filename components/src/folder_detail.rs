use dioxus::prelude::*;
use reader::{Library, PlaylistStore, models::Track};
use std::path::PathBuf;

#[component]
pub fn FolderDetail(
    folder_path: String,
    library: Signal<Library>,
    mut playlist_store: Signal<PlaylistStore>,
    config: Signal<config::AppConfig>,
    mut queue: Signal<Vec<Track>>,
    on_close: EventHandler<()>,
) -> Element {
    let folder_path_buf = PathBuf::from(&folder_path);
    let folder_name = folder_path_buf
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| folder_path.clone());

    let lib = library.read();
    let mut folder_tracks: Vec<Track> = lib
        .tracks
        .iter()
        .filter(|t| t.path.starts_with(&folder_path_buf))
        .cloned()
        .collect();
    folder_tracks.sort_by(|a, b| {
        a.disc_number
            .cmp(&b.disc_number)
            .then(a.track_number.cmp(&b.track_number))
            .then(a.title.cmp(&b.title))
    });

    let cover_url = folder_tracks.first().and_then(|t| {
        lib.albums
            .iter()
            .find(|a| a.id == t.album_id)
            .and_then(|a| utils::format_artwork_url(a.cover_path.as_ref()))
    });

    let mut ctrl = use_context::<hooks::use_player_controller::PlayerController>();
    let tracks_for_play = folder_tracks.clone();

    let mut active_menu_track = use_signal(|| None::<PathBuf>);
    let mut show_playlist_modal = use_signal(|| false);
    let mut selected_track_for_playlist = use_signal(|| None::<PathBuf>);

    let tracks_signal = {
        let t = folder_tracks.clone();
        use_signal(move || t)
    };

    rsx! {
        div { class: "w-full max-w-[1600px] mx-auto select-none",
            div { class: "flex items-center mb-8",
                button {
                    class: "flex items-center gap-2 text-slate-400 hover:text-white transition-colors",
                    onclick: move |_| on_close.call(()),
                    i { class: "fa-solid fa-arrow-left" }
                    "{i18n::t(\"back_to_playlists\")}"
                }
            }
            crate::showcase::Showcase {
                name: folder_name,
                description: i18n::t("folder_playlist"),
                cover_url,
                tracks: folder_tracks.clone(),
                library,
                active_track: active_menu_track.read().clone(),
                on_play: move |idx: usize| {
                    queue.set(tracks_for_play.clone());
                    ctrl.play_track(idx);
                },
                on_click_menu: move |idx: usize| {
                    if let Some(t) = tracks_signal.read().get(idx) {
                        if active_menu_track.read().as_ref() == Some(&t.path) {
                            active_menu_track.set(None);
                        } else {
                            active_menu_track.set(Some(t.path.clone()));
                        }
                    }
                },
                on_close_menu: move |_| active_menu_track.set(None),
                on_add_to_playlist: move |idx: usize| {
                    if let Some(t) = tracks_signal.read().get(idx) {
                        selected_track_for_playlist.set(Some(t.path.clone()));
                        show_playlist_modal.set(true);
                        active_menu_track.set(None);
                    }
                },
            }

            if *show_playlist_modal.read() {
                crate::playlist_modal::PlaylistModal {
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
                    },
                    on_create_playlist: move |name: String| {
                        if let Some(path) = selected_track_for_playlist.read().clone() {
                            let mut store = playlist_store.write();
                            store.playlists.push(reader::models::Playlist {
                                id: uuid::Uuid::new_v4().to_string(),
                                name,
                                tracks: vec![path],
                                cover_path: None,
                            });
                        }
                        show_playlist_modal.set(false);
                    },
                }
            }
        }
    }
}

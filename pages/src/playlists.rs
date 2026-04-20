use components::playlist_detail::PlaylistDetail;
use components::playlist_popups::AddPlaylistPopup;
use config::{AppConfig, MusicService, MusicSource};
use dioxus::prelude::*;
use player::player;
use reader::{Library, PlaylistStore};
use ::server::jellyfin::JellyfinClient;
use ::server::subsonic::SubsonicClient;

use crate::local::playlists::LocalPlaylists;
use crate::server::playlists::ServerPlaylists;

#[component]
pub fn PlaylistsPage(
    playlist_store: Signal<PlaylistStore>,
    library: Signal<Library>,
    config: Signal<AppConfig>,
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
    mut selected_playlist_id: Signal<Option<String>>,
) -> Element {
    let is_server = config.read().active_source == MusicSource::Server;

    let mut show_add_playlist = use_signal(|| false);
    let mut playlist_name = use_signal(|| String::new());
    let mut error = use_signal(|| Option::<String>::None);
    let mut saving = use_signal(|| false);
    let mut playlist_refresh_trigger = use_signal(|| 0u64);

    let handle_add_playlist = move |_| {
        if saving() { return; }
        let name = playlist_name();
        if is_server {
            let server_vals = {
                let conf = config.peek();
                conf.server.as_ref().and_then(|s| {
                    if let (Some(tok), Some(uid)) = (&s.access_token, &s.user_id) {
                        Some((s.service, s.url.clone(), tok.clone(), uid.clone(), conf.device_id.clone()))
                    } else { None }
                })
            };
            if let Some((service, url, token, user_id, device_id)) = server_vals {
                error.set(None);
                saving.set(true);
                spawn(async move {
                    let result = match service {
                        MusicService::Jellyfin => {
                            let remote = JellyfinClient::new(&url, Some(&token), &device_id, Some(&user_id));
                            remote.create_playlist(&name, &[]).await
                        }
                        MusicService::Subsonic | MusicService::Custom => {
                            let remote = SubsonicClient::new(&url, &user_id, &token);
                            remote.create_playlist(&name, &[]).await
                        }
                    };
                    saving.set(false);
                    match result {
                        Ok(_) => {
                            playlist_refresh_trigger.with_mut(|v| *v += 1);
                            show_add_playlist.set(false);
                            playlist_name.set(String::new());
                        }
                        Err(e) => {
                            error.set(Some(e));
                        }
                    }
                });
            } else {
                error.set(Some(rust_i18n::t!("error_server_not_configured").to_string()));
            }
        } else {
            let mut store = playlist_store.write();
            store.playlists.push(reader::models::Playlist {
                id: uuid::Uuid::new_v4().to_string(),
                name,
                tracks: Vec::new(),
            });
            show_add_playlist.set(false);
            playlist_name.set(String::new());
        }
    };

    let mut last_source = use_signal(|| config.read().active_source.clone());
    if *last_source.read() != config.read().active_source {
        selected_playlist_id.set(None);
        last_source.set(config.read().active_source.clone());
    }

    rsx! {
        div {
            class: "p-8",

            if let Some(pid) = selected_playlist_id.read().clone() {
                PlaylistDetail {
                    playlist_id: pid,
                    playlist_store,
                    library,
                    config,
                    player,
                    is_playing,
                    current_playing,
                    current_song_cover_url,
                    current_song_title,
                    current_song_artist,
                    current_song_duration,
                    current_song_progress,
                    queue,
                    current_queue_index,
                    on_close: move |_| selected_playlist_id.set(None),
                }
            } else {
                div { class: "flex items-center justify-between mb-8",
                    h1 { class: "text-3xl font-bold text-white", "{rust_i18n::t!(\"playlists\")}" }
                    button {
                        class: "text-white/60 flex items-center hover:text-white transition-colors p-3 rounded-full hover:bg-white/10",
                        title: rust_i18n::t!("add_playlist").to_string(),
                        aria_label: rust_i18n::t!("add_playlist").to_string(),
                        onclick: move |_| { error.set(None); show_add_playlist.set(true); },
                        i { class: "fa-solid fa-add" }
                    }
                }
                if show_add_playlist() {
                    AddPlaylistPopup {
                        playlist_name: playlist_name,
                        error: error,
                        on_close: move |_| { error.set(None); show_add_playlist.set(false); },
                        on_save: handle_add_playlist
                    }
                }

                if is_server {
                    ServerPlaylists {
                        playlist_store,
                        library,
                        config,
                        selected_playlist_id,
                        refresh_trigger: playlist_refresh_trigger,
                    }
                } else {
                    LocalPlaylists {
                        playlist_store,
                        library,
                        config,
                        selected_playlist_id,
                    }
                }
            }
        }
    }
}

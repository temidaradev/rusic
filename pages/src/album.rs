use config::{AppConfig, MusicSource};
use dioxus::prelude::*;
use player::player;
use reader::{Library, PlaylistStore};
use server::jellyfin::JellyfinRemote;

use crate::jellyfin::album::{JellyfinAlbum, JellyfinAlbumDetails};
use crate::local::album::LocalAlbum;

#[component]
pub fn Album(
    library: Signal<Library>,
    config: Signal<AppConfig>,
    album_id: Signal<String>,
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
) -> Element {
    let is_jellyfin = config.read().active_source == MusicSource::Jellyfin;

    let open_album_menu = use_signal(|| None::<String>);
    let mut show_album_playlist_modal = use_signal(|| false);
    let pending_album_id_for_playlist = use_signal(|| None::<String>);

    let mut has_fetched_jellyfin = use_signal(|| false);

    let mut fetch_jellyfin = move || {
        has_fetched_jellyfin.set(true);
        spawn(async move {
            let conf = config.read();
            if let Some(server) = &conf.server {
                if let (Some(token), Some(user_id)) = (&server.access_token, &server.user_id) {
                    let remote = JellyfinRemote::new(
                        &server.url,
                        Some(token),
                        &conf.device_id,
                        Some(user_id),
                    );
                    if let Ok(libs) = remote.get_music_libraries().await {
                        for lib in libs {
                            let mut album_start_index = 0;
                            let album_limit = 100;
                            loop {
                                if let Ok((albums, _total)) = remote
                                    .get_albums_paginated(&lib.id, album_start_index, album_limit)
                                    .await
                                {
                                    if albums.is_empty() {
                                        break;
                                    }
                                    let count = albums.len();
                                    let mut new_albums = Vec::new();
                                    for album_item in albums {
                                        let image_tag = album_item
                                            .image_tags
                                            .as_ref()
                                            .and_then(|t| t.get("Primary").cloned());
                                        let cover_url = if image_tag.is_some() {
                                            Some(std::path::PathBuf::from(format!(
                                                "jellyfin:{}:{}",
                                                album_item.id,
                                                image_tag.as_ref().unwrap()
                                            )))
                                        } else {
                                            Some(std::path::PathBuf::from(format!(
                                                "jellyfin:{}",
                                                album_item.id
                                            )))
                                        };
                                        new_albums.push(reader::models::Album {
                                            id: format!("jellyfin:{}", album_item.id),
                                            title: album_item.name,
                                            artist: album_item
                                                .album_artist
                                                .or_else(|| {
                                                    album_item
                                                        .artists
                                                        .as_ref()
                                                        .map(|a| a.join(", "))
                                                })
                                                .unwrap_or_default(),
                                            genre: album_item
                                                .genres
                                                .as_ref()
                                                .map(|g| g.join(", "))
                                                .unwrap_or_default(),
                                            year: album_item.production_year.unwrap_or(0),
                                            cover_path: cover_url,
                                        });
                                    }
                                    {
                                        let mut lib_write = library.write();
                                        for album in new_albums {
                                            if !lib_write
                                                .jellyfin_albums
                                                .iter()
                                                .any(|a| a.id == album.id)
                                            {
                                                lib_write.jellyfin_albums.push(album);
                                            }
                                        }
                                    }
                                    album_start_index += count;
                                    if count < album_limit {
                                        break;
                                    }
                                } else {
                                    break;
                                }
                            }

                            let mut start_index = 0;
                            let limit = 200;
                            loop {
                                if let Ok(items) = remote
                                    .get_music_library_items_paginated(&lib.id, start_index, limit)
                                    .await
                                {
                                    if items.is_empty() {
                                        break;
                                    }
                                    let count = items.len();
                                    let mut new_tracks = Vec::new();
                                    for item in items {
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
                                        new_tracks.push(reader::models::Track {
                                            path: std::path::PathBuf::from(path_str),
                                            album_id: item
                                                .album_id
                                                .map(|id| format!("jellyfin:{}", id))
                                                .unwrap_or_default(),
                                            title: item.name,
                                            artist: item
                                                .album_artist
                                                .or_else(|| item.artists.map(|a| a.join(", ")))
                                                .unwrap_or_default(),
                                            album: item.album.unwrap_or_default(),
                                            duration: duration_secs,
                                            khz: item.sample_rate.unwrap_or(0),
                                            bitrate: bitrate_u8,
                                            track_number: item.index_number,
                                            disc_number: item.parent_index_number,
                                            musicbrainz_release_id: None,
                                        });
                                    }
                                    {
                                        let mut lib_write = library.write();
                                        for track in new_tracks {
                                            if !lib_write
                                                .jellyfin_tracks
                                                .iter()
                                                .any(|t| t.path == track.path)
                                            {
                                                lib_write.jellyfin_tracks.push(track);
                                            }
                                        }
                                    }
                                    start_index += count;
                                    if count < limit {
                                        break;
                                    }
                                } else {
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        });
    };

    use_effect(move || {
        if is_jellyfin && !*has_fetched_jellyfin.read() {
            if library.read().jellyfin_tracks.is_empty()
                || library.read().jellyfin_albums.is_empty()
            {
                fetch_jellyfin();
            } else {
                has_fetched_jellyfin.set(true);
            }
        }
    });

    rsx! {
        div {
            class: "p-8 pb-24",

            if album_id.read().is_empty() {
                div {
                    h1 { class: "text-3xl font-bold text-white mb-6", "All Albums" }

                    if is_jellyfin {
                        JellyfinAlbum {
                            library,
                            config,
                            album_id,
                            playlist_store,
                            queue,
                            open_album_menu,
                            show_album_playlist_modal,
                            pending_album_id_for_playlist,
                        }
                    } else {
                        LocalAlbum {
                            library,
                            album_id,
                            playlist_store,
                            queue,
                            open_album_menu,
                            show_album_playlist_modal,
                            pending_album_id_for_playlist,
                        }
                    }

                    if *show_album_playlist_modal.read() {
                        components::playlist_modal::PlaylistModal {
                            playlist_store,
                            is_jellyfin,
                            on_close: move |_| show_album_playlist_modal.set(false),
                            on_add_to_playlist: move |playlist_id: String| {
                                if let Some(aid) = pending_album_id_for_playlist.read().clone() {
                                    let lib = library.read();
                                    let tracks: Vec<_> = if is_jellyfin {
                                        lib.jellyfin_tracks.iter()
                                            .filter(|t| t.album_id == aid)
                                            .map(|t| t.path.clone())
                                            .collect()
                                    } else {
                                        let album_title = lib.albums.iter()
                                            .find(|a| a.id == aid)
                                            .map(|a| a.title.clone());
                                        if let Some(title) = album_title {
                                            lib.tracks.iter()
                                                .filter(|t| t.album == title)
                                                .map(|t| t.path.clone())
                                                .collect()
                                        } else {
                                            Vec::new()
                                        }
                                    };
                                    let mut store = playlist_store.write();
                                    if let Some(playlist) = store.playlists.iter_mut().find(|p| p.id == playlist_id) {
                                        for path in tracks {
                                            if !playlist.tracks.contains(&path) {
                                                playlist.tracks.push(path);
                                            }
                                        }
                                    }
                                }
                                show_album_playlist_modal.set(false);
                            },
                            on_create_playlist: move |name: String| {
                                if let Some(aid) = pending_album_id_for_playlist.read().clone() {
                                    let lib = library.read();
                                    let tracks: Vec<_> = if is_jellyfin {
                                        lib.jellyfin_tracks.iter()
                                            .filter(|t| t.album_id == aid)
                                            .map(|t| t.path.clone())
                                            .collect()
                                    } else {
                                        let album_title = lib.albums.iter()
                                            .find(|a| a.id == aid)
                                            .map(|a| a.title.clone());
                                        if let Some(title) = album_title {
                                            lib.tracks.iter()
                                                .filter(|t| t.album == title)
                                                .map(|t| t.path.clone())
                                                .collect()
                                        } else {
                                            Vec::new()
                                        }
                                    };
                                    let mut store = playlist_store.write();
                                    store.playlists.push(reader::models::Playlist {
                                        id: uuid::Uuid::new_v4().to_string(),
                                        name,
                                        tracks,
                                    });
                                }
                                show_album_playlist_modal.set(false);
                            },
                        }
                    }
                }
            } else {
                if is_jellyfin {
                    JellyfinAlbumDetails {
                        album_jellyfin_id: album_id.read().clone(),
                        library,
                        config,
                        playlist_store,
                        queue,
                        on_close: move |_| album_id.set(String::new()),
                    }
                } else {
                    components::album_details::AlbumDetails {
                        album_id: album_id.read().clone(),
                        library,
                        playlist_store,
                        player,
                        is_playing,
                        current_song_cover_url,
                        current_song_title,
                        current_song_artist,
                        current_song_duration,
                        current_song_progress,
                        queue,
                        current_queue_index,
                        on_close: move |_| album_id.set(String::new()),
                    }
                }
            }
        }
    }
}

use components::playlist_modal::PlaylistModal;
use components::stat_card::StatCard;
use components::track_row::TrackRow;
use config::AppConfig;
use dioxus::prelude::*;
use hooks::use_player_controller::PlayerController;
use reader::Library;
use server::jellyfin::JellyfinRemote;
use std::collections::HashSet;
use std::path::PathBuf;

#[component]
pub fn JellyfinLibrary(
    library: Signal<Library>,
    config: Signal<AppConfig>,
    playlist_store: Signal<reader::PlaylistStore>,
    mut queue: Signal<Vec<reader::models::Track>>,
) -> Element {
    let mut ctrl = use_context::<PlayerController>();
    let mut is_loading = use_signal(|| false);
    let mut has_fetched = use_signal(|| false);
    let mut fetch_generation = use_signal(|| 0usize);
    let mut sort_order = use_signal(|| config.peek().sort_order.clone());

    use_effect(move || {
        let curr = sort_order.read().clone();
        if config.peek().sort_order != curr {
            config.write().sort_order = curr;
        }
    });

    let mut active_menu_track = use_signal(|| None::<PathBuf>);
    let mut show_playlist_modal = use_signal(|| false);
    let mut selected_track_for_playlist = use_signal(|| None::<PathBuf>);

    let mut fetch_jellyfin = move || {
        has_fetched.set(true);
        is_loading.set(true);
        fetch_generation.with_mut(|g| *g += 1);
        let current_gen = *fetch_generation.peek();
        {
            let mut lib_write = library.write();
            lib_write.jellyfin_tracks.clear();
            lib_write.jellyfin_albums.clear();
        }
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
                                if *fetch_generation.read() != current_gen {
                                    return;
                                }
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
                                            Some(PathBuf::from(format!(
                                                "jellyfin:{}:{}",
                                                album_item.id,
                                                image_tag.as_ref().unwrap()
                                            )))
                                        } else {
                                            Some(PathBuf::from(format!(
                                                "jellyfin:{}",
                                                album_item.id
                                            )))
                                        };
                                        let album = reader::models::Album {
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
                                        };
                                        new_albums.push(album);
                                    }
                                    if *fetch_generation.read() == current_gen {
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
                                    } else {
                                        return;
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
                                if *fetch_generation.read() != current_gen {
                                    return;
                                }
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
                                        let track = reader::models::Track {
                                            path: PathBuf::from(path_str),
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
                                        };
                                        new_tracks.push(track);
                                    }
                                    if *fetch_generation.read() == current_gen {
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
                                    } else {
                                        return;
                                    }
                                    start_index += count;
                                    if count < limit {
                                        break;
                                    }
                                } else {
                                    break;
                                }
                            }

                            if let Ok(genres) = remote.get_genres().await {
                                let mut lib_write = library.write();
                                lib_write.jellyfin_genres =
                                    genres.into_iter().map(|g| (g.name, g.id)).collect();
                            }
                        }
                    }
                }
            }
            is_loading.set(false);
        });
    };

    use_effect(move || {
        if !*has_fetched.read() {
            if library.read().jellyfin_tracks.is_empty() {
                fetch_jellyfin();
            } else {
                has_fetched.set(true);
            }
        }
    });

    let displayed_tracks = use_memo(move || {
        let mut tracks = library.read().jellyfin_tracks.clone();
        match *sort_order.read() {
            config::SortOrder::Title => {
                tracks.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()))
            }
            config::SortOrder::Artist => {
                tracks.sort_by(|a, b| a.artist.to_lowercase().cmp(&b.artist.to_lowercase()))
            }
            config::SortOrder::Album => {
                tracks.sort_by(|a, b| a.album.to_lowercase().cmp(&b.album.to_lowercase()))
            }
        }
        let conf = config.read();
        tracks
            .iter()
            .map(|t| {
                let cover_url = if let Some(server) = &conf.server {
                    let path_str = t.path.to_string_lossy();
                    let parts: Vec<&str> = path_str.split(':').collect();
                    if parts.len() >= 2 {
                        let id = parts[1];
                        let mut url = format!("{}/Items/{}/Images/Primary", server.url, id);
                        let mut params = Vec::new();
                        if parts.len() >= 3 {
                            params.push(format!("tag={}", parts[2]));
                        }
                        if let Some(token) = &server.access_token {
                            params.push(format!("api_key={}", token));
                        }
                        if !params.is_empty() {
                            url.push('?');
                            url.push_str(&params.join("&"));
                        }
                        Some(url)
                    } else {
                        None
                    }
                } else {
                    None
                };
                (t.clone(), cover_url)
            })
            .collect::<Vec<_>>()
    });

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
                        on_delete: move |_| active_menu_track.set(None),
                        on_play: move |_| {
                            queue.set(queue_source.clone());
                            ctrl.play_track(idx);
                        },
                    }
                }
            });

    rsx! {
        div {
            class: "p-4 md:p-8 relative min-h-full pb-24",

            if *show_playlist_modal.read() {
                PlaylistModal {
                    playlist_store,
                    is_jellyfin: true,
                    on_close: move |_| show_playlist_modal.set(false),
                    on_add_to_playlist: move |playlist_id: String| {
                        if let Some(path) = selected_track_for_playlist.read().clone() {
                            let path_clone = path.clone();
                            let pid = playlist_id.clone();
                            spawn(async move {
                                let conf = config.peek();
                                if let Some(server) = &conf.server {
                                    if let (Some(token), Some(user_id)) =
                                        (&server.access_token, &server.user_id)
                                    {
                                        let remote = JellyfinRemote::new(
                                            &server.url,
                                            Some(token),
                                            &conf.device_id,
                                            Some(user_id),
                                        );
                                        let parts: Vec<&str> = path_clone
                                            .to_str()
                                            .unwrap_or_default()
                                            .split(':')
                                            .collect();
                                        if parts.len() >= 2 {
                                            let item_id = parts[1];
                                            let _ = remote.add_to_playlist(&pid, item_id).await;
                                        }
                                    }
                                }
                            });
                        }
                        show_playlist_modal.set(false);
                        active_menu_track.set(None);
                    },
                    on_create_playlist: move |name: String| {
                        if let Some(path) = selected_track_for_playlist.read().clone() {
                            let path_clone = path.clone();
                            let playlist_name = name.clone();
                            spawn(async move {
                                let conf = config.peek();
                                if let Some(server) = &conf.server {
                                    if let (Some(token), Some(user_id)) =
                                        (&server.access_token, &server.user_id)
                                    {
                                        let remote = JellyfinRemote::new(
                                            &server.url,
                                            Some(token),
                                            &conf.device_id,
                                            Some(user_id),
                                        );
                                        let parts: Vec<&str> = path_clone
                                            .to_str()
                                            .unwrap_or_default()
                                            .split(':')
                                            .collect();
                                        if parts.len() >= 2 {
                                            let item_id = parts[1];
                                            let _ = remote
                                                .create_playlist(&playlist_name, &[item_id])
                                                .await;
                                        }
                                    }
                                }
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
                    title: "Refresh Jellyfin Library",
                    onclick: move |_| fetch_jellyfin(),
                    i { class: "fa-solid fa-rotate" }
                }
            }

            div {
                class: "grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 mb-12",
                {
                    let lib = library.read();
                    let artist_count = {
                        let mut artists = HashSet::new();
                        for album in &lib.jellyfin_albums { artists.insert(&album.artist); }
                        for track in &lib.jellyfin_tracks { artists.insert(&track.artist); }
                        artists.len()
                    };
                    rsx! {
                        StatCard { label: "Tracks",    value: "{lib.jellyfin_tracks.len()}",  icon: "fa-music" }
                        StatCard { label: "Albums",    value: "{lib.jellyfin_albums.len()}",  icon: "fa-compact-disc" }
                        StatCard { label: "Artists",   value: "{artist_count}",               icon: "fa-user" }
                        StatCard { label: "Playlists", value: "{playlist_store.read().jellyfin_playlists.len()}", icon: "fa-list" }
                    }
                }
            }

            div {
                class: "flex items-center justify-between mb-4",
                h2 { class: "text-xl font-semibold text-white/80", "Jellyfin Tracks" }
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
                    if *is_loading.read() {
                        div { class: "flex items-center justify-center py-12",
                            i { class: "fa-solid fa-spinner fa-spin text-3xl text-white/20" }
                        }
                    } else {
                        p { class: "text-slate-500 italic", "No tracks found." }
                    }
                } else {
                    {tracks_nodes}
                    if *is_loading.read() {
                        div { class: "flex items-center justify-center py-4",
                            i { class: "fa-solid fa-spinner fa-spin text-xl text-white/20" }
                        }
                    }
                }
            }
        }
    }
}

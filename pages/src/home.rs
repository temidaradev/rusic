use config::{AppConfig, MusicSource};
use dioxus::prelude::*;
use reader::{Library, PlaylistStore};
use server::jellyfin::JellyfinRemote;

#[component]
pub fn Home(
    library: Signal<Library>,
    playlist_store: Signal<PlaylistStore>,
    on_select_album: EventHandler<String>,
    on_search_artist: EventHandler<String>,
) -> Element {
    let config = use_context::<Signal<AppConfig>>();
    let is_jellyfin = config.read().active_source == MusicSource::Jellyfin;

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

                                        let sample_rate = item.sample_rate.unwrap_or(0);

                                        let track = reader::models::Track {
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
                                            khz: sample_rate,
                                            bitrate: bitrate_u8,
                                            track_number: item.index_number,
                                            disc_number: item.parent_index_number,
                                        };
                                        new_tracks.push(track);
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
        let is_jelly = config.read().active_source == MusicSource::Jellyfin;
        if is_jelly && !*has_fetched_jellyfin.read() {
            if library.read().jellyfin_tracks.is_empty()
                && library.read().jellyfin_albums.is_empty()
            {
                fetch_jellyfin();
            } else {
                has_fetched_jellyfin.set(true);
            }
        }
    });

    let recent_albums = use_memo(move || {
        let lib = library.read();
        lib.albums
            .iter()
            .rev()
            .take(10)
            .cloned()
            .collect::<Vec<_>>()
    });

    let recent_playlists = use_memo(move || {
        let store = playlist_store.read();
        store
            .playlists
            .iter()
            .rev()
            .take(10)
            .cloned()
            .collect::<Vec<_>>()
    });

    let artists = use_memo(move || {
        let lib = library.read();
        let mut unique_artists = std::collections::HashSet::new();
        let mut artist_list = Vec::new();

        for album in &lib.albums {
            if unique_artists.insert(album.artist.clone()) {
                let cover = album.cover_path.clone();
                artist_list.push((album.artist.clone(), cover));
            }
            if artist_list.len() >= 10 {
                break;
            }
        }
        artist_list
    });

    let jellyfin_albums = use_memo(move || {
        let lib = library.read();
        let conf = config.read();

        lib.jellyfin_albums
            .iter()
            .take(10)
            .map(|album| {
                let cover_url = if let Some(server) = &conf.server {
                    if let Some(cover_path) = &album.cover_path {
                        let path_str = cover_path.to_string_lossy();
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
                    }
                } else {
                    None
                };
                (
                    album.id.clone(),
                    album.title.clone(),
                    album.artist.clone(),
                    cover_url,
                )
            })
            .collect::<Vec<_>>()
    });

    let jellyfin_artists = use_memo(move || {
        let lib = library.read();
        let mut unique_artists = std::collections::HashSet::new();
        let mut artist_list = Vec::new();

        for track in &lib.jellyfin_tracks {
            if unique_artists.insert(track.artist.clone()) {
                let cover_url = if let Some(server) = &config.read().server {
                    let path_str = track.path.to_string_lossy();
                    let parts: Vec<&str> = path_str.split(':').collect();
                    if parts.len() >= 2 {
                        let id = parts[1];
                        let mut url = format!("{}/Items/{}/Images/Primary", server.url, id);
                        if let Some(token) = &server.access_token {
                            url.push_str(&format!("?api_key={}", token));
                        }
                        Some(url)
                    } else {
                        None
                    }
                } else {
                    None
                };
                artist_list.push((track.artist.clone(), cover_url));
            }
            if artist_list.len() >= 10 {
                break;
            }
        }
        artist_list
    });

    let jellyfin_recent = use_memo(move || {
        let lib = library.read();
        lib.jellyfin_tracks
            .iter()
            .take(10)
            .map(|t| {
                let cover_url = if let Some(server) = &config.read().server {
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

    let scroll_container = move |id: &str, direction: i32| {
        let script = format!(
            "document.getElementById('{}').scrollBy({{ left: {}, behavior: 'smooth' }})",
            id,
            direction * 300
        );
        let _ = document::eval(&script);
    };

    rsx! {
        div {
            class: "p-8 space-y-12 pb-24",

            if is_jellyfin {
                section {
                    div { class: "flex items-center justify-between mb-4",
                        h2 { class: "text-2xl font-bold text-white", "Jellyfin Artists" }
                        div { class: "flex gap-2",
                            button {
                                class: "w-8 h-8 rounded-full bg-white/5 hover:bg-white/10 flex items-center justify-center text-white transition-colors",
                                onclick: move |_| scroll_container("jelly-artists-scroll", -1),
                                i { class: "fa-solid fa-chevron-left" }
                            }
                            button {
                                class: "w-8 h-8 rounded-full bg-white/5 hover:bg-white/10 flex items-center justify-center text-white transition-colors",
                                onclick: move |_| scroll_container("jelly-artists-scroll", 1),
                                i { class: "fa-solid fa-chevron-right" }
                            }
                        }
                    }
                    div {
                        id: "jelly-artists-scroll",
                        class: "flex overflow-x-auto gap-6 pb-4 scrollbar-hide scroll-smooth",
                        for (artist, cover_url) in jellyfin_artists() {
                            div {
                                class: "flex-none w-48 group cursor-pointer",
                                onclick: {
                                    let artist = artist.clone();
                                    move |_| on_search_artist.call(artist.clone())
                                },
                                div { class: "w-48 h-48 rounded-full bg-stone-800 mb-4 overflow-hidden shadow-lg relative",
                                    if let Some(url) = cover_url {
                                        img { src: "{url}", class: "w-full h-full object-cover group-hover:scale-105 transition-transform duration-300" }
                                    } else {
                                         div { class: "w-full h-full flex items-center justify-center",
                                            i { class: "fa-solid fa-microphone text-4xl text-white/20" }
                                         }
                                    }
                                }
                                h3 { class: "text-white font-medium truncate text-center", "{artist}" }
                                p { class: "text-sm text-stone-400 text-center", "Artist" }
                            }
                        }
                    }
                }

                section {
                    div { class: "flex items-center justify-between mb-4",
                         h2 { class: "text-2xl font-bold text-white", "Albums" }
                         div { class: "flex gap-2",
                            button {
                                class: "w-8 h-8 rounded-full bg-white/5 hover:bg-white/10 flex items-center justify-center text-white transition-colors",
                                onclick: move |_| scroll_container("jelly-albums-scroll", -1),
                                i { class: "fa-solid fa-chevron-left" }
                            }
                            button {
                                class: "w-8 h-8 rounded-full bg-white/5 hover:bg-white/10 flex items-center justify-center text-white transition-colors",
                                onclick: move |_| scroll_container("jelly-albums-scroll", 1),
                                i { class: "fa-solid fa-chevron-right" }
                            }
                        }
                    }
                    div {
                        id: "jelly-albums-scroll",
                        class: "flex overflow-x-auto gap-6 pb-4 scrollbar-hide scroll-smooth",
                        for (album_id, title, artist, cover_url) in jellyfin_albums() {
                            div {
                               class: "flex-none w-48 group cursor-pointer",
                               onclick: {
                                   let id = album_id.clone();
                                   move |_| on_select_album.call(id.clone())
                               },
                               div { class: "w-48 h-48 rounded-md bg-stone-800 mb-4 overflow-hidden shadow-lg relative",
                                    if let Some(url) = cover_url {
                                        img { src: "{url}", class: "w-full h-full object-cover group-hover:scale-105 transition-transform duration-300" }
                                    } else {
                                         div { class: "w-full h-full flex items-center justify-center",
                                            i { class: "fa-solid fa-compact-disc text-4xl text-white/20" }
                                         }
                                    }
                               }
                               h3 { class: "text-white font-medium truncate", "{title}" }
                               p { class: "text-sm text-stone-400 truncate", "{artist}" }
                            }
                        }
                    }
                }

                section {
                    div { class: "flex items-center justify-between mb-4",
                         h2 { class: "text-2xl font-bold text-white", "Recent Tracks" }
                         div { class: "flex gap-2",
                            button {
                                class: "w-8 h-8 rounded-full bg-white/5 hover:bg-white/10 flex items-center justify-center text-white transition-colors",
                                onclick: move |_| scroll_container("jelly-recent-scroll", -1),
                                i { class: "fa-solid fa-chevron-left" }
                            }
                            button {
                                class: "w-8 h-8 rounded-full bg-white/5 hover:bg-white/10 flex items-center justify-center text-white transition-colors",
                                onclick: move |_| scroll_container("jelly-recent-scroll", 1),
                                i { class: "fa-solid fa-chevron-right" }
                            }
                        }
                    }
                    div {
                        id: "jelly-recent-scroll",
                        class: "flex overflow-x-auto gap-6 pb-4 scrollbar-hide scroll-smooth",
                        for (track, cover_url) in jellyfin_recent() {
                            div {
                               class: "flex-none w-48 group cursor-pointer",
                               div { class: "w-48 h-48 rounded-md bg-stone-800 mb-4 overflow-hidden shadow-lg relative",
                                    if let Some(url) = cover_url {
                                        img { src: "{url}", class: "w-full h-full object-cover group-hover:scale-105 transition-transform duration-300" }
                                    } else {
                                         div { class: "w-full h-full flex items-center justify-center",
                                            i { class: "fa-solid fa-music text-4xl text-white/20" }
                                         }
                                    }
                               }
                               h3 { class: "text-white font-medium truncate", "{track.title}" }
                               p { class: "text-sm text-stone-400 truncate", "{track.artist}" }
                            }
                        }
                    }
                }
            } else {
                section {
                    div { class: "flex items-center justify-between mb-4",
                        h2 { class: "text-2xl font-bold text-white", "Artists" }
                        div { class: "flex gap-2",
                            button {
                                class: "w-8 h-8 rounded-full bg-white/5 hover:bg-white/10 flex items-center justify-center text-white transition-colors",
                                onclick: move |_| scroll_container("artists-scroll", -1),
                                i { class: "fa-solid fa-chevron-left" }
                            }
                            button {
                                class: "w-8 h-8 rounded-full bg-white/5 hover:bg-white/10 flex items-center justify-center text-white transition-colors",
                                onclick: move |_| scroll_container("artists-scroll", 1),
                                i { class: "fa-solid fa-chevron-right" }
                            }
                        }
                    }
                    div {
                        id: "artists-scroll",
                        class: "flex overflow-x-auto gap-6 pb-4 scrollbar-hide scroll-smooth",
                        for (artist, cover_path) in artists() {
                            div {
                                class: "flex-none w-48 group cursor-pointer",
                                onclick: {
                                    let artist = artist.clone();
                                    move |_| on_search_artist.call(artist.clone())
                                },
                                div { class: "w-48 h-48 rounded-full bg-stone-800 mb-4 overflow-hidden shadow-lg relative",
                                    if let Some(path) = cover_path {
                                        if let Some(url) = utils::format_artwork_url(Some(&path)) {
                                            img { src: "{url}", class: "w-full h-full object-cover group-hover:scale-105 transition-transform duration-300" }
                                        }
                                    } else {
                                         div { class: "w-full h-full flex items-center justify-center",
                                            i { class: "fa-solid fa-microphone text-4xl text-white/20" }
                                         }
                                    }
                                }
                                h3 { class: "text-white font-medium truncate text-center", "{artist}" }
                                p { class: "text-sm text-stone-400 text-center", "Artist" }
                            }
                        }
                    }
                }

                section {
                    div { class: "flex items-center justify-between mb-4",
                         h2 { class: "text-2xl font-bold text-white", "Albums" }
                         div { class: "flex gap-2",
                            button {
                                class: "w-8 h-8 rounded-full bg-white/5 hover:bg-white/10 flex items-center justify-center text-white transition-colors",
                                onclick: move |_| scroll_container("albums-scroll", -1),
                                i { class: "fa-solid fa-chevron-left" }
                            }
                            button {
                                class: "w-8 h-8 rounded-full bg-white/5 hover:bg-white/10 flex items-center justify-center text-white transition-colors",
                                onclick: move |_| scroll_container("albums-scroll", 1),
                                i { class: "fa-solid fa-chevron-right" }
                            }
                        }
                    }
                    div {
                        id: "albums-scroll",
                        class: "flex overflow-x-auto gap-6 pb-4 scrollbar-hide scroll-smooth",
                        for album in recent_albums() {
                            div {
                               class: "flex-none w-48 group cursor-pointer",
                               onclick: {
                                   let id = album.id.clone();
                                   move |_| on_select_album.call(id.clone())
                               },
                               div { class: "w-48 h-48 rounded-md bg-stone-800 mb-4 overflow-hidden shadow-lg relative",
                                    if let Some(url) = utils::format_artwork_url(album.cover_path.as_ref()) {
                                        img { src: "{url}", class: "w-full h-full object-cover group-hover:scale-105 transition-transform duration-300" }
                                    } else {
                                         div { class: "w-full h-full flex items-center justify-center",
                                            i { class: "fa-solid fa-compact-disc text-4xl text-white/20" }
                                         }
                                    }
                               }
                               h3 { class: "text-white font-medium truncate", "{album.title}" }
                               p { class: "text-sm text-stone-400 truncate", "{album.artist}" }
                            }
                        }
                    }
                }
            }

            if !recent_playlists().is_empty() {
                section {
                    div { class: "flex items-center justify-between mb-4",
                         h2 { class: "text-2xl font-bold text-white", "Playlists" }
                         div { class: "flex gap-2",
                            button {
                                class: "w-8 h-8 rounded-full bg-white/5 hover:bg-white/10 flex items-center justify-center text-white transition-colors",
                                onclick: move |_| scroll_container("playlists-scroll", -1),
                                i { class: "fa-solid fa-chevron-left" }
                            }
                            button {
                                class: "w-8 h-8 rounded-full bg-white/5 hover:bg-white/10 flex items-center justify-center text-white transition-colors",
                                onclick: move |_| scroll_container("playlists-scroll", 1),
                                i { class: "fa-solid fa-chevron-right" }
                            }
                        }
                    }
                    div {
                        id: "playlists-scroll",
                        class: "flex overflow-x-auto gap-6 pb-4 scrollbar-hide scroll-smooth",
                        for playlist in recent_playlists() {
                            div {
                               class: "flex-none w-48 group cursor-pointer",
                               div { class: "w-48 h-48 rounded-md bg-stone-800 mb-4 overflow-hidden shadow-lg relative grid grid-cols-2 gap-0.5 p-0.5",
                                    div { class: "col-span-2 row-span-2 bg-gradient-to-br from-indigo-500 to-purple-600 flex items-center justify-center",
                                        i { class: "fa-solid fa-list-ul text-4xl text-white/50" }
                                    }
                               }
                               h3 { class: "text-white font-medium truncate", "{playlist.name}" }
                               p { class: "text-sm text-stone-400 truncate", "{playlist.tracks.len()} tracks" }
                            }
                        }
                    }
                }
            }
        }
    }
}

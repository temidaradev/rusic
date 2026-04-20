use dioxus::prelude::*;
use rand::seq::SliceRandom;
use rand::thread_rng;
use reader::{FavoritesStore, Library, PlaylistStore};

#[component]
pub fn LocalHome(
    library: Signal<Library>,
    playlist_store: Signal<PlaylistStore>,
    favorites_store: Signal<FavoritesStore>,
    on_select_album: EventHandler<String>,
    on_play_album: EventHandler<String>,
    on_select_playlist: EventHandler<String>,
    on_search_artist: EventHandler<String>,
) -> Element {
    let recent_albums = use_memo(move || {
        let lib = library.read();
        let mut unique_albums = Vec::new();
        let mut seen_titles = std::collections::HashSet::new();
        for album in lib.albums.iter().rev() {
            let title_key = album.title.trim().to_lowercase();
            if seen_titles.insert(title_key) {
                unique_albums.push(album.clone());
            }
            if unique_albums.len() >= 10 {
                break;
            }
        }
        unique_albums
    });

    let recent_playlists = use_memo(move || {
        let store = playlist_store.read();
        store
            .playlists
            .iter()
            .rev()
            .take(10)
            .cloned()
            .map(|p| {
                (
                    p.id,
                    p.name,
                    p.tracks.len(),
                    p.tracks
                        .first()
                        .and_then(|p| p.to_str())
                        .map(|s| s.to_string()),
                )
            })
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

    let local_shuffled = use_memo(move || {
        let lib = library.read();
        let mut unique_albums = Vec::new();
        let mut seen_titles = std::collections::HashSet::new();
        for album in &lib.albums {
            let title_key = album.title.trim().to_lowercase();
            if seen_titles.insert(title_key) {
                unique_albums.push(album.clone());
            }
        }
        if unique_albums.is_empty() {
            return Vec::new();
        }
        let mut rng = thread_rng();
        unique_albums.shuffle(&mut rng);
        unique_albums
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
            section { class: "relative h-[350px] rounded-3xl overflow-hidden mb-12",
                {
                    let local_list = local_shuffled.read();
                    if let Some(album) = local_list.first() {
                        rsx! {
                            div { class: "absolute inset-0",
                                if let Some(url) = utils::format_artwork_url(album.cover_path.as_ref()) {
                                    img { src: "{url}", class: "w-full h-full object-cover" }
                                }
                                div { class: "absolute inset-0 bg-gradient-to-r from-black/90 via-black/40 to-transparent" }
                            }
                            div { class: "relative h-full flex flex-col justify-center p-8 md:p-12",
                                span { class: "text-indigo-400 font-bold tracking-widest uppercase text-[10px] mb-3 flex items-center gap-2",
                                    i { class: "fa-solid fa-clock-rotate-left text-[8px]" }
                                    "{rust_i18n::t!(\"jump_back_in\")}"
                                }
                                h1 { class: "text-3xl md:text-5xl font-black text-white mb-4 leading-tight max-w-xl break-words", "{album.title}" }
                                p { class: "text-base md:text-lg text-white/60 mb-8 font-medium line-clamp-1 max-w-md", "{rust_i18n::t!(\"by_artist_full\", artist = album.artist)}" }
                                div { class: "flex items-center gap-4",
                                    button {
                                        class: "flex items-center gap-3 bg-white text-black px-8 py-3 rounded-full font-bold hover:bg-white/90 hover:scale-105 active:scale-95 transition-all w-fit",
                                        onclick: {
                                            let id = album.id.clone();
                                            move |_| on_play_album.call(id.clone())
                                        },
                                        i { class: "fa-solid fa-play text-[10px]" }
                                        span { class: "text-sm", "{rust_i18n::t!(\"start_listening\")}" }
                                    }
                                    {
                                        let local_hero_album_id = album.id.clone();
                                        let local_hero_fav = {
                                            let lib = library.read();
                                            let store = favorites_store.read();
                                            let tracks: Vec<_> = lib.tracks.iter()
                                                .filter(|t| t.album_id == album.id)
                                                .collect();
                                            !tracks.is_empty() && tracks.iter().all(|t| store.is_local_favorite(&t.path))
                                        };
                                        let heart_class = if local_hero_fav {
                                            "w-11 h-11 rounded-full bg-white/10 border border-white/20 flex items-center justify-center text-red-400 hover:bg-white/20 transition-all"
                                        } else {
                                            "w-11 h-11 rounded-full bg-white/10 border border-white/20 flex items-center justify-center text-white hover:bg-white/20 transition-all"
                                        };
                                        let heart_icon = if local_hero_fav { "fa-solid fa-heart" } else { "fa-regular fa-heart" };
                                        rsx! {
                                            button {
                                                class: "{heart_class}",
                                                onclick: move |_| {
                                                    let lib = library.read();
                                                    let tracks: Vec<_> = lib.tracks.iter()
                                                        .filter(|t| t.album_id == local_hero_album_id)
                                                        .cloned()
                                                        .collect();
                                                    drop(lib);
                                                    let new_fav = !local_hero_fav;
                                                    for track in tracks {
                                                        let currently = favorites_store.read().is_local_favorite(&track.path);
                                                        if new_fav && !currently {
                                                            favorites_store.write().toggle_local(track.path);
                                                        } else if !new_fav && currently {
                                                            favorites_store.write().toggle_local(track.path);
                                                        }
                                                    }
                                                },
                                                i { class: "{heart_icon}" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        rsx! {
                            div { class: "absolute inset-0 bg-gradient-to-br from-indigo-900/40 to-purple-900/40 flex items-center justify-center",
                                div { class: "text-center",
                                    i { class: "fa-solid fa-music text-6xl text-white/10 mb-4" }
                                    h2 { class: "text-2xl font-bold text-white/40", "{rust_i18n::t!(\"add_music_to_get_started\")}" }
                                }
                            }
                        }
                    }
                }
            }

            {
                let local_list = local_shuffled.read();
                if !local_list.is_empty() {
                    rsx! {
                        section { class: "mb-12",
                            div { class: "flex items-end justify-between mb-6 text-white",
                                div {
                                    h2 { class: "text-3xl font-extrabold tracking-tight", "{rust_i18n::t!(\"listen_now\")}" }
                                }
                            }
                            div { class: "grid grid-cols-[repeat(auto-fill,minmax(350px,1fr))] gap-4",
                                for album in local_list.iter().skip(1).take(8) {
                                    div {
                                        class: "flex items-center bg-white/5 hover:bg-white/10 border border-white/5 rounded-2xl cursor-pointer transition-all duration-300 group overflow-hidden pr-4",
                                        onclick: {
                                            let id = album.id.clone();
                                            move |_| on_select_album.call(id.clone())
                                        },
                                        div { class: "w-16 h-16 md:w-20 md:h-20 flex-shrink-0 bg-stone-800/50 relative overflow-hidden",
                                            if let Some(url) = utils::format_artwork_url(album.cover_path.as_ref()) {
                                                img { src: "{url}", class: "w-full h-full object-cover group-hover:scale-105 transition-transform duration-500" }
                                            } else {
                                                div { class: "w-full h-full flex items-center justify-center",
                                                    i { class: "fa-solid fa-compact-disc text-xl text-white/20" }
                                                }
                                            }
                                            div { class: "absolute inset-0 bg-black/0 group-hover:bg-black/20 transition-colors duration-300" }
                                        }
                                        div { class: "p-4 flex-1 min-w-0 flex flex-col justify-center",
                                            h3 { class: "text-white font-bold truncate text-sm md:text-base", "{album.title}" }
                                            p { class: "text-xs text-white/50 truncate font-semibold mt-1", "{album.artist}" }
                                        }
                                        div { class: "opacity-0 group-hover:opacity-100 transition-all duration-300",
                                            div {
                                                class: "w-8 h-8 rounded-full bg-white/10 flex items-center justify-center hover:bg-white/20 transition-colors",
                                                onclick: {
                                                    let id = album.id.clone();
                                                    move |evt| {
                                                        evt.stop_propagation();
                                                        on_play_album.call(id.clone());
                                                    }
                                                },
                                                i { class: "fa-solid fa-play text-white/80 text-xs" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else {
                    rsx! { div {} }
                }
            }

            if !artists().is_empty() {
                section { class: "mt-12",
                    div { class: "flex items-center justify-between mb-6",
                        h2 { class: "text-2xl font-bold text-white tracking-tight", "{rust_i18n::t!(\"top_artists\")}" }
                        div { class: "flex gap-2",
                            button {
                                class: "w-8 h-8 rounded-full bg-white/5 hover:bg-white/10 flex items-center justify-center text-white transition-all",
                                onclick: move |_| scroll_container("artists-scroll", -1),
                                i { class: "fa-solid fa-chevron-left text-sm" }
                            }
                            button {
                                class: "w-8 h-8 rounded-full bg-white/5 hover:bg-white/10 flex items-center justify-center text-white transition-all",
                                onclick: move |_| scroll_container("artists-scroll", 1),
                                i { class: "fa-solid fa-chevron-right text-sm" }
                            }
                        }
                    }
                    div {
                        id: "artists-scroll",
                        class: "flex overflow-x-auto gap-6 pb-6 pt-2 overflow-y-visible scrollbar-hide scroll-smooth -mx-2 px-2",
                        for (artist, cover_path) in artists() {
                            div {
                                class: "flex-none w-32 md:w-36 group cursor-pointer",
                                onclick: {
                                    let artist = artist.clone();
                                    move |_| on_search_artist.call(artist.clone())
                                },
                                div { class: "aspect-square rounded-full bg-stone-800/80 mb-4 overflow-hidden transition-all duration-300 relative mx-auto",
                                    if let Some(path) = cover_path {
                                        if let Some(url) = utils::format_artwork_url(Some(&path)) {
                                            img { src: "{url}", class: "w-full h-full object-cover group-hover:scale-110 transition-transform duration-700" }
                                        }
                                    } else {
                                        div { class: "w-full h-full flex items-center justify-center border border-white/5 rounded-full",
                                            i { class: "fa-solid fa-microphone text-3xl text-white/20" }
                                        }
                                    }
                                    div { class: "absolute inset-0 bg-black/0 group-hover:bg-black/10 transition-colors duration-300 rounded-full" }
                                }
                                h3 { class: "text-white font-bold truncate text-center px-1 text-sm md:text-base", "{artist}" }
                            }
                        }
                    }
                }
            }

            if !recent_albums().is_empty() {
                section { class: "mt-12",
                    div { class: "flex items-center justify-between mb-6",
                        h2 { class: "text-2xl font-bold text-white tracking-tight", "{rust_i18n::t!(\"new_releases\")}" }
                        div { class: "flex gap-2",
                            button {
                                class: "w-8 h-8 rounded-full bg-white/5 hover:bg-white/10 flex items-center justify-center text-white transition-all",
                                onclick: move |_| scroll_container("albums-scroll", -1),
                                i { class: "fa-solid fa-chevron-left text-sm" }
                            }
                            button {
                                class: "w-8 h-8 rounded-full bg-white/5 hover:bg-white/10 flex items-center justify-center text-white transition-all",
                                onclick: move |_| scroll_container("albums-scroll", 1),
                                i { class: "fa-solid fa-chevron-right text-sm" }
                            }
                        }
                    }
                    div {
                        id: "albums-scroll",
                        class: "flex overflow-x-auto gap-5 pb-6 pt-2 scrollbar-hide scroll-smooth -mx-2 px-2",
                        for album in recent_albums() {
                            div {
                                class: "flex-none w-36 md:w-44 group cursor-pointer",
                                onclick: {
                                    let id = album.id.clone();
                                    move |_| on_select_album.call(id.clone())
                                },
                                div { class: "aspect-square rounded-2xl bg-stone-800/80 mb-4 overflow-hidden transition-all duration-300 relative",
                                    if let Some(url) = utils::format_artwork_url(album.cover_path.as_ref()) {
                                        img { src: "{url}", class: "w-full h-full object-cover group-hover:scale-105 transition-transform duration-500" }
                                    } else {
                                        div { class: "w-full h-full flex items-center justify-center border border-white/5 rounded-2xl",
                                            i { class: "fa-solid fa-compact-disc text-3xl text-white/20" }
                                        }
                                    }
                                    div { class: "absolute inset-0 bg-black/0 group-hover:bg-black/20 transition-colors duration-300" }
                                    div {
                                        class: "absolute right-3 bottom-3 w-10 h-10 bg-white text-black rounded-full flex items-center justify-center translate-y-4 opacity-0 group-hover:translate-y-0 group-hover:opacity-100 transition-all duration-300",
                                        onclick: {
                                            let id = album.id.clone();
                                            move |evt| {
                                                evt.stop_propagation();
                                                on_play_album.call(id.clone());
                                            }
                                        },
                                        i { class: "fa-solid fa-play text-xs ml-0.5" }
                                    }
                                }
                                h3 { class: "text-white font-bold truncate text-sm md:text-base px-1", "{album.title}" }
                                p { class: "text-xs md:text-sm text-white/50 truncate px-1 font-semibold mt-1", "{album.artist}" }
                            }
                        }
                    }
                }
            }

            if !recent_playlists().is_empty() {
                section { class: "mt-16",
                    div { class: "flex items-center justify-between mb-6",
                        div {
                            h2 { class: "text-2xl font-bold text-white tracking-tight", "{rust_i18n::t!(\"playlists\")}" }
                        }
                        div { class: "flex gap-2",
                            button {
                                class: "w-8 h-8 rounded-full bg-white/5 hover:bg-white/10 flex items-center justify-center text-white transition-all",
                                onclick: move |_| scroll_container("playlists-scroll", -1),
                                i { class: "fa-solid fa-chevron-left text-sm" }
                            }
                            button {
                                class: "w-8 h-8 rounded-full bg-white/5 hover:bg-white/10 flex items-center justify-center text-white transition-all",
                                onclick: move |_| scroll_container("playlists-scroll", 1),
                                i { class: "fa-solid fa-chevron-right text-sm" }
                            }
                        }
                    }
                    div {
                        id: "playlists-scroll",
                        class: "flex overflow-x-auto gap-6 pb-6 pt-2 scrollbar-hide scroll-smooth -mx-2 px-2",
                        for (id, name, track_count, first_track) in recent_playlists() {
                            {
                                let track_count_text = if track_count == 1 {
                                    rust_i18n::t!("track_count_singular").to_string()
                                } else {
                                    rust_i18n::t!("track_count", count = track_count).to_string()
                                };
                                let cover_url = if let Some(track_path) = first_track {
                                    let lib = library.peek();
                                    lib.tracks
                                        .iter()
                                        .find(|t| t.path.to_string_lossy() == track_path)
                                        .and_then(|t| {
                                            lib.albums
                                                .iter()
                                                .find(|a| a.id == t.album_id)
                                                .and_then(|a| a.cover_path.as_ref())
                                                .and_then(|cp| utils::format_artwork_url(Some(cp)))
                                        })
                                } else {
                                    None
                                };
                                rsx! {
                                    div {
                                        class: "flex-none w-40 md:w-48 group cursor-pointer",
                                        onclick: {
                                            let id = id.clone();
                                            move |_| on_select_playlist.call(id.clone())
                                        },
                                        div { class: "aspect-square rounded-2xl bg-white/5 mb-4 overflow-hidden transition-all duration-500 relative",
                                            if let Some(url) = cover_url {
                                                img { src: "{url}", class: "w-full h-full object-cover group-hover:scale-110 transition-transform duration-700" }
                                            } else {
                                                div { class: "w-full h-full flex items-center justify-center bg-gradient-to-br from-indigo-600/20 to-purple-600/20 group-hover:scale-110 transition-transform duration-700",
                                                    i { class: "fa-solid fa-music text-5xl opacity-40 text-white" }
                                                }
                                            }
                                            div { class: "absolute inset-0 bg-black/0 group-hover:bg-black/20 transition-colors duration-300" }
                                        }
                                        div {
                                            h3 { class: "text-white font-bold truncate text-sm md:text-base px-1 group-hover:text-indigo-400 transition-colors", "{name}" }
                                            p { class: "text-xs md:text-sm text-white/40 truncate px-1 font-semibold mt-1", "{track_count_text}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

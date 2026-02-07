use crate::reader::{Library, PlaylistStore};
use dioxus::prelude::*;

#[component]
pub fn Home(
    library: Signal<Library>,
    playlist_store: Signal<PlaylistStore>,
    on_select_album: EventHandler<String>,
    on_search_artist: EventHandler<String>,
) -> Element {
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
                                    if let Some(url) = crate::utils::format_artwork_url(Some(&path)) {
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
                                if let Some(url) = crate::utils::format_artwork_url(album.cover_path.as_ref()) {
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

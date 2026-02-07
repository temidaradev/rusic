use crate::player::player;
use crate::reader::{Library, PlaylistStore};
use dioxus::prelude::*;

#[component]
pub fn Album(
    library: Signal<Library>,
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
    mut queue: Signal<Vec<crate::reader::models::Track>>,
    mut current_queue_index: Signal<usize>,
) -> Element {
    let albums = library.read().albums.clone();

    rsx! {
        div {
            class: "p-8 pb-24",

            if album_id.read().is_empty() {
                div {
                    h1 { class: "text-3xl font-bold text-white mb-6", "All Albums" }
                    if albums.is_empty() {
                         p { class: "text-slate-500", "No albums found in library." }
                    } else {
                        div { class: "grid grid-cols-2 md:grid-cols-4 lg:grid-cols-5 gap-6",
                            for album in albums {
                                div {
                                    key: "{album.id}",
                                    class: "group cursor-pointer p-4 bg-white/5 rounded-xl hover:bg-white/10 transition-colors",
                                    onclick: {
                                        let id = album.id.clone();
                                        move |_| album_id.set(id.clone())
                                    },
                                    div { class: "aspect-square rounded-lg bg-stone-800 mb-3 overflow-hidden shadow-lg relative",
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
                }
            } else {
                crate::components::album_details::AlbumDetails {
                    album_id: album_id.read().clone(),
                    library: library,
                    playlist_store: playlist_store,
                    player: player,
                    is_playing: is_playing,
                    current_song_cover_url: current_song_cover_url,
                    current_song_title: current_song_title,
                    current_song_artist: current_song_artist,
                    current_song_duration: current_song_duration,
                    current_song_progress: current_song_progress,
                    queue: queue,
                    current_queue_index: current_queue_index,
                    on_close: move |_| album_id.set(String::new()),
                }
            }
        }
    }
}

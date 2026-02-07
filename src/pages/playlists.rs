use crate::components::playlist_detail::PlaylistDetail;
use crate::player::player;
use crate::reader::{Library, PlaylistStore};
use dioxus::prelude::*;

#[component]
pub fn PlaylistsPage(
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
) -> Element {
    let store = playlist_store.read();
    let mut selected_playlist_id = use_signal(|| None::<String>);

    rsx! {
        div {
            class: "p-8",
            if let Some(pid) = selected_playlist_id.read().clone() {
                 PlaylistDetail {
                     playlist_id: pid,
                     playlist_store: playlist_store,
                     library: library,
                     player: player,
                     is_playing: is_playing,
                     current_playing: current_playing,
                     current_song_cover_url: current_song_cover_url,
                     current_song_title: current_song_title,
                     current_song_artist: current_song_artist,
                     current_song_duration: current_song_duration,
                     current_song_progress: current_song_progress,
                     queue: queue,
                     current_queue_index: current_queue_index,
                     on_close: move |_| selected_playlist_id.set(None),
                 }
            } else {
                div { class: "flex items-center justify-between mb-8",
                    h1 { class: "text-3xl font-bold text-white", "Playlists" }
                }

                if store.playlists.is_empty() {
                    div { class: "flex flex-col items-center justify-center h-64 text-slate-500",
                        i { class: "fa-regular fa-folder-open text-4xl mb-4 opacity-50" }
                        p { "No playlists yet. Add songs from your library!" }
                    }
                } else {
                    div { class: "grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-6",
                        for playlist in &store.playlists {
                            div {
                                key: "{playlist.id}",
                                class: "bg-white/5 border border-white/5 rounded-2xl p-6 hover:bg-white/10 transition-all cursor-pointer group",
                                onclick: {
                                    let id = playlist.id.clone();
                                    move |_| selected_playlist_id.set(Some(id.clone()))
                                },
                                div { class: "mb-4 w-12 h-12 bg-indigo-500/20 rounded-full flex items-center justify-center text-indigo-400 group-hover:text-indigo-300 group-hover:bg-indigo-500/30 transition-colors",
                                    i { class: "fa-solid fa-list-ul" }
                                }
                                h3 { class: "text-xl font-bold text-white mb-1", "{playlist.name}" }
                                p { class: "text-sm text-slate-400", "{playlist.tracks.len()} tracks" }
                            }
                        }
                    }
                }
            }
        }
    }
}

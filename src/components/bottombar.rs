use crate::utils::format_artwork_url;
use crate::web_audio_store;
use crate::{player::player::Player, reader::Library};
use dioxus::prelude::*;

#[component]
pub fn Bottombar(
    library: Signal<Library>,
    player: Signal<Player>,
    mut is_playing: Signal<bool>,
    mut current_song_duration: Signal<u64>,
    mut current_song_progress: Signal<u64>,
    queue: Signal<Vec<crate::reader::models::Track>>,
    mut current_queue_index: Signal<usize>,
    mut current_song_title: Signal<String>,
    mut current_song_artist: Signal<String>,
    mut current_song_cover_url: Signal<String>,
    mut volume: Signal<f32>,
) -> Element {
    let format_time = |seconds: u64| {
        let minutes = seconds / 60;
        let seconds = seconds % 60;
        format!("{}:{:02}", minutes, seconds)
    };

    let progress_percent = if *current_song_duration.read() > 0 {
        (*current_song_progress.read() as f64 / *current_song_duration.read() as f64) * 100.0
    } else {
        0.0
    };

    let volume_percent = *volume.read() * 100.0;

    let mut play_song_at_index = move |index: usize| {
        let q = queue.read();
        if index < q.len() {
            let track = &q[index];
            let lib = library.peek();
            let album = lib.albums.iter().find(|a| a.id == track.album_id);
            let artwork = album.and_then(|a| {
                a.cover_path
                    .as_ref()
                    .map(|p| p.to_string_lossy().into_owned())
            });

            let meta = crate::player::player::NowPlayingMeta {
                title: track.title.clone(),
                artist: track.artist.clone(),
                album: track.album.clone(),
                duration: std::time::Duration::from_secs(track.duration),
                artwork,
            };

            let path_key = track.path.to_string_lossy().to_string();
            if let Some(url) = web_audio_store::get_blob_url(&path_key) {
                player.write().play_url(&url, meta);
                player.read().set_volume(*volume.peek());

                current_song_title.set(track.title.clone());
                current_song_artist.set(track.artist.clone());
                current_song_duration.set(track.duration);
                current_song_progress.set(0);

                if let Some(album) = album {
                    if let Some(url) = format_artwork_url(album.cover_path.as_ref()) {
                        current_song_cover_url.set(url);
                    } else {
                        current_song_cover_url.set(String::new());
                    }
                } else {
                    current_song_cover_url.set(String::new());
                }

                current_queue_index.set(index);
                is_playing.set(true);
            } else {
                web_sys::console::error_1(
                    &format!("File not found in audio store: {}", path_key).into(),
                );
            }
        }
    };

    rsx! {
        div {
            class: "h-24 bg-black/60 backdrop-blur-md border-t border-white/5 px-4 flex items-center justify-between select-none shrink-0",

            div {
                class: "flex items-center gap-4 w-1/4",
                div {
                    class: "w-14 h-14 bg-white/5 rounded-md flex-shrink-0 overflow-hidden shadow-lg",
                    if current_song_cover_url.read().is_empty() {
                        div {
                            class: "w-full h-full flex items-center justify-center",
                            style: "font-size: 1.5em;",
                            i { class: "fa-solid fa-music text-white/20" }
                        }
                    } else {
                        img {
                            src: "{current_song_cover_url}",
                            class: "w-full h-full object-cover"
                        }
                    }
                }
                div {
                    class: "flex flex-col min-w-0",
                    span { class: "text-sm font-bold text-white/90 truncate hover:underline cursor-pointer", "{current_song_title}" }
                    span { class: "text-xs text-slate-400 truncate hover:text-white/70 cursor-pointer", "{current_song_artist}" }
                }
                button {
                    class: "ml-2 text-slate-400 hover:text-red-400 transition-colors",
                    i { class: "fa-regular fa-heart" }
                }
            }

            div {
                class: "flex flex-col items-center max-w-[40%] w-full gap-2",
                div {
                    class: "flex items-center gap-6",
                    button { class: "text-slate-400 hover:text-white transition-all active:scale-95", i { class: "fa-solid fa-shuffle text-sm" } }
                    button {
                        class: "text-slate-400 hover:text-white transition-all active:scale-90",
                        onclick: move |_| {
                            let idx = *current_queue_index.read();
                            if idx > 0 {
                                play_song_at_index(idx - 1);
                            }
                        },
                        i { class: "fa-solid fa-backward-step text-xl" }
                    }
                    button {
                        class: "w-10 h-10 bg-white rounded-full flex items-center justify-center text-black hover:scale-105 active:scale-95 transition-all",
                        onclick: move |_| {
                            if *is_playing.read() {
                                player.write().pause();
                                is_playing.set(false);
                            } else {
                                player.write().play_resume();
                                is_playing.set(true);
                            }
                        },
                        i { class: if *is_playing.read() { "fa-solid fa-pause text-lg" } else { "fa-solid fa-play text-lg ml-0.5" } }
                    }
                    button {
                        class: "text-slate-400 hover:text-white transition-all active:scale-90",
                        onclick: move |_| {
                            let idx = *current_queue_index.read();
                            if idx + 1 < queue.read().len() {
                                play_song_at_index(idx + 1);
                            }
                        },
                        i { class: "fa-solid fa-forward-step text-xl" }
                    }
                    button { class: "text-slate-400 hover:text-white transition-all active:scale-95", i { class: "fa-solid fa-repeat text-sm" } }
                }

                div {
                    class: "flex items-center gap-2 w-full",
                    span { class: "text-[10px] text-slate-500 w-8 text-right font-mono", "{format_time(*current_song_progress.read())}" }
                    div {
                        class: "flex-1 h-1 bg-white/10 rounded-full group cursor-pointer relative",
                        div {
                            class: "absolute top-0 left-0 h-full bg-white group-hover:bg-green-500 rounded-full transition-colors pointer-events-none",
                            style: "width: {progress_percent}%",
                            div { class: "absolute -right-1.5 -top-1 w-3 h-3 bg-white rounded-full shadow-lg opacity-0 group-hover:opacity-100 transition-opacity" }
                        }
                        input {
                            r#type: "range",
                            min: "0",
                            max: "{*current_song_duration.read()}",
                            value: "{*current_song_progress.read()}",
                            class: "absolute top-0 left-0 w-full h-full opacity-0 cursor-pointer z-10",
                            oninput: move |evt| {
                                if let Ok(val) = evt.value().parse::<u64>() {
                                    player.write().seek(std::time::Duration::from_secs(val));
                                    current_song_progress.set(val);
                                }
                            }
                        }
                    }
                    span { class: "text-[10px] text-slate-500 w-8 font-mono", "{format_time(*current_song_duration.read())}" }
                }
            }

            div {
                class: "flex items-center justify-end gap-4 w-1/4",
                button { class: "text-slate-400 hover:text-white", i { class: "fa-solid fa-list-ul text-xs" } }
                button { class: "text-slate-400 hover:text-white", i { class: "fa-solid fa-desktop text-xs" } }
                div {
                    class: "flex items-center gap-2 group",
                    i { class: "fa-solid fa-volume-high text-xs text-slate-400 group-hover:text-white" }
                    div {
                        class: "w-24 h-1 bg-white/10 rounded-full group/vol cursor-pointer relative",
                        div {
                            class: "absolute top-0 left-0 h-full bg-white group-hover/vol:bg-green-500 rounded-full transition-colors pointer-events-none",
                            style: "width: {volume_percent}%",
                            div { class: "absolute -right-1.5 -top-1 w-3 h-3 bg-white rounded-full shadow-lg opacity-0 group-hover/vol:opacity-100 transition-opacity" }
                        }
                         input {
                            r#type: "range",
                            min: "0",
                            max: "1",
                            step: "0.01",
                            value: "{*volume.read()}",
                            class: "absolute top-0 left-0 w-full h-full opacity-0 cursor-pointer z-10",
                            oninput: move |evt| {
                                if let Ok(val) = evt.value().parse::<f32>() {
                                    player.read().set_volume(val);
                                    volume.set(val);
                                }
                            }
                        }
                    }
                }
                button { class: "text-slate-400 hover:text-white", i { class: "fa-solid fa-up-right-and-down-left-from-center text-xs" } }
            }
        }
    }
}

use dioxus::prelude::*;
use hooks::use_player_controller::PlayerController;
use player::player::Player;
use reader::Library;

#[component]
pub fn MobileBottombar(
    library: Signal<Library>,
    player: Signal<Player>,
    mut is_playing: Signal<bool>,
    mut is_fullscreen: Signal<bool>,
    mut current_song_duration: Signal<u64>,
    mut current_song_progress: Signal<u64>,
    queue: Signal<Vec<reader::models::Track>>,
    mut current_queue_index: Signal<usize>,
    mut current_song_title: Signal<String>,
    mut current_song_artist: Signal<String>,
    mut current_song_cover_url: Signal<String>,
    mut volume: Signal<f32>,
) -> Element {
    let progress_percent = if *current_song_duration.read() > 0 {
        (*current_song_progress.read() as f64 / *current_song_duration.read() as f64) * 100.0
    } else {
        0.0
    };

    let mut ctrl = use_context::<PlayerController>();

    rsx! {
        div {
            class: "md:hidden flex flex-col bg-[#121212]/95 backdrop-blur-xl border-t border-white/5 shrink-0 relative w-full pb-2 pt-2 px-3",

            div {
                class: "absolute top-0 left-0 h-[2px] w-full bg-white/10",
                div {
                    class: "h-full bg-green-500 transition-all duration-300",
                    style: "width: {progress_percent}%"
                }
            }

            div {
                class: "flex items-center justify-between w-full mt-1",

                div {
                    class: "flex items-center gap-3 min-w-0 flex-1",

                    div {
                        class: "w-12 h-12 bg-white/5 rounded-md flex-shrink-0 overflow-hidden shadow-lg border border-white/5",
                        if current_song_cover_url.read().is_empty() {
                            div {
                                class: "w-full h-full flex items-center justify-center",
                                style: "font-size: 1.2em;",
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
                        class: "flex flex-col min-w-0 flex-1 justify-center",
                        onclick: move |_| is_fullscreen.set(true),
                        span { class: "text-[15px] font-semibold text-white/95 truncate", "{current_song_title}" }
                        span { class: "text-[13px] text-slate-400 truncate mt-0.5", "{current_song_artist}" }
                    }
                }

                div {
                    class: "flex items-center gap-2 sm:gap-4 ml-2 pr-1",
                    button {
                        class: "text-slate-400 hover:text-white transition-all active:scale-90 p-2",
                        onclick: move |_| ctrl.play_prev(),
                        i { class: "fa-solid fa-backward-step text-lg sm:text-xl" }
                    }
                    button {
                        class: "text-slate-300 hover:text-white transition-all active:scale-90 p-2",
                        onclick: move |_| ctrl.toggle(),
                        i { class: if *is_playing.read() { "fa-solid fa-pause text-2xl" } else { "fa-solid fa-play text-2xl ml-0.5" } }
                    }
                    button {
                        class: "text-slate-400 hover:text-white transition-all active:scale-90 p-2",
                        onclick: move |_| ctrl.play_next(),
                        i { class: "fa-solid fa-forward-step text-lg sm:text-xl" }
                    }
                }
            }
        }
    }
}

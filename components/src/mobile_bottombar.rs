use dioxus::prelude::*;
use hooks::use_player_controller::PlayerController;

#[component]
pub fn MobileBottombar(
    current_song_title: Signal<String>,
    current_song_artist: Signal<String>,
    current_song_cover_url: Signal<String>,
    mut is_playing: Signal<bool>,
    mut is_fullscreen: Signal<bool>,
    mut current_song_progress: Signal<u64>,
    current_song_duration: Signal<u64>,
) -> Element {
    let mut ctrl = use_context::<PlayerController>();

    let progress_percent = if *current_song_duration.read() > 0 {
        (*current_song_progress.read() as f64 / *current_song_duration.read() as f64) * 100.0
    } else {
        0.0
    };

    rsx! {
        div {
            class: "fixed bottom-4 left-2 right-2 h-16 bg-white/10 backdrop-blur-xl border border-white/10 rounded-2xl flex items-center px-3 gap-3 z-[90] shadow-2xl overflow-hidden",
            onclick: move |_| is_fullscreen.set(true),

            // Progress bar at the bottom
            div {
                class: "absolute bottom-0 left-0 h-0.5 bg-white/20 w-full",
                div {
                    class: "h-full bg-white transition-all duration-300",
                    style: "width: {progress_percent}%"
                }
            }

            div {
                class: "w-10 h-10 bg-white/5 rounded-lg flex-shrink-0 overflow-hidden",
                if current_song_cover_url.read().is_empty() {
                    i { class: "fa-solid fa-music text-white/20 m-auto" }
                } else {
                    img { src: "{current_song_cover_url}", class: "w-full h-full object-cover" }
                }
            }

            div {
                class: "flex-1 min-w-0 flex flex-col justify-center",
                span { class: "text-xs font-bold text-white truncate", "{current_song_title}" }
                span { class: "text-[10px] text-white/60 truncate", "{current_song_artist}" }
            }

            div {
                class: "flex items-center gap-2",
                button {
                    class: "w-10 h-10 flex items-center justify-center text-white text-lg",
                    onclick: move |evt| {
                        evt.stop_propagation();
                        ctrl.toggle();
                    },
                    i { class: if *is_playing.read() { "fa-solid fa-pause" } else { "fa-solid fa-play ml-1" } }
                }
                button {
                    class: "w-10 h-10 flex items-center justify-center text-white text-lg",
                    onclick: move |evt| {
                        evt.stop_propagation();
                        ctrl.play_next();
                    },
                    i { class: "fa-solid fa-forward-step" }
                }
            }
        }
    }
}

use config::AppConfig;
use dioxus::prelude::*;
use hooks::use_player_controller::PlayerController;
use player::player::Player;
use reader::Library;

#[component]
pub fn Fullscreen(
    library: Signal<Library>,
    player: Signal<Player>,
    mut is_playing: Signal<bool>,
    mut is_fullscreen: Signal<bool>,
    mut current_song_duration: Signal<u64>,
    mut current_song_progress: Signal<u64>,
    queue: Signal<Vec<reader::Track>>,
    mut current_queue_index: Signal<usize>,
    mut current_song_title: Signal<String>,
    mut current_song_artist: Signal<String>,
    mut current_song_khz: Signal<u32>,
    mut current_song_bitrate: Signal<u8>,
    mut current_song_cover_url: Signal<String>,
    mut current_song_album: Signal<String>,
    mut volume: Signal<f32>,
) -> Element {
    if !*is_fullscreen.read() {
        return rsx! { div {} };
    }

    let mut active_tab = use_signal(|| 1usize);
    let mut ctrl = use_context::<PlayerController>();

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
        ctrl.play_track(index);
    };

    let config = use_context::<Signal<AppConfig>>();

    let get_track_cover = |track: &reader::Track| -> Option<String> {
        let lib = library.read();
        let conf = config.read();

        let is_jellyfin_track = track.path.to_string_lossy().starts_with("jellyfin:");

        if is_jellyfin_track {
            if let Some(server) = &conf.server {
                let path_str = track.path.to_string_lossy();
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
                    return Some(url);
                }
            }
            None
        } else {
            lib.albums
                .iter()
                .find(|a| a.id == track.album_id)
                .and_then(|album| utils::format_artwork_url(album.cover_path.as_ref()))
        }
    };

    rsx! {
        div {
            class: "fixed inset-0 z-50 flex text-white select-none",
            style: "background-color: var(--color-black);",

            div {
                class: "flex flex-col items-center justify-center p-8",
                style: "width: 50%; max-width: 520px;",

                div {
                    class: "rounded-lg overflow-hidden shadow-2xl mb-6",
                    style: "width: 280px; height: 280px;",
                    if current_song_cover_url.read().is_empty() {
                        div {
                            class: "w-full h-full flex items-center justify-center bg-black/30",
                            i { class: "fa-solid fa-music text-5xl text-white/20" }
                        }
                    } else {
                        img {
                            src: "{current_song_cover_url}",
                            class: "w-full h-full object-cover"
                        }
                    }
                }

                div {
                    class: "w-full mb-4",
                    style: "max-width: 340px;",
                    div {
                        class: "flex items-center gap-3",
                        span { class: "text-xs text-white/70 font-mono", style: "width: 50px; text-align: left;", "{format_time(*current_song_progress.read())}" }
                        div {
                            class: "flex-1 cursor-pointer relative",
                            style: "height: 20px;",
                            div {
                                class: "absolute bg-white/20 rounded-full",
                                style: "height: 4px; top: 8px; left: 0; right: 0;"
                            }
                            div {
                                class: "absolute rounded-full pointer-events-none",
                                style: "height: 4px; top: 8px; left: 0; width: {progress_percent}%; background: linear-gradient(to right, #5a9a9a, #ffffff);"
                            }
                            div {
                                class: "absolute bg-white rounded-full shadow-lg pointer-events-none",
                                style: "width: 12px; height: 12px; top: 4px; left: calc({progress_percent}% - 6px);"
                            }
                            input {
                                r#type: "range",
                                min: "0",
                                max: "{*current_song_duration.read()}",
                                value: "{*current_song_progress.read()}",
                                class: "absolute top-0 left-0 w-full h-full opacity-0 cursor-pointer",
                                oninput: move |evt| {
                                    if let Ok(val) = evt.value().parse::<u64>() {
                                        player.write().seek(std::time::Duration::from_secs(val));
                                        current_song_progress.set(val);
                                    }
                                }
                            }
                        }
                        span { class: "text-xs text-white/70 font-mono", style: "width: 50px; text-align: right;", "{format_time(*current_song_duration.read())}" }
                    }
                }

                div {
                    class: "text-center mb-3",
                    h2 { class: "text-base font-medium text-white", "{current_song_artist}" }
                    h1 { class: "text-lg font-bold text-white", "{current_song_title}" }
                    h3 { class: "text-sm text-white/50", "{current_song_album}" }
                }

                div {
                    class: "flex items-center justify-center gap-4 text-xs text-white/50 mb-4",
                    span { style: "font-size: 10px;", "{current_song_khz} / {current_song_bitrate}" }
                }

                div {
                    class: "flex items-center justify-center gap-8 mb-6 w-full px-10",
                    button {
                        class: "text-white/50 hover:text-white transition-colors flex-shrink-0",
                        i { class: "fa-solid fa-shuffle" }
                    }
                    button {
                        class: "text-white hover:text-white/80 transition-colors flex-shrink-0",
                        onclick: move |_| {
                            ctrl.play_prev();
                        },
                        i { class: "fa-solid fa-backward-step text-2xl" }
                    }
                    button {
                        class: "w-16 h-16 bg-white/10 hover:bg-white/20 rounded-full flex items-center justify-center transition-all flex-shrink-0 mx-2",
                        onclick: move |_| {
                            ctrl.toggle();
                        },
                        i { class: if *is_playing.read() { "fa-solid fa-pause text-2xl" } else { "fa-solid fa-play text-2xl ml-1" } }
                    }
                    button {
                        class: "text-white hover:text-white/80 transition-colors flex-shrink-0",
                        onclick: move |_| {
                            ctrl.play_next();
                        },
                        i { class: "fa-solid fa-forward-step text-2xl" }
                    }
                    button {
                        class: "text-white/50 hover:text-white transition-colors flex-shrink-0",
                        i { class: "fa-solid fa-repeat" }
                    }
                }

                div {
                    class: "flex items-center gap-5 w-full mb-auto",
                    style: "max-width: 320px;",
                    i { class: "fa-solid fa-volume-low text-white/40" }
                    div {
                        class: "flex-1 cursor-pointer relative",
                        style: "height: 20px;",
                        div {
                            class: "absolute bg-white rounded-full",
                            style: "height: 4px; top: 8px; left: 6px; right: 0;"
                        }
                        div {
                            class: "absolute bg-white/70 rounded-full pointer-events-none",
                            style: "height: 4px; top: 8px; left: 0; width: {volume_percent}%;"
                        }
                        div {
                            class: "absolute bg-white rounded-full shadow-lg pointer-events-none",
                            style: "width: 12px; height: 12px; top: 4px; left: calc({volume_percent}% - 6px);"
                        }
                        input {
                            r#type: "range",
                            min: "0",
                            max: "1",
                            step: "0.01",
                            value: "{*volume.read()}",
                            class: "absolute top-0 left-0 w-full h-full opacity-0 cursor-pointer",
                            oninput: move |evt| {
                                if let Ok(val) = evt.value().parse::<f32>() {
                                    player.read().set_volume(val);
                                    volume.set(val);
                                }
                            }
                        }
                    }
                }

                div {
                    class: "flex items-center justify-center gap-6 text-white/30 mt-8",
                    button {
                        class: "hover:text-white transition-colors",
                        onclick: move |_| is_fullscreen.set(false),
                        i { class: "fa-solid fa-chevron-down" }
                    }
                }
            }

            div {
                class: "flex-1 flex flex-col h-full",

                div {
                    class: "flex items-center gap-1 px-6 pt-4 pb-2 border-b border-white/10",
                    button {
                        class: if *active_tab.read() == 0 {
                            "px-4 py-2 text-xs font-medium tracking-wider text-white border-b-2 border-white"
                        } else {
                            "px-4 py-2 text-xs font-medium tracking-wider text-white/40 hover:text-white/70 transition-colors"
                        },
                        onclick: move |_| active_tab.set(0),
                        "BACK TO"
                    }
                    button {
                        class: if *active_tab.read() == 1 {
                            "px-4 py-2 text-xs font-medium tracking-wider text-white border-b-2 border-white"
                        } else {
                            "px-4 py-2 text-xs font-medium tracking-wider text-white/40 hover:text-white/70 transition-colors"
                        },
                        onclick: move |_| active_tab.set(1),
                        "UP NEXT"
                    }
                }

                div {
                    class: "flex-1 overflow-y-auto px-4 py-2 space-y-1",

                    if *active_tab.read() == 0 {
                        if *current_queue_index.read() == 0 {
                            div { class: "text-white/30 text-center py-10 text-sm", "No previous songs" }
                        }
                        for i in 0..*current_queue_index.read() {
                            {
                                let track = queue.read()[i].clone();
                                let cover_url = get_track_cover(&track);
                                rsx! {
                                    div {
                                        key: "{i}",
                                        class: "flex items-center gap-6 px-3 py-3 hover:bg-white/5 cursor-pointer rounded transition-colors group",
                                        onclick: move |_| play_song_at_index(i),
                                        div {
                                            class: "rounded overflow-hidden bg-black/30 shadow-md mr-6",
                                            style: "width: 42px; height: 42px; flex-shrink: 0;",
                                            if let Some(ref url) = cover_url {
                                                img { src: "{url}", class: "w-full h-full object-cover" }
                                            } else {
                                                div {
                                                    class: "w-full h-full flex items-center justify-center",
                                                    i { class: "fa-solid fa-music text-white/20", style: "font-size: 14px;" }
                                                }
                                            }
                                        }
                                        div {
                                            class: "flex-1 min-w-0 flex flex-col justify-center",
                                            div { class: "text-sm text-white truncate font-medium", "{track.title}" }
                                            div { class: "text-xs text-white/50 truncate group-hover:text-white/70", "{track.artist}" }
                                        }
                                    }
                                }
                            }
                        }
                    } else if *active_tab.read() == 1 {
                        if queue.read().len() <= *current_queue_index.read() + 1 {
                            div { class: "text-white/30 text-center py-10 text-sm", "No more songs in queue" }
                        }
                        for i in (*current_queue_index.read() + 1)..queue.read().len() {
                            {
                                let track = queue.read()[i].clone();
                                let cover_url = get_track_cover(&track);
                                rsx! {
                                    div {
                                        key: "{i}",
                                        class: "flex items-center gap-6 px-3 py-3 hover:bg-white/5 cursor-pointer rounded transition-colors group",
                                        onclick: move |_| play_song_at_index(i),
                                        div {
                                            class: "rounded overflow-hidden bg-black/30 shadow-md mr-6",
                                            style: "width: 42px; height: 42px; flex-shrink: 0;",
                                            if let Some(ref url) = cover_url {
                                                img { src: "{url}", class: "w-full h-full object-cover" }
                                            } else {
                                                div {
                                                    class: "w-full h-full flex items-center justify-center",
                                                    i { class: "fa-solid fa-music text-white/20", style: "font-size: 14px;" }
                                                }
                                            }
                                        }
                                        div {
                                            class: "flex-1 min-w-0 flex flex-col justify-center",
                                            div { class: "text-sm text-white truncate font-medium", "{track.title}" }
                                            div { class: "text-xs text-white/50 truncate group-hover:text-white/70", "{track.artist}" }
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

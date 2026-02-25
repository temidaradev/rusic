use config::AppConfig;
use dioxus::prelude::*;
use hooks::use_player_controller::PlayerController;
use reader::Library;

fn format_duration(seconds: u64) -> String {
    let minutes = seconds / 60;
    let seconds = seconds % 60;
    format!("{}:{:02}", minutes, seconds)
}

#[component]
pub fn Logs(library: Signal<Library>, config: Signal<AppConfig>) -> Element {
    let mut ctrl = use_context::<PlayerController>();

    let sorted_tracks = use_memo(move || {
        let lib = library.read();
        let conf = config.read();

        let mut all_tracks = if conf.active_source == config::MusicSource::Jellyfin {
            lib.jellyfin_tracks.clone()
        } else {
            lib.tracks.clone()
        };

        all_tracks.sort_by(|a, b| {
            let a_plays = conf
                .listen_counts
                .get(&a.path.to_string_lossy().to_string())
                .copied()
                .unwrap_or(0);
            let b_plays = conf
                .listen_counts
                .get(&b.path.to_string_lossy().to_string())
                .copied()
                .unwrap_or(0);

            match b_plays.cmp(&a_plays) {
                std::cmp::Ordering::Equal => a.title.cmp(&b.title),
                other => other,
            }
        });

        all_tracks
    });

    let conf = config.read();

    rsx! {
        div { class: "p-8 h-full overflow-y-auto w-full",
            div { class: "max-w-[1600px] mx-auto",
                div { class: "mb-8 flex items-end justify-between",
                    div {
                        h1 { class: "text-3xl font-bold text-white mb-2", "Listening Logs" }
                        p { class: "text-slate-400 text-sm", "Your most played tracks for the active source." }
                    }
                    div {
                        div { class: "w-12 h-12 rounded-full flex items-center justify-center bg-white/5 border border-white/10 text-slate-400 shadow-sm",
                            i { class: "fa-solid fa-chart-simple" }
                        }
                    }
                }

                div { class: "flex items-center px-4 py-3 mb-2 text-xs font-semibold tracking-wider text-slate-400 uppercase border-b border-white/10",
                    div { class: "w-12 shrink-0 text-center", "#" }
                    div { class: "flex-1 min-w-0 pl-14 pr-4", "Title" }
                    div { class: "w-48 lg:w-64 shrink-0 hidden md:block pr-4", "Album" }
                    div { class: "w-24 shrink-0 hidden lg:block pr-4", "Genre" }
                    div { class: "w-24 shrink-0 text-right", "Time" }
                    div { class: "w-24 shrink-0 text-right", "Plays" }
                }

                div { class: "flex flex-col pb-32 space-y-1",
                    for (idx, track) in sorted_tracks.read().iter().enumerate() {
                        {
                            let track_id = track.path.to_string_lossy().to_string();
                            let plays = conf.listen_counts.get(&track_id).copied().unwrap_or(0);

                            let genre = library.read().albums.iter()
                                .chain(library.read().jellyfin_albums.iter())
                                .find(|a| a.id == track.album_id)
                                .map(|a| a.genre.clone())
                                .unwrap_or_default();

                            let is_jellyfin = track.path.to_string_lossy().starts_with("jellyfin:");
                            let source_icon = if is_jellyfin { "fa-database" } else { "fa-hard-drive" };
                            let source_title = if is_jellyfin { "Jellyfin" } else { "Local" };

                            let cover_url = if is_jellyfin {
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
                                        Some(url)
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            } else {
                                library.read().albums.iter()
                                    .find(|a| a.id == track.album_id)
                                    .and_then(|a| a.cover_path.as_ref())
                                    .and_then(|p| utils::format_artwork_url(Some(&p)))
                            };

                            rsx! {
                                div {
                                    key: "{track_id}",
                                    class: "flex items-center px-4 py-2 hover:bg-white/5 rounded-xl cursor-pointer transition-colors group",
                                    onclick: move |_| {
                                        ctrl.queue.set(sorted_tracks.read().clone());
                                        ctrl.play_track(idx);
                                    },
                                    div { class: "w-12 shrink-0 flex items-center justify-center tabular-nums text-slate-500 font-medium group-hover:text-white transition-colors relative",
                                        span { class: "group-hover:opacity-0 transition-opacity", "{idx + 1}" }
                                        i { class: "fa-solid fa-play absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 opacity-0 group-hover:opacity-100 transition-opacity" }
                                    }

                                    div { class: "flex-1 min-w-0 pr-4 flex items-center",
                                        div { class: "w-10 h-10 bg-white/5 rounded-md shadow-sm flex items-center justify-center mr-4 shrink-0 text-slate-500 group-hover:text-slate-300 transition-colors overflow-hidden",
                                            if let Some(url) = cover_url {
                                                img { src: "{url}", class: "w-full h-full object-cover" }
                                            } else {
                                                i { class: "fa-solid fa-music text-xs" }
                                            }
                                        }
                                        div { class: "flex-1 min-w-0",
                                            div { class: "text-white font-medium truncate text-[15px] mb-0.5 flex items-center gap-2",
                                                "{track.title}"
                                                i {
                                                    class: "fa-solid {source_icon} text-[10px] text-slate-500",
                                                    title: "{source_title}"
                                                }
                                            }
                                            div { class: "text-slate-400 text-sm truncate group-hover:text-slate-300 transition-colors", "{track.artist}" }
                                        }
                                    }

                                    div { class: "w-48 lg:w-64 shrink-0 hidden md:block text-slate-400 text-sm truncate pr-4 group-hover:text-slate-300 transition-colors",
                                        "{track.album}"
                                    }

                                    div { class: "w-24 shrink-0 hidden lg:block text-slate-400 text-sm truncate pr-4 group-hover:text-slate-300 transition-colors",
                                        if genre.is_empty() {
                                            "-"
                                        } else {
                                            "{genre}"
                                        }
                                    }

                                    div { class: "w-24 shrink-0 text-right text-slate-400 text-sm tabular-nums group-hover:text-slate-300 transition-colors",
                                        "{format_duration(track.duration)}"
                                    }

                                    div { class: "w-24 shrink-0 text-right text-slate-400 text-sm tabular-nums group-hover:text-slate-300 transition-colors flex items-center justify-end gap-2",
                                        if plays > 0 {
                                            i { class: "fa-solid fa-fire text-orange-500/80 text-[10px]" }
                                        }
                                        span { class: if plays > 0 { "text-white font-medium" } else { "" }, "{plays}" }
                                    }
                                }
                            }
                        }
                    }
                    if sorted_tracks.read().is_empty() {
                        div { class: "flex flex-col items-center justify-center py-24 text-slate-500",
                            i { class: "fa-solid fa-headphones text-4xl mb-4 opacity-50" }
                            p { "No tracks found in your library." }
                        }
                    }
                }
            }
        }
    }
}

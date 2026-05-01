use ::server::youtube_music::{YTMHomeSection, YTMTrack, yt_dlp_home, yt_dlp_search};
use components::track_row::TrackRow;
use config::AppConfig;
use dioxus::prelude::*;
use hooks::use_player_controller::PlayerController;
use reader::models::Track;
use std::path::PathBuf;

fn ytm_to_track(t: &YTMTrack) -> Track {
    Track {
        path: PathBuf::from(format!("ytmusic:{}", t.video_id)),
        album_id: t.video_id.clone(),
        title: t.title.clone(),
        artist: t.artist.clone(),
        album: t.album.clone().unwrap_or_default(),
        duration: t.duration_seconds.unwrap_or(0),
        khz: 0,
        bitrate: 0,
        track_number: None,
        disc_number: None,
        musicbrainz_release_id: None,
        playlist_item_id: None,
    }
}

#[component]
pub fn YouTubeMusicHome() -> Element {
    let config = use_context::<Signal<AppConfig>>();
    let mut ctrl = use_context::<PlayerController>();

    let mut sections = use_signal(Vec::<YTMHomeSection>::new);
    let mut loading = use_signal(|| true);
    let mut error = use_signal(|| Option::<String>::None);

    use_effect(move || {
        let browser = config.read().ytm_browser.clone();

        spawn(async move {
            match yt_dlp_home(browser.as_deref()).await {
                Ok(s) => {
                    sections.set(s);
                    loading.set(false);
                }
                Err(e) => {
                    error.set(Some(format!("Failed to load home feed: {}", e)));
                    loading.set(false);
                }
            }
        });
    });

    let scroll_section = move |id: String, dir: i32| {
        let script = format!(
            "document.getElementById('{}').scrollBy({{ left: {}, behavior: 'smooth' }})",
            id,
            dir * 300
        );
        let _ = document::eval(&script);
    };

    rsx! {
        div { class: "p-8 h-full overflow-y-auto w-full",
            div { class: "max-w-[1600px] mx-auto",
                div { class: "mb-8 flex items-center gap-4",
                    div { class: "w-12 h-12 rounded-xl bg-red-500/20 flex items-center justify-center",
                        i { class: "fa-brands fa-youtube text-red-400 text-xl" }
                    }
                    div {
                        h1 { class: "text-3xl font-bold text-white", "YouTube Music" }
                        p { class: "text-slate-400 text-sm", "Your feed" }
                    }
                }

                if *loading.read() {
                    div { class: "flex items-center gap-3 text-slate-400 py-12",
                        div { class: "w-5 h-5 border-2 border-slate-600 border-t-white rounded-full animate-spin" }
                        "Loading feed..."
                    }
                } else if let Some(err) = error.read().clone() {
                    div { class: "text-red-400 py-8", "{err}" }
                } else if sections.read().is_empty() {
                    div { class: "text-slate-400 py-12 text-center",
                        i { class: "fa-solid fa-music text-4xl mb-4 block opacity-30" }
                        p { "No content available." }
                        p { class: "text-sm mt-1 text-slate-500", "Try the search tab to find music." }
                    }
                } else {
                    div { class: "space-y-12 pb-32",
                        for (section_idx, section) in sections.read().iter().enumerate() {
                            {
                                let section_id = format!("ytm-section-{}", section_idx);
                                let sid_left = section_id.clone();
                                let sid_right = section_id.clone();
                                let title = section.title.clone();
                                let section_tracks = section.tracks.clone();

                                rsx! {
                                    section {
                                        div { class: "flex items-center justify-between mb-4",
                                            h2 { class: "text-xl font-bold text-white tracking-tight", "{title}" }
                                            div { class: "flex gap-2",
                                                button {
                                                    class: "w-8 h-8 rounded-full bg-white/5 hover:bg-white/10 flex items-center justify-center text-white transition-all hover:scale-105",
                                                    onclick: move |_| scroll_section(sid_left.clone(), -1),
                                                    i { class: "fa-solid fa-chevron-left text-sm" }
                                                }
                                                button {
                                                    class: "w-8 h-8 rounded-full bg-white/5 hover:bg-white/10 flex items-center justify-center text-white transition-all hover:scale-105",
                                                    onclick: move |_| scroll_section(sid_right.clone(), 1),
                                                    i { class: "fa-solid fa-chevron-right text-sm" }
                                                }
                                            }
                                        }
                                        div {
                                            id: "{section_id}",
                                            class: "flex overflow-x-auto gap-4 pb-4 scrollbar-hide scroll-smooth -mx-2 px-2",
                                            for (track_idx, track) in section_tracks.iter().enumerate() {
                                                {
                                                    let title = track.title.clone();
                                                    let artist = track.artist.clone();
                                                    let thumb = track.thumbnail_url.clone();
                                                    let all_tracks = section_tracks.clone();

                                                    rsx! {
                                                        div {
                                                            class: "flex-none w-36 md:w-44 group cursor-pointer",
                                                            onclick: move |_| {
                                                                let track_list: Vec<Track> = all_tracks.iter().map(ytm_to_track).collect();
                                                                ctrl.queue.set(track_list);
                                                                ctrl.play_track(track_idx);
                                                            },
                                                            div { class: "aspect-square rounded-xl bg-stone-800/80 mb-3 overflow-hidden relative",
                                                                if let Some(url) = &thumb {
                                                                    img {
                                                                        src: "{url}",
                                                                        class: "w-full h-full object-cover",
                                                                        loading: "lazy",
                                                                        decoding: "async"
                                                                    }
                                                                } else {
                                                                    div { class: "w-full h-full flex items-center justify-center border border-white/5",
                                                                        i { class: "fa-solid fa-music text-3xl text-white/20" }
                                                                    }
                                                                }
                                                                div { class: "absolute inset-0 bg-black/0 group-hover:bg-black/30 transition-colors duration-200 flex items-center justify-center",
                                                                    div { class: "w-10 h-10 bg-white rounded-full flex items-center justify-center opacity-0 group-hover:opacity-100 transition-all duration-200 translate-y-2 group-hover:translate-y-0",
                                                                        i { class: "fa-solid fa-play text-black ml-0.5 text-sm" }
                                                                    }
                                                                }
                                                            }
                                                            p { class: "text-white font-semibold text-xs md:text-sm truncate px-1", "{title}" }
                                                            p { class: "text-white/50 text-xs truncate px-1 mt-0.5", "{artist}" }
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
            }
        }
    }
}

#[component]
pub fn YouTubeMusicSearch(search_query: Signal<String>) -> Element {
    let config = use_context::<Signal<AppConfig>>();
    let mut ctrl = use_context::<PlayerController>();

    let mut results = use_signal(Vec::<YTMTrack>::new);
    let mut loading = use_signal(|| false);
    let mut error = use_signal(|| Option::<String>::None);
    let mut active_menu = use_signal(|| Option::<String>::None);
    let mut last_query = use_signal(String::new);

    let mut do_search = move || {
        let query = search_query.read().trim().to_string();
        if query.is_empty() || query == *last_query.read() {
            return;
        }
        last_query.set(query.clone());

        let browser = config.read().ytm_browser.clone();

        loading.set(true);
        error.set(None);

        spawn(async move {
            match yt_dlp_search(&query, browser.as_deref()).await {
                Ok(r) => {
                    results.set(r);
                    loading.set(false);
                }
                Err(e) => {
                    error.set(Some(format!("Search failed: {}", e)));
                    loading.set(false);
                }
            }
        });
    };

    let track_list = use_memo(move || results.read().iter().map(ytm_to_track).collect::<Vec<_>>());

    rsx! {
        div { class: "p-8 h-full overflow-y-auto w-full",
            div { class: "max-w-[1600px] mx-auto",
                div { class: "mb-6 flex items-center gap-4",
                    div { class: "flex-1 relative",
                        i { class: "fa-solid fa-magnifying-glass absolute left-4 top-1/2 -translate-y-1/2 text-slate-400" }
                        input {
                            class: "w-full bg-white/5 border border-white/10 rounded-xl pl-11 pr-4 py-3 text-white placeholder-slate-500 focus:outline-none focus:border-white/20",
                            placeholder: "Search YouTube Music...",
                            value: "{search_query}",
                            oninput: move |e| {
                                search_query.set(e.value());
                            },
                            onkeydown: move |e| {
                                if e.key() == Key::Enter {
                                    do_search();
                                }
                            },
                        }
                    }
                    button {
                        class: "bg-red-500/20 hover:bg-red-500/30 text-red-400 px-5 py-3 rounded-xl font-medium transition-colors",
                        onclick: move |_| do_search(),
                        "Search"
                    }
                }

                if *loading.read() {
                    div { class: "flex items-center gap-3 text-slate-400 py-12",
                        div { class: "w-5 h-5 border-2 border-slate-600 border-t-white rounded-full animate-spin" }
                        "Searching..."
                    }
                } else if let Some(err) = error.read().clone() {
                    div { class: "text-red-400 py-8", "{err}" }
                } else if track_list.read().is_empty() && !last_query.read().is_empty() {
                    div { class: "text-slate-400 py-12 text-center", "No results found." }
                } else {
                    div { class: "flex flex-col space-y-1 pb-32",
                        for (idx, track) in track_list.read().iter().enumerate() {
                            {
                                let track_clone = track.clone();
                                let ytm_snap = results.read().clone();
                                let cover = ytm_snap.get(idx).and_then(|t| t.thumbnail_url.clone());
                                let path = track.path.clone();
                                let is_open = active_menu.read().as_deref() == Some(&track.path.to_string_lossy());

                                rsx! {
                                    TrackRow {
                                        key: "{idx}",
                                        track: track_clone,
                                        cover_url: cover,
                                        is_menu_open: is_open,
                                        on_click_menu: move |_| {
                                            let p = path.to_string_lossy().to_string();
                                            if active_menu.read().as_deref() == Some(&p) {
                                                active_menu.set(None);
                                            } else {
                                                active_menu.set(Some(p));
                                            }
                                        },
                                        on_close_menu: move |_| active_menu.set(None),
                                        on_add_to_playlist: move |_| {},
                                        on_delete: move |_| active_menu.set(None),
                                        on_remove_from_playlist: None,
                                        on_select: None,
                                        on_long_press: None,
                                        hide_delete: true,
                                        on_play: move |_| {
                                            let all: Vec<Track> = track_list.read().clone();
                                            ctrl.queue.set(all);
                                            ctrl.play_track(idx);
                                        },
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

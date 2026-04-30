use ::server::youtube_music::{YTMTrack, YouTubeMusicClient};
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

    let mut tracks = use_signal(Vec::<YTMTrack>::new);
    let mut loading = use_signal(|| false);
    let mut error = use_signal(|| Option::<String>::None);
    let mut active_menu = use_signal(|| Option::<String>::None);

    use_effect(move || {
        let token = config.read().ytm_access_token.clone().unwrap_or_default();

        if token.is_empty() {
            error.set(Some(
                "YouTube Music not connected. Go to Settings to connect.".to_string(),
            ));
            return;
        }

        loading.set(false);
    });

    let track_list = use_memo(move || tracks.read().iter().map(ytm_to_track).collect::<Vec<_>>());

    rsx! {
        div { class: "p-8 h-full overflow-y-auto w-full",
            div { class: "max-w-[1600px] mx-auto",
                div { class: "mb-8 flex items-center gap-4",
                    div { class: "w-12 h-12 rounded-xl bg-red-500/20 flex items-center justify-center",
                        i { class: "fa-brands fa-youtube text-red-400 text-xl" }
                    }
                    div {
                        h1 { class: "text-3xl font-bold text-white", "YouTube Music" }
                        p { class: "text-slate-400 text-sm", "Your library" }
                    }
                }

                if *loading.read() {
                    div { class: "flex items-center gap-3 text-slate-400 py-12",
                        div { class: "w-5 h-5 border-2 border-slate-600 border-t-white rounded-full animate-spin" }
                        "Loading library..."
                    }
                } else if let Some(err) = error.read().clone() {
                    div { class: "text-red-400 py-8", "{err}" }
                } else if track_list.read().is_empty() {
                    div { class: "text-slate-400 py-12 text-center",
                        i { class: "fa-solid fa-music text-4xl mb-4 block opacity-30" }
                        p { "Welcome to YouTube Music." }
                        p { class: "text-sm mt-1 text-slate-500", "Use the search or explore tabs to find music." }
                    }
                } else {
                    div { class: "flex flex-col space-y-1 pb-32",
                        for (idx, track) in track_list.read().iter().enumerate() {
                            {
                                let track_clone = track.clone();
                                let ytm_tracks_snap = tracks.read().clone();
                                let cover = ytm_tracks_snap
                                    .get(idx)
                                    .and_then(|t| t.thumbnail_url.clone());
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

        let token = config.read().ytm_access_token.clone().unwrap_or_default();

        if token.is_empty() {
            error.set(Some("YouTube Music not connected.".to_string()));
            return;
        }

        loading.set(true);
        error.set(None);

        spawn(async move {
            let client = YouTubeMusicClient::new(&token);
            match client.search(&query).await {
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

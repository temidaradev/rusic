use crate::player::player::Player;
use dioxus::prelude::*;

mod components;
pub mod config;
pub mod hooks;
pub mod pages;
pub mod player;
pub mod reader;
pub mod utils;
pub mod web_audio_store;
use components::{bottombar::Bottombar, sidebar::Sidebar};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Route {
    Home,
    Search,
    Library,
    Album,
    Playlists,
    Settings,
}

const FAVICON: Asset = asset!("assets/favicon.ico");
const MAIN_CSS: Asset = asset!("assets/main.css");
const THEME_CSS: Asset = asset!("assets/themes.css");
const TAILWIND_CSS: Asset = asset!("assets/tailwind.css");

fn main() {
    web_audio_store::init_store();
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let mut library = use_signal(reader::Library::default);
    let mut current_route = use_signal(|| Route::Home);
    let mut show_reload_prompt = use_signal(|| false);

    let cache_dir = use_memo(|| std::path::PathBuf::from("rusic_cache"));

    let lib_path = use_memo(move || cache_dir().join("library.json"));
    let config_path = use_memo(move || cache_dir().join("config.json"));
    let config = use_signal(|| config::AppConfig::load(&config_path()));
    let playlist_path = use_memo(move || cache_dir().join("playlists.json"));
    let playlist_store =
        use_signal(|| reader::PlaylistStore::load(&playlist_path()).unwrap_or_default());

    let current_playing = use_signal(|| 0);
    let player = use_signal(Player::new);
    let current_song_cover_url = use_signal(|| String::new());
    let current_song_title = use_signal(|| String::new());
    let current_song_artist = use_signal(|| String::new());
    let current_song_duration = use_signal(|| 0u64);
    let current_song_progress = use_signal(|| 0u64);
    let volume = use_signal(|| 1.0f32);

    let is_playing = use_signal(|| false);

    let mut selected_album_id = use_signal(|| String::new());
    let mut search_query = use_signal(|| String::new());

    use_effect(move || {
        let _ = config.read().save(&config_path());
    });

    use_effect(move || {
        let _ = playlist_store.read().save(&playlist_path());
    });

    use_hook(move || {
        spawn(async move {
            if let Ok(loaded) = reader::Library::load(&lib_path()) {
                let has_tracks = !loaded.tracks.is_empty();
                library.set(loaded);

                if has_tracks && web_audio_store::file_count() == 0 {
                    show_reload_prompt.set(true);
                }
            }
        });
    });

    let queue = use_signal(|| Vec::<reader::Track>::new());
    let current_queue_index = use_signal(|| 0usize);

    let ctrl = hooks::use_player_controller(
        player,
        is_playing,
        queue,
        current_queue_index,
        current_song_title,
        current_song_artist,
        current_song_duration,
        current_song_progress,
        current_song_cover_url,
        volume,
        library,
    );
    provide_context(ctrl);

    hooks::use_player_task(ctrl);

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: THEME_CSS }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        document::Link { rel: "stylesheet", href: "https://fonts.bunny.net/css?family=jetbrains-mono:400,500,700,800&display=swap" }
        document::Link { rel: "stylesheet", href: "https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.5.1/css/all.min.css" }

        if *show_reload_prompt.read() {
            components::folder_reload_prompt::FolderReloadPrompt {
                config: config,
                library: library,
                on_dismiss: move |_| show_reload_prompt.set(false)
            }
        }

        div {
            class: "flex flex-col h-screen theme-{config.read().theme}",
            div {
                class: "flex flex-1 overflow-hidden",
                Sidebar {
                    current_route,
                    on_navigate: move |route| {
                        if route == Route::Album {
                            selected_album_id.set(String::new());
                        }
                        if route == Route::Search && !search_query.read().is_empty() {
                        }
                        current_route.set(route);
                    }
                }
                div {
                    class: "flex-1 overflow-y-auto",
                    match *current_route.read() {
                        Route::Home => rsx! {
                            pages::home::Home {
                                library,
                                playlist_store,
                                on_select_album: move |id| {
                                    selected_album_id.set(id);
                                    current_route.set(Route::Album);
                                },
                                on_search_artist: move |artist| {
                                    search_query.set(artist);
                                    current_route.set(Route::Search);
                                }
                            }
                        },
                        Route::Search => rsx! {
                            pages::search::Search {
                                library: library,
                                playlist_store: playlist_store,
                                search_query: search_query,
                                player: player.clone(),
                                is_playing: is_playing,
                                current_playing: current_playing,
                                current_song_cover_url: current_song_cover_url,
                                current_song_title: current_song_title,
                                current_song_artist: current_song_artist,
                                current_song_duration: current_song_duration,
                                current_song_progress: current_song_progress,
                                queue: queue,
                                current_queue_index: current_queue_index,
                            }
                        },
                        Route::Library => rsx! {
                            pages::library::LibraryPage {
                                library: library,
                                playlist_store: playlist_store,
                                on_rescan: move |_| {},
                                player: player.clone(),
                                is_playing: is_playing,
                                current_playing: current_playing,
                                current_song_cover_url: current_song_cover_url,
                                current_song_title: current_song_title,
                                current_song_artist: current_song_artist,
                                current_song_duration: current_song_duration,
                                current_song_progress: current_song_progress,
                                queue: queue,
                                current_queue_index: current_queue_index,
                            }
                        },
                        Route::Album => rsx! {
                            pages::album::Album {
                                library: library,
                                album_id: selected_album_id,
                                playlist_store: playlist_store,
                                player: player.clone(),
                                is_playing: is_playing,
                                current_playing: current_playing,
                                current_song_cover_url: current_song_cover_url,
                                current_song_title: current_song_title,
                                current_song_artist: current_song_artist,
                                current_song_duration: current_song_duration,
                                current_song_progress: current_song_progress,
                                queue: queue,
                                current_queue_index: current_queue_index,
                            }
                        },
                        Route::Playlists => rsx! {
                            pages::playlists::PlaylistsPage {
                                playlist_store: playlist_store,
                                library: library,
                                player: player.clone(),
                                is_playing: is_playing,
                                current_playing: current_playing,
                                current_song_cover_url: current_song_cover_url,
                                current_song_title: current_song_title,
                                current_song_artist: current_song_artist,
                                current_song_duration: current_song_duration,
                                current_song_progress: current_song_progress,
                                queue: queue,
                                current_queue_index: current_queue_index,
                            }
                        },
                        Route::Settings => rsx! { pages::settings::Settings { config, library } },
                    }
                }
            }
            Bottombar {
                library: library,
                current_song_cover_url: current_song_cover_url,
                current_song_title: current_song_title,
                current_song_artist: current_song_artist,
                player: player,
                is_playing: is_playing,
                current_song_duration: current_song_duration,
                current_song_progress: current_song_progress,
                queue: queue,
                current_queue_index: current_queue_index,
                volume: volume,
            }
        }
    }
}

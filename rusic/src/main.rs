use components::{bottombar::Bottombar, fullscreen::Fullscreen, sidebar::Sidebar};
use dioxus::desktop::tao::dpi::LogicalSize;
#[cfg(target_os = "macos")]
use dioxus::desktop::tao::platform::macos::WindowBuilderExtMacOS;
use dioxus::prelude::*;
use discord_presence::Presence;
use player::player::Player;
use rusic_route::Route;
use std::{borrow::Cow, sync::Arc};

const FAVICON: Asset = asset!("../assets/favicon.ico");
const MAIN_CSS: Asset = asset!("../assets/main.css");
const THEME_CSS: Asset = asset!("../assets/themes.css");
const TAILWIND_CSS: Asset = asset!("../assets/tailwind.css");

static PRESENCE: std::sync::OnceLock<Option<Arc<Presence>>> = std::sync::OnceLock::new();

fn main() {
    let presence: Option<Arc<Presence>> = match Presence::new("1470087339639443658") {
        Ok(p) => {
            println!("Discord presence connected!");
            Some(Arc::new(p))
        }
        Err(e) => {
            eprintln!("Failed to connect to Discord: {e}");
            None
        }
    };

    PRESENCE.set(presence).ok();

    #[cfg(target_os = "macos")]
    {
        player::systemint::init();
    }

    let mut window = dioxus::desktop::WindowBuilder::new()
        .with_title("Rusic")
        .with_resizable(true)
        .with_inner_size(LogicalSize::new(1350.0, 800.0));

    #[cfg(target_os = "macos")]
    {
        window = window
            .with_title_hidden(true)
            .with_titlebar_transparent(true)
            .with_fullsize_content_view(true);
    }

    let config = dioxus::desktop::Config::new()
        .with_window(window)
        .with_custom_protocol("artwork", |_headers, request| {
            let path = request.uri().path();
            let decoded = percent_encoding::percent_decode_str(path).decode_utf8_lossy();

            let mime = if decoded.ends_with(".png") {
                "image/png"
            } else {
                "image/jpeg"
            };

            let mut decoded_path = decoded.to_string();

            if decoded_path.starts_with("/~") {
                if let Ok(home) = std::env::var("HOME") {
                    decoded_path = decoded_path.replacen("/~", &home, 1);
                }
            } else if decoded_path.starts_with('~') {
                if let Ok(home) = std::env::var("HOME") {
                    decoded_path = decoded_path.replacen('~', &home, 1);
                }
            }

            let path = std::path::Path::new(&decoded_path);
            let content = std::fs::read(path)
                .or_else(|_| {
                    if decoded_path.strip_prefix('/').is_some() {
                        std::fs::read(std::path::Path::new(&decoded_path[1..]))
                    } else {
                        Err(std::io::Error::from(std::io::ErrorKind::NotFound))
                    }
                })
                .map(Cow::from)
                .unwrap_or_else(|_| std::borrow::Cow::from(Vec::new()));

            http::Response::builder()
                .header("Content-Type", mime)
                .header("Access-Control-Allow-Origin", "*")
                .body(content)
                .unwrap()
        });

    dioxus::LaunchBuilder::desktop()
        .with_cfg(config)
        .launch(App);
}

#[component]
fn App() -> Element {
    let mut library = use_signal(reader::Library::default);
    let mut current_route = use_signal(|| Route::Home);
    let cache_dir = use_memo(move || {
        let path = directories::ProjectDirs::from("com", "temidaradev", "rusic")
            .map(|dirs| dirs.cache_dir().to_path_buf())
            .unwrap_or_else(|| std::path::Path::new("./cache").to_path_buf());
        let _ = std::fs::create_dir_all(&path);
        path
    });
    let config_dir = use_memo(move || {
        let path = directories::ProjectDirs::from("com", "temidaradev", "rusic")
            .map(|dirs| dirs.config_dir().to_path_buf())
            .unwrap_or_else(|| std::path::Path::new("./config").to_path_buf());
        let _ = std::fs::create_dir_all(&path);
        path
    });
    let lib_path = use_memo(move || cache_dir().join("library.json"));
    let config_path = use_memo(move || config_dir().join("config.json"));
    let config = use_signal(|| config::AppConfig::load(&config_path()));
    let playlist_path = use_memo(move || cache_dir().join("playlists.json"));
    let playlist_store =
        use_signal(|| reader::PlaylistStore::load(&playlist_path()).unwrap_or_default());
    let cover_cache = use_memo(move || cache_dir().join("covers"));
    let _ = std::fs::create_dir_all(cover_cache());
    let mut trigger_rescan = use_signal(|| 0);
    let current_playing = use_signal(|| 0);
    let player = use_signal(Player::new);
    //why changed all use_signal(|| String::new()) to use_signal(String::new) it is because Needlessly creating
    //a closure adds code for no benefit and gives the optimizer more work.
    let current_song_cover_url = use_signal(String::new);
    let current_song_title = use_signal(String::new);
    let current_song_artist = use_signal(String::new);
    let current_song_album = use_signal(String::new);
    let current_song_duration = use_signal(|| 0u64);
    let current_song_khz = use_signal(|| 0u32);
    let current_song_bitrate = use_signal(|| 0u8);
    let current_song_progress = use_signal(|| 0u64);
    let volume = use_signal(|| 1.0f32);

    let is_playing = use_signal(|| false);
    let is_fullscreen = use_signal(|| false);

    let presence = PRESENCE.get().cloned().flatten();

    provide_context(presence.clone());

    let mut selected_album_id = use_signal(String::new);
    let mut selected_artist_name = use_signal(String::new);
    let search_query = use_signal(String::new);

    use_effect(move || {
        if let Err(e) = config.read().save(&config_path()) {
            eprintln!("Failed to save config: {}", e);
        }
    });

    use_effect(move || {
        if let Err(e) = playlist_store.read().save(&playlist_path()) {
            eprintln!("Failed to save playlists: {}", e);
        }
    });

    use_effect(move || {
        if let Err(e) = library.read().save(&lib_path()) {
            eprintln!("Failed to save library: {}", e);
        }
    });

    use_hook(move || {
        spawn(async move {
            if let Ok(loaded) = reader::Library::load(&lib_path()) {
                library.set(loaded);
            }
        });
    });

    use_effect(move || {
        let music_dir = config.read().music_directory.clone();
        let _ = trigger_rescan.read();

        spawn(async move {
            if music_dir.exists() {
                let mut current_lib = library.peek().clone();

                if current_lib.root_path != music_dir {
                    current_lib = reader::Library::new(music_dir.clone());
                    library.set(current_lib.clone());
                }

                if (reader::scan_directory(music_dir, cover_cache(), &mut current_lib).await)
                    .is_ok()
                {
                    library.set(current_lib.clone());
                    let _ = current_lib.save(&lib_path());
                }
            }
        });
    });

    let queue = use_signal(Vec::<reader::Track>::new);
    let current_queue_index = use_signal(|| 0usize);

    let mut ctrl = hooks::use_player_controller(
        player,
        is_playing,
        queue,
        current_queue_index,
        current_song_title,
        current_song_artist,
        current_song_album,
        current_song_khz,
        current_song_bitrate,
        current_song_duration,
        current_song_progress,
        current_song_cover_url,
        volume,
        library,
        config,
    );
    provide_context(ctrl);
    provide_context(config);

    hooks::use_player_task(ctrl);

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: THEME_CSS }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        document::Link { rel: "stylesheet", href: "https://fonts.bunny.net/css?family=jetbrains-mono:400,500,700,800&display=swap" }
        document::Link { rel: "stylesheet", href: "https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.5.1/css/all.min.css" }
        div {
            class: "flex flex-col h-screen theme-{config.read().theme}",
            tabindex: "0",
            autofocus: true,
            onkeydown: move |evt| {
                use dioxus::prelude::Key;
                let key = evt.key();
                if key == Key::Character(" ".into()) {
                    ctrl.toggle();
                }
            },
            div {
                class: "flex flex-1 overflow-hidden",
                Sidebar {
                    current_route,
                    on_navigate: move |route| {
                        if route == Route::Album {
                            selected_album_id.set(String::new());
                        }
                        if route == Route::Artist {
                            selected_artist_name.set(String::new());
                        }
                        if route == Route::Search && !search_query.read().is_empty() {
                            // Keep search query if already set? Or maybe clear it?
                            // For now keep it.
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
                                    selected_artist_name.set(artist);
                                    current_route.set(Route::Artist);
                                }
                            }
                        },
                        Route::Search => rsx! {
                            pages::search::Search {
                                library: library,
                                config: config,
                                playlist_store: playlist_store,
                                search_query: search_query,
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
                            }
                        },
                        Route::Library => rsx! {
                            pages::library::LibraryPage {
                                library: library,
                                config: config,
                                playlist_store: playlist_store,
                                on_rescan: move |_| *trigger_rescan.write() += 1,
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
                            }
                        },
                        Route::Album => rsx! {
                            pages::album::Album {
                                library: library,
                                config: config,
                                album_id: selected_album_id,
                                playlist_store: playlist_store,
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
                            }
                        },
                        Route::Artist => rsx! {
                            pages::artist::Artist {
                                library: library,
                                config: config,
                                artist_name: selected_artist_name,
                                playlist_store: playlist_store,
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
                                on_close: move |_evt: ()| {
                                    selected_artist_name.set(String::new());
                                    current_route.set(Route::Home);
                                }
                            }
                        },
                        Route::Playlists => rsx! {
                            pages::playlists::PlaylistsPage {
                                playlist_store: playlist_store,
                                library: library,
                                config: config,
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
                            }
                        },
                        Route::Logs => rsx! {
                          pages::logs::Logs {
                              library: library,
                              config: config,
                          }
                        },
                        Route::Settings => rsx! { pages::settings::Settings { config } },
                    }
                }
            }
            Fullscreen {
                library: library,
                player: player,
                is_playing: is_playing,
                is_fullscreen: is_fullscreen,
                current_song_duration: current_song_duration,
                current_song_progress: current_song_progress,
                queue: queue,
                current_song_album: current_song_album,
                current_queue_index: current_queue_index,
                current_song_title: current_song_title,
                current_song_khz: current_song_khz,
                current_song_bitrate: current_song_bitrate,
                current_song_artist: current_song_artist,
                current_song_cover_url: current_song_cover_url,
                volume: volume,
            }
            Bottombar {
                library: library,
                current_song_cover_url: current_song_cover_url,
                current_song_title: current_song_title,
                current_song_artist: current_song_artist,
                player: player,
                is_playing: is_playing,
                is_fullscreen: is_fullscreen,
                current_song_duration: current_song_duration,
                current_song_progress: current_song_progress,
                queue: queue,
                current_queue_index: current_queue_index,
                volume: volume,
            }
        }
    }
}

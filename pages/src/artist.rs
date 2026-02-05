use config::{AppConfig, MusicSource};
use dioxus::prelude::*;
use player::player;
use reader::{Library, PlaylistStore};

#[component]
pub fn Artist(
    library: Signal<Library>,
    config: Signal<AppConfig>,
    artist_name: Signal<String>,
    playlist_store: Signal<PlaylistStore>,
    player: Signal<player::Player>,
    mut is_playing: Signal<bool>,
    mut current_song_cover_url: Signal<String>,
    mut current_song_title: Signal<String>,
    mut current_song_artist: Signal<String>,
    mut current_song_duration: Signal<u64>,
    mut current_song_progress: Signal<u64>,
    mut queue: Signal<Vec<reader::models::Track>>,
    mut current_queue_index: Signal<usize>,
    on_close: EventHandler<()>,
) -> Element {
    let is_jellyfin = config.read().active_source == MusicSource::Jellyfin;

    let artist_tracks = use_memo(move || {
        let lib = library.read();
        let artist = artist_name.read();

        if artist.is_empty() {
            return Vec::new();
        }

        if is_jellyfin {
            lib.jellyfin_tracks
                .iter()
                .filter(|t| t.artist.to_lowercase() == artist.to_lowercase())
                .cloned()
                .collect::<Vec<_>>()
        } else {
            lib.tracks
                .iter()
                .filter(|t| t.artist.to_lowercase() == artist.to_lowercase())
                .cloned()
                .collect::<Vec<_>>()
        }
    });

    let artist_cover = use_memo(move || {
        let lib = library.read();
        let conf = config.read();
        let artist = artist_name.read();

        if artist.is_empty() {
            return None;
        }

        if is_jellyfin {
            lib.jellyfin_albums
                .iter()
                .find(|a| a.artist.to_lowercase() == artist.to_lowercase())
                .and_then(|album| {
                    if let Some(server) = &conf.server {
                        if let Some(cover_path) = &album.cover_path {
                            let path_str = cover_path.to_string_lossy();
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
                    }
                    None
                })
        } else {
            lib.albums
                .iter()
                .find(|a| a.artist.to_lowercase() == artist.to_lowercase())
                .and_then(|album| utils::format_artwork_url(album.cover_path.as_ref()))
        }
    });

    let name = artist_name.read().clone();

    rsx! {
        div {
            class: "p-8 pb-24",

            if name.is_empty() {
                div { class: "text-slate-500", "No artist selected" }
            } else {
                components::showcase::Showcase {
                    name: name.clone(),
                    description: "Artist".to_string(),
                    cover_url: artist_cover(),
                    tracks: artist_tracks(),
                    library: library,
                    active_track: None,
                    on_play: move |idx: usize| {
                        let tracks = artist_tracks();
                        queue.set(tracks.clone());
                        current_queue_index.set(idx);

                        if let Some(t) = tracks.get(idx) {
                            if is_jellyfin {
                                let mut ctrl =
                                    use_context::<hooks::use_player_controller::PlayerController>();
                                ctrl.play_track(idx);
                            } else {
                                let file = match std::fs::File::open(&t.path) {
                                    Ok(f) => f,
                                    Err(_) => return,
                                };
                                let source =
                                    match rodio::Decoder::new(std::io::BufReader::new(file)) {
                                        Ok(s) => s,
                                        Err(_) => return,
                                    };

                                let lib = library.peek();
                                let album_info = lib.albums.iter().find(|a| a.id == t.album_id);
                                let artwork = album_info.and_then(|a| {
                                    a.cover_path.as_ref().map(|p| p.to_string_lossy().into_owned())
                                });

                                let meta = player::NowPlayingMeta {
                                    title: t.title.clone(),
                                    artist: t.artist.clone(),
                                    album: t.album.clone(),
                                    duration: std::time::Duration::from_secs(t.duration),
                                    artwork,
                                };
                                player.write().play(source, meta);
                                current_song_title.set(t.title.clone());
                                current_song_artist.set(t.artist.clone());
                                current_song_duration.set(t.duration);
                                current_song_progress.set(0);
                                is_playing.set(true);

                                if let Some(album) = album_info {
                                    if let Some(url) =
                                        utils::format_artwork_url(album.cover_path.as_ref())
                                    {
                                        current_song_cover_url.set(url);
                                    } else {
                                        current_song_cover_url.set(String::new());
                                    }
                                } else {
                                    current_song_cover_url.set(String::new());
                                }
                            }
                        }
                    },
                    on_click_menu: None,
                    on_close_menu: None,
                    on_add_to_playlist: None,
                    on_delete_track: None,
                    actions: rsx! {
                        button {
                            class: "flex items-center gap-2 text-slate-400 hover:text-white transition-colors",
                            onclick: move |_| on_close.call(()),
                            i { class: "fa-solid fa-arrow-left" }
                            "Back"
                        }
                    }
                }
            }
        }
    }
}

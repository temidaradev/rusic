use config::AppConfig;
use dioxus::prelude::*;
use player::player::{NowPlayingMeta, Player};
use reader::{Library, Track};
use utils;

#[derive(Clone, Copy)]
pub struct PlayerController {
    pub player: Signal<Player>,
    pub is_playing: Signal<bool>,
    pub is_loading: Signal<bool>,
    pub queue: Signal<Vec<Track>>,
    pub current_queue_index: Signal<usize>,
    pub current_song_title: Signal<String>,
    pub current_song_artist: Signal<String>,
    pub current_song_album: Signal<String>,
    pub current_song_khz: Signal<u32>,
    pub current_song_bitrate: Signal<u8>,
    pub current_song_duration: Signal<u64>,
    pub current_song_progress: Signal<u64>,
    pub current_song_cover_url: Signal<String>,
    pub volume: Signal<f32>,
    pub library: Signal<Library>,
    pub config: Signal<AppConfig>,
    pub play_generation: Signal<usize>,
}

impl PlayerController {
    pub fn play_track(&mut self, idx: usize) {
        self.play_generation.with_mut(|g| *g += 1);
        let current_gen = *self.play_generation.peek();

        let q = self.queue.peek();
        if idx < q.len() {
            let track = q[idx].clone();
            let is_jellyfin = track.path.to_string_lossy().starts_with("jellyfin:");

            if is_jellyfin {
                let path_str = track.path.to_string_lossy();
                let parts: Vec<&str> = path_str.split(':').collect();
                let id = parts.get(1).unwrap_or(&"").to_string();

                let conf = self.config.read();
                if let Some(server) = &conf.server {
                    let mut stream_url = format!("{}/Audio/{}/stream?static=true", server.url, id);
                    if let Some(token) = &server.access_token {
                        stream_url.push_str(&format!("&api_key={}", token));
                    }

                    let mut cover_url = format!("{}/Items/{}/Images/Primary", server.url, id);
                    if let (Some(tag), Some(token)) = (parts.get(2), &server.access_token) {
                        cover_url.push_str(&format!("?tag={}&api_key={}", tag, token));
                    } else if let Some(token) = &server.access_token {
                        cover_url.push_str(&format!("?api_key={}", token));
                    }

                    self.player.write().stop();
                    self.is_playing.set(false);

                    let mut player = self.player;
                    let mut is_playing = self.is_playing;
                    let mut is_loading = self.is_loading;
                    let play_generation = self.play_generation;
                    let volume = self.volume;

                    self.current_song_title.set(track.title.clone());
                    self.current_song_artist.set(track.artist.clone());
                    self.current_song_album.set(track.album.clone());
                    self.current_song_duration.set(track.duration);
                    self.current_song_progress.set(0);
                    self.current_song_cover_url.set(cover_url.clone());
                    self.current_queue_index.set(idx);

                    self.is_loading.set(true);

                    spawn(async move {
                        let stream = utils::stream_buffer::StreamBuffer::new(stream_url);
                        let source_res =
                            tokio::task::spawn_blocking(move || rodio::Decoder::new(stream)).await;

                        if let Ok(Ok(source)) = source_res {
                            if *play_generation.read() == current_gen {
                                let meta = NowPlayingMeta {
                                    title: track.title.clone(),
                                    artist: track.artist.clone(),
                                    album: track.album.clone(),
                                    duration: std::time::Duration::from_secs(track.duration),
                                    artwork: Some(cover_url),
                                };

                                player.write().play(source, meta);
                                player.read().set_volume(*volume.peek());
                                is_loading.set(false);
                                is_playing.set(true);
                            }
                        } else {
                            is_loading.set(false);
                        }
                    });
                }
            } else {
                if let Ok(file) = std::fs::File::open(&track.path) {
                    if let Ok(source) = rodio::Decoder::new(std::io::BufReader::new(file)) {
                        let lib = self.library.peek();
                        let album = lib.albums.iter().find(|a| a.id == track.album_id);
                        let artwork = album.and_then(|a| {
                            a.cover_path
                                .as_ref()
                                .map(|p| p.to_string_lossy().into_owned())
                        });

                        let meta = NowPlayingMeta {
                            title: track.title.clone(),
                            artist: track.artist.clone(),
                            album: track.album.clone(),
                            duration: std::time::Duration::from_secs(track.duration),
                            artwork,
                        };

                        self.player.write().play(source, meta);
                        self.player.read().set_volume(*self.volume.peek());

                        self.current_song_title.set(track.title.clone());
                        self.current_song_artist.set(track.artist.clone());
                        self.current_song_album.set(track.album.clone());
                        self.current_song_khz.set(track.khz);
                        self.current_song_bitrate.set(track.bitrate);
                        self.current_song_duration.set(track.duration);
                        self.current_song_progress.set(0);

                        if let Some(album) = album {
                            if let Some(url) = utils::format_artwork_url(album.cover_path.as_ref())
                            {
                                self.current_song_cover_url.set(url);
                            } else {
                                self.current_song_cover_url.set(String::new());
                            }
                        } else {
                            self.current_song_cover_url.set(String::new());
                        }

                        self.current_queue_index.set(idx);
                        self.is_playing.set(true);
                    }
                }
            }
        }
    }

    pub fn play_next(&mut self) {
        let idx = *self.current_queue_index.peek();
        if idx + 1 < self.queue.peek().len() {
            self.play_track(idx + 1);
        } else {
            self.is_playing.set(false);
        }
    }

    pub fn play_prev(&mut self) {
        let idx = *self.current_queue_index.peek();
        if idx > 0 {
            self.play_track(idx - 1);
        }
    }

    pub fn pause(&mut self) {
        self.player.write().pause();
        self.is_playing.set(false);
    }

    pub fn resume(&mut self) {
        self.player.write().play_resume();
        self.is_playing.set(true);
    }

    pub fn toggle(&mut self) {
        if *self.is_playing.peek() {
            self.pause();
        } else {
            self.resume();
        }
    }
}

pub fn use_player_controller(
    player: Signal<Player>,
    is_playing: Signal<bool>,
    queue: Signal<Vec<Track>>,
    current_queue_index: Signal<usize>,
    current_song_title: Signal<String>,
    current_song_artist: Signal<String>,
    current_song_album: Signal<String>,
    current_song_khz: Signal<u32>,
    current_song_bitrate: Signal<u8>,
    current_song_duration: Signal<u64>,
    current_song_progress: Signal<u64>,
    current_song_cover_url: Signal<String>,
    volume: Signal<f32>,
    library: Signal<Library>,
    config: Signal<AppConfig>,
) -> PlayerController {
    let play_generation = use_signal(|| 0);
    let is_loading = use_signal(|| false);
    PlayerController {
        player,
        is_playing,
        is_loading,
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
        play_generation,
    }
}

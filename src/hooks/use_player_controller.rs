use crate::player::player::{NowPlayingMeta, Player};
use crate::reader::{Library, Track};
use crate::utils;
use dioxus::prelude::*;

#[derive(Clone, Copy)]
pub struct PlayerController {
    pub player: Signal<Player>,
    pub is_playing: Signal<bool>,
    pub queue: Signal<Vec<Track>>,
    pub current_queue_index: Signal<usize>,
    pub current_song_title: Signal<String>,
    pub current_song_artist: Signal<String>,
    pub current_song_duration: Signal<u64>,
    pub current_song_progress: Signal<u64>,
    pub current_song_cover_url: Signal<String>,
    pub volume: Signal<f32>,
    pub library: Signal<Library>,
}

impl PlayerController {
    pub fn play_track(&mut self, idx: usize) {
        let q = self.queue.peek();
        if idx < q.len() {
            let track = &q[idx];
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
                    self.current_song_duration.set(track.duration);
                    self.current_song_progress.set(0);

                    if let Some(album) = album {
                        if let Some(url) = utils::format_artwork_url(album.cover_path.as_ref()) {
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
    current_song_duration: Signal<u64>,
    current_song_progress: Signal<u64>,
    current_song_cover_url: Signal<String>,
    volume: Signal<f32>,
    library: Signal<Library>,
) -> PlayerController {
    PlayerController {
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
    }
}

use config::AppConfig;
use config::BackBehavior;
use config::MusicService;
use dioxus::{logger::tracing, prelude::*};
use player::player::{NowPlayingMeta, Player};
use reader::{Library, Track};
use scrobble;
use utils;
use std::time::Duration;

#[cfg(not(target_arch = "wasm32"))]
use player::decoder;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum LoopMode {
    None,
    Queue,
    Track,
}

impl LoopMode {
    pub fn next(&self) -> Self {
        match self {
            LoopMode::None => LoopMode::Queue,
            LoopMode::Queue => LoopMode::Track,
            LoopMode::Track => LoopMode::None,
        }
    }
}

#[derive(Clone, Copy)]
pub struct PlayerController {
    pub player: Signal<Player>,
    pub is_playing: Signal<bool>,
    pub is_loading: Signal<bool>,
    pub skip_in_progress: Signal<bool>,
    pub history: Signal<Vec<usize>>,
    pub queue: Signal<Vec<Track>>,
    pub shuffle: Signal<bool>,
    pub loop_mode: Signal<LoopMode>,
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
    pending_resume: Signal<Option<PendingResumeState>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PendingResumeState {
    track_path: String,
    progress_secs: u64,
}

impl PlayerController {
    fn track_key(track: &Track) -> String {
        track.path.to_string_lossy().to_string()
    }

    fn current_track(&self, idx: usize) -> Option<Track> {
        self.queue.peek().get(idx).cloned()
    }

    fn cover_url_for_track(&self, track: &Track) -> String {
        let path_str = Self::track_key(track);
        let scheme = path_str
            .split(':')
            .next()
            .unwrap_or_default()
            .to_ascii_lowercase();

        match scheme.as_str() {
            "jellyfin" => self
                .config
                .read()
                .server
                .as_ref()
                .and_then(|server| {
                    utils::jellyfin_image::jellyfin_image_url_from_path(
                        &path_str,
                        &server.url,
                        server.access_token.as_deref(),
                        800,
                        90,
                    )
                })
                .unwrap_or_default(),
            "subsonic" | "custom" => self
                .config
                .read()
                .server
                .as_ref()
                .and_then(|server| {
                    utils::subsonic_image::subsonic_image_url_from_path(
                        &path_str,
                        &server.url,
                        server.access_token.as_deref(),
                        800,
                        90,
                    )
                })
                .unwrap_or_default(),
            _ => self
                .library
                .read()
                .albums
                .iter()
                .find(|album| album.id == track.album_id)
                .and_then(|album| utils::format_artwork_url(album.cover_path.as_ref()))
                .unwrap_or_default(),
        }
    }

    fn clear_current_track_metadata(&mut self) {
        self.current_song_title.set(String::new());
        self.current_song_artist.set(String::new());
        self.current_song_album.set(String::new());
        self.current_song_khz.set(0);
        self.current_song_bitrate.set(0);
        self.current_song_duration.set(0);
        self.current_song_progress.set(0);
        self.current_song_cover_url.set(String::new());
    }

    fn hydrate_current_track_metadata(&mut self, idx: usize, progress_secs: u64) {
        if let Some(track) = self.current_track(idx) {
            let progress_secs = progress_secs.min(track.duration);
            self.current_queue_index.set(idx);
            self.current_song_title.set(track.title.clone());
            self.current_song_artist.set(track.artist.clone());
            self.current_song_album.set(track.album.clone());
            self.current_song_khz.set(track.khz);
            self.current_song_bitrate.set(track.bitrate);
            self.current_song_duration.set(track.duration);
            self.current_song_progress.set(progress_secs);
            self.current_song_cover_url.set(self.cover_url_for_track(&track));
        } else {
            self.current_queue_index.set(0);
            self.clear_current_track_metadata();
        }
    }

    fn take_pending_resume_seek(&mut self, track: &Track) -> Option<u64> {
        let pending = self.pending_resume.read().clone();
        self.pending_resume.set(None);

        pending.and_then(|pending| {
            if pending.track_path == Self::track_key(track) {
                Some(pending.progress_secs.min(track.duration))
            } else {
                None
            }
        })
    }

    fn apply_restore_seek(&mut self, seek_secs: u64) {
        self.player.write().seek(Duration::from_secs(seek_secs));
        self.current_song_progress.set(seek_secs);
    }

    /// Remap a queue index after moving one item within the queue.
    ///
    /// `index` is the position to remap, `from` is the original position of the moved item,
    /// and `to` is its destination after the move.
    ///
    /// Returns the new position for `index` after applying the move:
    /// - if `index == from`, this is the moved item itself, so it now lives at `to`
    /// - if the item moved forward (`from < to`), every item that was between `from + 1`
    ///   and `to` shifts left by one slot
    /// - if the item moved backward (`to < from`), every item that was between `to`
    ///   and `from - 1` shifts right by one slot
    /// - all other indices are unaffected
    fn remap_queue_index(index: usize, from: usize, to: usize) -> usize {
        if index == from {
            to
        } else if from < to && index > from && index <= to {
            index - 1
        } else if to < from && index >= to && index < from {
            index + 1
        } else {
            index
        }
    }

    pub fn play_track(&mut self, idx: usize) {
        let current_idx = *self.current_queue_index.peek();
        self.history.with_mut(|h| {
            if h.last() != Some(&current_idx) {
                h.push(current_idx);
            }
        });
        self.play_track_no_history(idx);
    }

    pub fn play_track_no_history(&mut self, idx: usize) {
        self.play_generation.with_mut(|g| *g += 1);
        let current_gen = *self.play_generation.peek();

        if let Some(track) = self.current_track(idx) {
            let path_str = track.path.to_string_lossy().to_string();
            let restore_seek_secs = self.take_pending_resume_seek(&track);
            let scheme = path_str
                .split(':')
                .next()
                .unwrap_or_default()
                .to_ascii_lowercase();
            let is_server_item = matches!(scheme.as_str(), "jellyfin" | "subsonic" | "custom");

            if is_server_item {
                let parts: Vec<&str> = path_str.split(':').collect();
                let id = parts.get(1).unwrap_or(&"").to_string();

                if let Some((stream_url, cover_url)) = {
                    let conf = self.config.read();
                    conf.server.as_ref().map(|server| match server.service {
                        MusicService::Jellyfin => {
                            let mut stream_url =
                                format!("{}/Audio/{}/stream?static=true", server.url, id);
                            if let Some(token) = &server.access_token {
                                stream_url.push_str(&format!("&api_key={}", token));
                            }

                            let cover_url = {
                                let path_str = track.path.to_string_lossy();
                                utils::jellyfin_image::jellyfin_image_url_from_path(
                                    &path_str,
                                    &server.url,
                                    server.access_token.as_deref(),
                                    800,
                                    90,
                                )
                                .unwrap_or_default()
                            };

                            (stream_url, cover_url)
                        }
                        MusicService::Subsonic | MusicService::Custom => {
                            let (stream_url, cover_url) = if let (Some(password), Some(username)) =
                                (&server.access_token, &server.user_id)
                            {
                                let remote = ::server::subsonic::SubsonicClient::new(
                                    &server.url,
                                    username,
                                    password,
                                );
                                let stream_url = remote.stream_url(&id).unwrap_or_default();
                                let cover_url =
                                    utils::subsonic_image::subsonic_image_url_from_path(
                                        &path_str,
                                        &server.url,
                                        server.access_token.as_deref(),
                                        800,
                                        90,
                                    )
                                    .or_else(|| remote.cover_art_url(&id, Some(800)).ok())
                                    .unwrap_or_default();
                                (stream_url, cover_url)
                            } else {
                                (String::new(), String::new())
                            };

                            (stream_url, cover_url)
                        }
                    })
                } {

                    if stream_url.is_empty() {
                        self.is_loading.set(false);
                        self.skip_in_progress.set(false);
                        return;
                    }

                    self.player.write().stop();
                    self.is_playing.set(false);

                    let mut player = self.player;
                    let mut is_playing = self.is_playing;
                    let mut is_loading = self.is_loading;
                    let mut skip_in_progress = self.skip_in_progress;
                    let play_generation = self.play_generation;
                    let volume = self.volume;
                    let mut current_song_progress = self.current_song_progress;
                    let cfg_signal = self.config;

                    self.hydrate_current_track_metadata(idx, 0);
                    self.current_song_cover_url.set(cover_url.clone());

                    self.is_loading.set(true);

                    #[cfg(not(target_arch = "wasm32"))]
                    spawn(async move {
                        let stream = utils::stream_buffer::StreamBuffer::new(stream_url);
                        let source_res = tokio::task::spawn_blocking(move || {
                            let (source, hint) = decoder::from_stream(stream);
                            Ok::<_, std::io::Error>((source, hint))
                        })
                        .await;

                        if let Ok(Ok((source, hint))) = source_res {
                            if *play_generation.read() == current_gen {
                                let meta = NowPlayingMeta {
                                    title: track.title.clone(),
                                    artist: track.artist.clone(),
                                    album: track.album.clone(),
                                    duration: std::time::Duration::from_secs(track.duration),
                                    artwork: Some(cover_url.clone()),
                                };

                                if let Err(e) = player.write().play(source, meta, hint) {
                                    eprintln!("Playback error: {e}");
                                    is_loading.set(false);
                                    skip_in_progress.set(false);
                                    return;
                                }
                                player.write().set_volume(*volume.peek());
                                if let Some(seek_secs) = restore_seek_secs {
                                    if seek_secs > 0 {
                                        player.write().seek(Duration::from_secs(seek_secs));
                                        current_song_progress.set(seek_secs);
                                    }
                                }
                                is_loading.set(false);
                                is_playing.set(true);
                                skip_in_progress.set(false);

                                let scrobble_track = track.clone();
                                let scrobble_gen = current_gen;
                                let scrobble_play_gen = play_generation;
                                let scrobble_cfg = cfg_signal;
                                let duration_secs = scrobble_track.duration;
                                let threshold_secs = std::cmp::min(240, (duration_secs / 2) as u64);

                                spawn(async move {
                                    let token_raw = scrobble_cfg.read().musicbrainz_token.clone();
                                    if !token_raw.is_empty() {
                                        let auth = if token_raw.contains(' ') {
                                            token_raw.clone()
                                        } else {
                                            format!("Token {}", token_raw)
                                        };

                                        let playing_now = scrobble::musicbrainz::make_playing_now(
                                            &scrobble_track.artist,
                                            &scrobble_track.title,
                                            Some(&scrobble_track.album),
                                        );

                                        if let Err(e) = scrobble::musicbrainz::submit_listens(
                                            &auth,
                                            vec![playing_now],
                                            "playing_now",
                                        )
                                        .await
                                        {
                                            tracing::warn!(
                                                "Jellyfin: failed to submit playing_now: {}",
                                                e
                                            );
                                        }
                                    }

                                    tokio::time::sleep(std::time::Duration::from_secs(
                                        threshold_secs,
                                    ))
                                    .await;

                                    if *scrobble_play_gen.read() != scrobble_gen {
                                        return;
                                    }

                                    let token_raw = scrobble_cfg.read().musicbrainz_token.clone();
                                    if token_raw.is_empty() {
                                        return;
                                    }

                                    let auth = if token_raw.contains(' ') {
                                        token_raw
                                    } else {
                                        format!("Token {}", token_raw)
                                    };

                                    let listen = scrobble::musicbrainz::make_listen(
                                        &scrobble_track.artist,
                                        &scrobble_track.title,
                                        Some(&scrobble_track.album),
                                    );

                                    match scrobble::musicbrainz::submit_listens(
                                        &auth,
                                        vec![listen],
                                        "single",
                                    )
                                    .await
                                    {
                                        Ok(_) => tracing::info!(
                                            "Jellyfin scrobbled: {} - {}",
                                            scrobble_track.artist,
                                            scrobble_track.title
                                        ),
                                        Err(e) => tracing::warn!("Jellyfin scrobble failed: {}", e),
                                    }
                                });

                                let cover_url = cover_url.clone();
                                let track = track.clone();
                                let mut player = player;
                                let play_generation = play_generation;

                                spawn(async move {
                                    if let Ok(response) = reqwest::get(&cover_url).await {
                                        if let Ok(bytes) = response.bytes().await {
                                            let temp_dir = std::env::temp_dir();
                                            let random_id: u64 = rand::random();
                                            let file_path = temp_dir
                                                .join(format!("kopuz_cover_{}.jpg", random_id));

                                            if tokio::fs::write(&file_path, bytes).await.is_ok() {
                                                if *play_generation.read() == current_gen {
                                                    let path_str =
                                                        file_path.to_string_lossy().to_string();
                                                    let new_meta = NowPlayingMeta {
                                                        title: track.title,
                                                        artist: track.artist,
                                                        album: track.album,
                                                        duration: std::time::Duration::from_secs(
                                                            track.duration,
                                                        ),
                                                        artwork: Some(path_str),
                                                    };
                                                    player.write().update_metadata(new_meta);
                                                }
                                            }
                                        }
                                    }
                                });
                            }
                        } else {
                            is_loading.set(false);
                            skip_in_progress.set(false);
                        }
                    });

                    #[cfg(target_arch = "wasm32")]
                    spawn(async move {
                        if *play_generation.read() == current_gen {
                            let meta = NowPlayingMeta {
                                title: track.title.clone(),
                                artist: track.artist.clone(),
                                album: track.album.clone(),
                                duration: std::time::Duration::from_secs(track.duration),
                                artwork: Some(cover_url.clone()),
                            };

                            player.write().play_url(stream_url, meta);
                            player.write().set_volume(*volume.peek());
                            if let Some(seek_secs) = restore_seek_secs {
                                if seek_secs > 0 {
                                    player.write().seek(Duration::from_secs(seek_secs));
                                    current_song_progress.set(seek_secs);
                                }
                            }
                            is_loading.set(false);
                            is_playing.set(true);
                            skip_in_progress.set(false);

                            let scrobble_track = track.clone();
                            let scrobble_gen = current_gen;
                            let scrobble_play_gen = play_generation;
                            let scrobble_cfg = cfg_signal;
                            let duration_secs = scrobble_track.duration;
                            let threshold_secs = std::cmp::min(240, (duration_secs / 2) as u64);

                            spawn(async move {
                                let token_raw = scrobble_cfg.read().musicbrainz_token.clone();
                                if !token_raw.is_empty() {
                                    let auth = if token_raw.contains(' ') {
                                        token_raw.clone()
                                    } else {
                                        format!("Token {}", token_raw)
                                    };

                                    let playing_now = scrobble::musicbrainz::make_playing_now(
                                        &scrobble_track.artist,
                                        &scrobble_track.title,
                                        Some(&scrobble_track.album),
                                    );

                                    if let Err(e) = scrobble::musicbrainz::submit_listens(
                                        &auth,
                                        vec![playing_now],
                                        "playing_now",
                                    )
                                    .await
                                    {
                                        tracing::warn!(
                                            "Jellyfin: failed to submit playing_now: {}",
                                            e
                                        );
                                    }
                                }

                                utils::sleep(std::time::Duration::from_secs(threshold_secs)).await;

                                if *scrobble_play_gen.read() != scrobble_gen {
                                    return;
                                }

                                let token_raw = scrobble_cfg.read().musicbrainz_token.clone();
                                if token_raw.is_empty() {
                                    return;
                                }

                                let auth = if token_raw.contains(' ') {
                                    token_raw
                                } else {
                                    format!("Token {}", token_raw)
                                };

                                let listen = scrobble::musicbrainz::make_listen(
                                    &scrobble_track.artist,
                                    &scrobble_track.title,
                                    Some(&scrobble_track.album),
                                );

                                match scrobble::musicbrainz::submit_listens(
                                    &auth,
                                    vec![listen],
                                    "single",
                                )
                                .await
                                {
                                    Ok(_) => tracing::info!(
                                        "Jellyfin scrobbled: {} - {}",
                                        scrobble_track.artist,
                                        scrobble_track.title
                                    ),
                                    Err(e) => tracing::warn!("Jellyfin scrobble failed: {}", e),
                                }
                            });
                        } else {
                            is_loading.set(false);
                            skip_in_progress.set(false);
                        }
                    });
                }
            } else {
                #[cfg(not(target_arch = "wasm32"))]
                self.current_queue_index.set(idx);
                #[cfg(target_arch = "wasm32")]
                {
                    let _ = idx;
                    return;
                } // local files not supported on web
                #[cfg(not(target_arch = "wasm32"))]
                if let Ok((source, hint)) = decoder::open_file(&track.path) {
                    {
                        let artwork = {
                            let lib = self.library.peek();
                            lib.albums
                                .iter()
                                .find(|a| a.id == track.album_id)
                                .and_then(|a| {
                                    a.cover_path
                                        .as_ref()
                                        .map(|p| p.to_string_lossy().into_owned())
                                })
                        };

                        let meta = NowPlayingMeta {
                            title: track.title.clone(),
                            artist: track.artist.clone(),
                            album: track.album.clone(),
                            duration: std::time::Duration::from_secs(track.duration),
                            artwork,
                        };

                        if let Err(e) = self.player.write().play(source, meta, hint) {
                            eprintln!("Playback error: {e}");
                            self.skip_in_progress.set(false);
                            return;
                        }
                        self.player.write().set_volume(*self.volume.peek());

                        self.skip_in_progress.set(false);

                        self.hydrate_current_track_metadata(idx, 0);

                        self.is_playing.set(true);

                        if let Some(seek_secs) = restore_seek_secs {
                            if seek_secs > 0 {
                                self.apply_restore_seek(seek_secs);
                            }
                        }

                        let cfg_signal = self.config;
                        let play_generation_signal = self.play_generation;
                        let gen_snapshot = current_gen;
                        let scrobble_track = track.clone();

                        let duration_secs = scrobble_track.duration;
                        let threshold_secs = std::cmp::min(240, (duration_secs / 2) as u64);

                        spawn(async move {
                            let token_raw = cfg_signal.read().musicbrainz_token.clone();
                            if !token_raw.is_empty() {
                                let auth_header_value = if token_raw.contains(' ') {
                                    token_raw
                                } else {
                                    format!("Token {}", token_raw)
                                };

                                let playing_now = scrobble::musicbrainz::make_playing_now(
                                    &scrobble_track.artist,
                                    &scrobble_track.title,
                                    Some(&scrobble_track.album),
                                );

                                if let Err(e) = scrobble::musicbrainz::submit_listens(
                                    &auth_header_value,
                                    vec![playing_now],
                                    "playing_now",
                                )
                                .await
                                {
                                    tracing::warn!("Failed to submit playing_now: {}", e);
                                }
                            }

                            tokio::time::sleep(std::time::Duration::from_secs(threshold_secs))
                                .await;
                            if *play_generation_signal.read() != gen_snapshot {
                                return;
                            }

                            let token_raw = cfg_signal.read().musicbrainz_token.clone();
                            if token_raw.is_empty() {
                                return;
                            }

                            let auth_header_value = if token_raw.contains(' ') {
                                token_raw
                            } else {
                                format!("Token {}", token_raw)
                            };

                            let listen = scrobble::musicbrainz::make_listen(
                                &scrobble_track.artist,
                                &scrobble_track.title,
                                Some(&scrobble_track.album),
                            );

                            match scrobble::musicbrainz::submit_listens(
                                &auth_header_value,
                                vec![listen],
                                "single",
                            )
                            .await
                            {
                                Ok(_) => tracing::info!(
                                    "Scrobbled: {} - {}",
                                    scrobble_track.artist,
                                    scrobble_track.title
                                ),
                                Err(e) => tracing::warn!("Scrobble failed: {}", e),
                            }
                        });
                    }
                }
            }
        }
    }

    pub fn play_next(&mut self) {
        let idx = *self.current_queue_index.peek();
        let queue_len = self.queue.peek().len();

        if queue_len == 0 {
            return;
        }

        let loop_mode = *self.loop_mode.peek();
        let shuffle = *self.shuffle.peek();

        match loop_mode {
            LoopMode::Track => {
                self.play_track(idx);
            }
            _ => {
                if shuffle && queue_len > 1 {
                    let mut rng = rand::thread_rng();
                    use rand::Rng;
                    let mut next_idx = rng.gen_range(0..queue_len);
                    while next_idx == idx {
                        next_idx = rng.gen_range(0..queue_len);
                    }
                    self.play_track(next_idx);
                } else if shuffle && queue_len == 1 {
                    self.play_track(0);
                } else if idx + 1 < queue_len {
                    self.play_track(idx + 1);
                } else if loop_mode == LoopMode::Queue {
                    self.play_track(0);
                } else {
                    self.is_playing.set(false);
                }
            }
        }
    }

    pub fn play_prev(&mut self) {
        let progress = *self.current_song_progress.peek();
        let back_behavior = self.config.peek().back_behavior;

        if back_behavior == BackBehavior::RewindThenPrev && progress > 3 {
            self.player
                .write()
                .seek(std::time::Duration::ZERO);
            self.current_song_progress.set(0);
            return;
        }

        let idx = *self.current_queue_index.peek();
        let queue_len = self.queue.peek().len();

        if queue_len == 0 {
            return;
        }

        if let Some(prev_idx) = self.history.with_mut(|h| h.pop()) {
            self.play_track_no_history(prev_idx);
            return;
        }

        if idx > 0 {
            self.play_track_no_history(idx - 1);
        } else if *self.loop_mode.peek() == LoopMode::Queue {
            self.play_track_no_history(queue_len - 1);
        }
    }

    pub fn toggle_shuffle(&mut self) {
        self.shuffle.with_mut(|s| *s = !*s);
    }

    pub fn toggle_loop(&mut self) {
        self.loop_mode.with_mut(|l| *l = l.next());
    }

    pub fn pause(&mut self) {
        self.player.write().pause();
        self.is_playing.set(false);
    }

    pub fn resume(&mut self) {
        if !self.player.peek().can_resume() {
            let idx = *self.current_queue_index.peek();
            if let Some(track) = self.current_track(idx) {
                let pending_resume = self.pending_resume.read().clone();
                if let Some(mut pending) = pending_resume {
                    if pending.track_path == Self::track_key(&track) {
                        pending.progress_secs =
                            (*self.current_song_progress.peek()).min(track.duration);
                        self.pending_resume.set(Some(pending));
                    }
                }
                self.play_track_no_history(idx);
            }
            return;
        }

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

    pub fn move_queue_item(&mut self, from: usize, to: usize) {
        let len = self.queue.peek().len();
        if from >= len || to >= len || from == to {
            return;
        }

        self.queue.with_mut(|queue| {
            let track = queue.remove(from);
            queue.insert(to, track);
        });

        let current_idx = *self.current_queue_index.peek();
        self.current_queue_index
            .set(Self::remap_queue_index(current_idx, from, to));

        self.history.with_mut(|history| {
            for idx in history.iter_mut() {
                *idx = Self::remap_queue_index(*idx, from, to);
            }
        });
    }

    pub fn restore_queue_state(
        &mut self,
        queue: Vec<Track>,
        current_queue_index: usize,
        progress_secs: u64,
    ) {
        self.player.write().stop();
        self.is_playing.set(false);
        self.is_loading.set(false);
        self.skip_in_progress.set(false);
        self.history.set(Vec::new());
        self.queue.set(queue);

        let queue_len = self.queue.peek().len();
        if queue_len == 0 {
            self.current_queue_index.set(0);
            self.pending_resume.set(None);
            self.clear_current_track_metadata();
            return;
        }

        let idx = current_queue_index.min(queue_len - 1);
        let track = self.current_track(idx).expect("queue index should be valid");
        let progress_secs = progress_secs.min(track.duration);

        self.hydrate_current_track_metadata(idx, progress_secs);
        self.pending_resume.set(if progress_secs > 0 {
            Some(PendingResumeState {
                track_path: Self::track_key(&track),
                progress_secs,
            })
        } else {
            None
        });
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
    let skip_in_progress = use_signal(|| false);
    let history = use_signal(|| Vec::new());
    let shuffle = use_signal(|| false);
    let loop_mode = use_signal(|| LoopMode::None);
    let pending_resume = use_signal(|| None::<PendingResumeState>);

    PlayerController {
        player,
        is_playing,
        is_loading,
        skip_in_progress,
        history,
        queue,
        shuffle,
        loop_mode,
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
        pending_resume,
    }
}

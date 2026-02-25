use crate::use_player_controller::PlayerController;
use config::AppConfig;
use dioxus::prelude::*;
use discord_presence::Presence;
use server::jellyfin::JellyfinRemote;
use std::sync::Arc;

pub fn use_player_task(ctrl: PlayerController) {
    let presence: Option<Arc<Presence>> = use_context();
    let mut config: Signal<AppConfig> = use_context();
    let mut last_title = use_signal(String::new);
    let mut was_playing = use_signal(|| false);

    use_future(move || {
        let mut ctrl = ctrl;
        async move {
            #[cfg(target_os = "macos")]
            {
                use player::systemint::{SystemEvent, wait_event};
                println!("[player_task] Starting macOS event loop");
                loop {
                    let event = wait_event().await;
                    if let Some(event) = event {
                        println!("[player_task] Received MacOS system event: {:?}", event);
                        match event {
                            SystemEvent::Play => ctrl.resume(),
                            SystemEvent::Pause => ctrl.pause(),
                            SystemEvent::Toggle => ctrl.toggle(),
                            SystemEvent::Next => ctrl.play_next(),
                            SystemEvent::Prev => ctrl.play_prev(),
                        }
                    } else {
                        println!("[player_task] wait_event returned None - channel closed?");
                        break;
                    }
                }
            }

            #[cfg(target_os = "linux")]
            {
                use player::systemint::{SystemEvent, poll_event};
                loop {
                    let mut processed = false;
                    while let Some(event) = poll_event() {
                        processed = true;
                        match event {
                            SystemEvent::Play => ctrl.resume(),
                            SystemEvent::Pause => ctrl.pause(),
                            SystemEvent::Toggle => ctrl.toggle(),
                            SystemEvent::Next => ctrl.play_next(),
                            SystemEvent::Prev => ctrl.play_prev(),
                        }
                    }
                    if !processed {
                        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                    }
                }
            }

            #[cfg(not(any(target_os = "macos", target_os = "linux")))]
            {
                std::future::pending::<()>().await;
            }
        }
    });

    use_future(move || {
        let mut ctrl = ctrl;
        let presence = presence.clone();
        let mut last_discord_enabled = false;
        let mut last_jellyfin_id: Option<String> = None;
        let mut last_ping = std::time::Instant::now();
        let mut last_progress_report = std::time::Instant::now();

        async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;

                let is_playing = *ctrl.is_playing.read();
                let discord_enabled = config.read().discord_presence.unwrap_or(true);
                let pos = ctrl.player.read().get_position();

                let jellyfin_info = {
                    let conf = config.read();
                    conf.server.clone().map(|s| (s, conf.device_id.clone()))
                };

                if let Some((server, device_id)) = jellyfin_info {
                    let remote = JellyfinRemote::new(
                        &server.url,
                        server.access_token.as_deref(),
                        &device_id,
                        server.user_id.as_deref(),
                    );

                    if last_ping.elapsed().as_secs() >= 30 {
                        let _ = remote.ping().await;
                        last_ping = std::time::Instant::now();
                    }

                    let track = {
                        let q = ctrl.queue.read();
                        let idx = *ctrl.current_queue_index.read();
                        q.get(idx).cloned()
                    };

                    if let Some(track) = track {
                        let path_str = track.path.to_string_lossy();
                        if path_str.starts_with("jellyfin:") {
                            let parts: Vec<&str> = path_str.split(':').collect();
                            if let Some(id) = parts.get(1) {
                                let current_id = id.to_string();

                                if last_jellyfin_id.as_ref() != Some(&current_id) {
                                    if let Some(old_id) = last_jellyfin_id {
                                        let _ = remote
                                            .report_playback_stopped(
                                                &old_id,
                                                pos.as_micros() as u64 * 10,
                                            )
                                            .await;
                                    }
                                    let _ = remote.report_playback_start(&current_id).await;
                                    last_jellyfin_id = Some(current_id.clone());
                                }

                                if last_progress_report.elapsed().as_secs() >= 5
                                    || is_playing != *was_playing.peek()
                                {
                                    let ticks = pos.as_micros() as u64 * 10;
                                    let _ = remote
                                        .report_playback_progress(&current_id, ticks, !is_playing)
                                        .await;
                                    last_progress_report = std::time::Instant::now();
                                }
                            }
                        } else if let Some(old_id) = last_jellyfin_id.take() {
                            let _ = remote
                                .report_playback_stopped(&old_id, pos.as_micros() as u64 * 10)
                                .await;
                        }
                    } else if let Some(old_id) = last_jellyfin_id.take() {
                        let _ = remote
                            .report_playback_stopped(&old_id, pos.as_micros() as u64 * 10)
                            .await;
                    }
                }

                if is_playing {
                    ctrl.current_song_progress.set(pos.as_secs());

                    if let Some(ref p) = presence {
                        let title = ctrl.current_song_title.read().clone();
                        let artist = ctrl.current_song_artist.read().clone();
                        let album = ctrl.current_song_album.read().clone();
                        let duration = *ctrl.current_song_duration.read();
                        let progress = pos.as_secs();
                        let cover = ctrl.current_song_cover_url.read().clone();

                        if discord_enabled {
                            if title != *last_title.peek()
                                || !*was_playing.peek()
                                || !last_discord_enabled
                            {
                                last_title.set(title.clone());
                                println!("Cover URL: {}", cover);
                                let cover_ref = if cover.starts_with("http") {
                                    Some(cover.as_str())
                                } else {
                                    None
                                };
                                let _ = p.set_now_playing(
                                    &title, &artist, &album, progress, duration, cover_ref,
                                );
                            }
                        } else if last_discord_enabled {
                            let _ = p.clear_activity();
                        }
                    }

                    let duration = *ctrl.current_song_duration.read();
                    if (ctrl.player.read().is_empty()
                        || (duration > 0 && pos.as_secs() >= duration))
                        && !*ctrl.is_loading.read()
                    {
                        {
                            let mut config_write = config.write();
                            let q = ctrl.queue.peek();
                            let idx = *ctrl.current_queue_index.peek();
                            if let Some(track) = q.get(idx) {
                                let track_id = track.path.to_string_lossy().to_string();
                                *config_write.listen_counts.entry(track_id).or_insert(0) += 1;
                            }
                        }
                        ctrl.play_next();
                    }
                } else if *was_playing.peek() {
                    if let Some(ref p) = presence {
                        let title = ctrl.current_song_title.read().clone();
                        let artist = ctrl.current_song_artist.read().clone();
                        if discord_enabled {
                            let _ = p.set_paused(&title, &artist);
                        } else if last_discord_enabled {
                            let _ = p.clear_activity();
                        }
                    }
                } else if let Some(ref p) = presence {
                    if !discord_enabled && last_discord_enabled {
                        let _ = p.clear_activity();
                    } else if discord_enabled && !last_discord_enabled {
                        let title = ctrl.current_song_title.read().clone();
                        if !title.is_empty() {
                            let artist = ctrl.current_song_artist.read().clone();
                            let _ = p.set_paused(&title, &artist);
                        }
                    }
                }

                was_playing.set(is_playing);
                last_discord_enabled = discord_enabled;
            }
        }
    });
}

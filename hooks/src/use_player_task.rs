use crate::use_player_controller::PlayerController;
use config::AppConfig;
use dioxus::prelude::*;
use discord_presence::Presence;
use std::sync::Arc;

pub fn use_player_task(ctrl: PlayerController) {
    let presence: Option<Arc<Presence>> = use_context();
    let config: Signal<AppConfig> = use_context();
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

        async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;

                let is_playing = *ctrl.is_playing.read();
                let discord_enabled = config.read().discord_presence.unwrap_or(true);

                if is_playing {
                    let pos = ctrl.player.read().get_position();
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

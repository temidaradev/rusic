use crate::hooks::use_player_controller::PlayerController;
use dioxus::prelude::*;
use gloo_timers::future::TimeoutFuture;

pub fn use_player_task(ctrl: PlayerController) {
    use_future(move || {
        let mut ctrl = ctrl;
        async move {
            loop {
                TimeoutFuture::new(250).await;

                if *ctrl.is_playing.read() {
                    let pos = ctrl.player.read().get_position();
                    ctrl.current_song_progress.set(pos.as_secs());

                    let current_duration = *ctrl.current_song_duration.read();
                    if current_duration == 0 {
                        let audio_duration = ctrl.player.read().get_duration();
                        if audio_duration.as_secs() > 0 {
                            ctrl.current_song_duration.set(audio_duration.as_secs());
                        }
                    }

                    if ctrl.player.read().is_empty() {
                        ctrl.play_next();
                    }
                }
            }
        }
    });
}

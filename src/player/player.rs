use std::time::Duration;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[derive(Clone)]
pub struct NowPlayingMeta {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration: Duration,
    pub artwork: Option<String>,
}

pub struct Player {
    audio: web_sys::HtmlAudioElement,
    now_playing: Option<NowPlayingMeta>,
}

impl Player {
    pub fn new() -> Self {
        let audio = web_sys::HtmlAudioElement::new().expect("create audio element");
        Self {
            audio,
            now_playing: None,
        }
    }

    pub fn play_url(&mut self, url: &str, meta: NowPlayingMeta) {
        self.audio.set_src(url);
        let _ = self.audio.play();
        self.now_playing = Some(meta.clone());
        self.update_media_session(&meta);
    }

    pub fn play_blob(&mut self, blob: &web_sys::Blob, meta: NowPlayingMeta) {
        let url = web_sys::Url::create_object_url_with_blob(blob).unwrap_or_default();
        self.audio.set_src(&url);
        let _ = self.audio.play();
        self.now_playing = Some(meta.clone());
        self.update_media_session(&meta);
    }

    pub fn pause(&mut self) {
        let _ = self.audio.pause();
        self.update_media_session_playback_state(false);
    }

    pub fn play_resume(&mut self) {
        let _ = self.audio.play();
        self.update_media_session_playback_state(true);
    }

    pub fn seek(&mut self, time: Duration) {
        self.audio.set_current_time(time.as_secs_f64());
    }

    pub fn is_empty(&self) -> bool {
        self.audio.ended() || self.audio.src().is_empty()
    }

    pub fn is_paused(&self) -> bool {
        self.audio.paused()
    }

    pub fn set_volume(&self, volume: f32) {
        self.audio.set_volume(volume as f64);
    }

    pub fn get_position(&self) -> Duration {
        Duration::from_secs_f64(self.audio.current_time())
    }

    pub fn get_duration(&self) -> Duration {
        let dur = self.audio.duration();
        if dur.is_nan() || dur.is_infinite() {
            Duration::from_secs(0)
        } else {
            Duration::from_secs_f64(dur)
        }
    }

    pub fn audio_element(&self) -> &web_sys::HtmlAudioElement {
        &self.audio
    }

    fn update_media_session(&self, meta: &NowPlayingMeta) {
        if let Some(window) = web_sys::window() {
            if let Ok(navigator) = js_sys::Reflect::get(&window, &JsValue::from_str("navigator")) {
                if let Ok(media_session) =
                    js_sys::Reflect::get(&navigator, &JsValue::from_str("mediaSession"))
                {
                    if !media_session.is_undefined() {
                        let init = js_sys::Object::new();
                        let _ = js_sys::Reflect::set(
                            &init,
                            &JsValue::from_str("title"),
                            &JsValue::from_str(&meta.title),
                        );
                        let _ = js_sys::Reflect::set(
                            &init,
                            &JsValue::from_str("artist"),
                            &JsValue::from_str(&meta.artist),
                        );
                        let _ = js_sys::Reflect::set(
                            &init,
                            &JsValue::from_str("album"),
                            &JsValue::from_str(&meta.album),
                        );

                        if let Ok(media_metadata_ctor) =
                            js_sys::Reflect::get(&window, &JsValue::from_str("MediaMetadata"))
                        {
                            if let Ok(ctor) = media_metadata_ctor.dyn_into::<js_sys::Function>() {
                                if let Ok(metadata) =
                                    js_sys::Reflect::construct(&ctor, &js_sys::Array::of1(&init))
                                {
                                    let _ = js_sys::Reflect::set(
                                        &media_session,
                                        &JsValue::from_str("metadata"),
                                        &metadata,
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn update_media_session_playback_state(&self, playing: bool) {
        if let Some(window) = web_sys::window() {
            if let Ok(navigator) = js_sys::Reflect::get(&window, &JsValue::from_str("navigator")) {
                if let Ok(media_session) =
                    js_sys::Reflect::get(&navigator, &JsValue::from_str("mediaSession"))
                {
                    if !media_session.is_undefined() {
                        let state = if playing { "playing" } else { "paused" };
                        let _ = js_sys::Reflect::set(
                            &media_session,
                            &JsValue::from_str("playbackState"),
                            &JsValue::from_str(state),
                        );
                    }
                }
            }
        }
    }
}

impl Default for Player {
    fn default() -> Self {
        Self::new()
    }
}

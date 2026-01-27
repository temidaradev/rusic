#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "macos")]
pub use macos::{poll_event, update_now_playing, SystemEvent};

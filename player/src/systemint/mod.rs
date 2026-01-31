#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "macos")]
pub use macos::{SystemEvent, poll_event, update_now_playing};

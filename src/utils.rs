use crate::web_audio_store;
use std::path::Path;

pub fn format_artwork_url(path: Option<&impl AsRef<Path>>) -> Option<String> {
    path.and_then(|p| {
        let path_key = p.as_ref().to_string_lossy().to_string();

        if path_key.starts_with("blob:") {
            return Some(path_key);
        }

        web_audio_store::get_blob_url(&path_key)
    })
}

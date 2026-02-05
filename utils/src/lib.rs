pub mod stream_buffer;

use std::path::Path;

pub fn format_artwork_url(path: Option<&impl AsRef<Path>>) -> Option<String> {
    const FRAGMENT: &percent_encoding::AsciiSet = &percent_encoding::CONTROLS
        .add(b' ')
        .add(b'"')
        .add(b'<')
        .add(b'>')
        .add(b'`');

    path.map(|p| {
        let p = p.as_ref();
        let p_str = p.to_string_lossy();
        // why changed to strip_prefix -> Using str:strip_{prefix,suffix} is safer and may have better performance as there is no slicing which may panic
        // and the compiler does not need to insert this panic code.
        // It is also sometimes more readable as it removes the need for duplicating or storing the pattern used by str::{starts,ends}_with and in the slicing.
        let abs_path = if let Some(path) = p_str.strip_prefix("./") {
            std::env::current_dir().unwrap_or_default().join(path)
        } else {
            p.to_path_buf()
        };

        format!(
            "artwork://local{}",
            percent_encoding::utf8_percent_encode(&abs_path.to_string_lossy(), FRAGMENT)
        )
    })
}

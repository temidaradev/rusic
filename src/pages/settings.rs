use crate::config::AppConfig;
use crate::reader::{Album, Library, Track};
use crate::web_audio_store;
use dioxus::prelude::*;
use std::collections::HashMap;
use std::path::PathBuf;

use js_sys::Reflect;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{FileList, HtmlInputElement};

#[component]
pub fn Settings(config: Signal<AppConfig>, library: Signal<Library>) -> Element {
    rsx! {
        div {
            class: "p-8 max-w-4xl",
            h1 { class: "text-3xl font-bold text-white mb-6", "Settings" }

            div {
                class: "space-y-8",

                section {
                    h2 { class: "text-lg font-semibold text-white/80 mb-4 border-b border-white/5 pb-2", "General" }
                    div { class: "space-y-4",
                        SettingItem {
                            title: "Appearance",
                            description: "Select your preferred color theme.".to_string(),
                            control: rsx! {
                                select {
                                    class: "bg-white/5 border border-white/10 rounded px-3 py-1 text-sm text-white focus:outline-none focus:border-white/20",
                                    value: "{config.read().theme}",
                                    onchange: move |evt| {
                                        config.write().theme = evt.value();
                                    },
                                    option { value: "default", "Default" }
                                    option { value: "gruvbox", "Gruvbox Material" }
                                    option { value: "dracula", "Dracula" }
                                    option { value: "nord", "Nord" }
                                    option { value: "catppuccin", "Catppuccin Mocha" }
                                }
                            }
                        }
                        MusicDirectorySetting { config, library }
                    }
                }

                section {
                    h2 { class: "text-lg font-semibold text-white/80 mb-4 border-b border-white/5 pb-2", "Library Stats" }
                    div { class: "space-y-2",
                        p { class: "text-slate-400",
                            "Tracks: "
                            span { class: "text-white font-medium", "{library.read().tracks.len()}" }
                        }
                        p { class: "text-slate-400",
                            "Albums: "
                            span { class: "text-white font-medium", "{library.read().albums.len()}" }
                        }
                        p { class: "text-slate-400",
                            "Files in memory: "
                            span { class: "text-white font-medium", "{web_audio_store::file_count()}" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn MusicDirectorySetting(config: Signal<AppConfig>, library: Signal<Library>) -> Element {
    let mut is_loading = use_signal(|| false);
    let mut files_count = use_signal(|| 0usize);
    let mut error_msg = use_signal(|| Option::<String>::None);

    let last_folder = config.read().last_folder_name.clone();
    let files_in_memory = web_audio_store::file_count();

    rsx! {
        SettingItem {
            title: "Music Folder",
            description: if *is_loading.read() {
                "Scanning folder...".to_string()
            } else if *files_count.read() > 0 {
                format!("Added {} audio files to library", *files_count.read())
            } else if let Some(folder) = &last_folder {
                if files_in_memory > 0 {
                    format!("Loaded: {} ({} files)", folder, files_in_memory)
                } else {
                    format!("Last folder: {} (needs reload)", folder)
                }
            } else {
                "Select a folder containing your music files.".to_string()
            },
            control: rsx! {
                div {
                    class: "flex flex-col items-end gap-2",
                    div {
                        class: "relative",
                        input {
                            r#type: "file",
                            "webkitdirectory": "true",
                            "directory": "true",
                            class: "absolute inset-0 w-full h-full opacity-0 cursor-pointer",
                            id: "folder-input",
                            onchange: move |_evt| {
                                is_loading.set(true);
                                error_msg.set(None);
                                files_count.set(0);

                                spawn(async move {
                                    match process_folder_selection(config, library).await {
                                        Ok(count) => {
                                            files_count.set(count);
                                            is_loading.set(false);
                                        }
                                        Err(e) => {
                                            error_msg.set(Some(e));
                                            is_loading.set(false);
                                        }
                                    }
                                });
                            }
                        }
                        button {
                            class: if *is_loading.read() {
                                "bg-white/5 px-3 py-1 rounded text-sm text-slate-500 pointer-events-none"
                            } else {
                                "bg-white/10 hover:bg-white/20 px-3 py-1 rounded text-sm text-white transition-colors pointer-events-none"
                            },
                            if *is_loading.read() {
                                i { class: "fa-solid fa-spinner fa-spin mr-2" }
                                "Scanning..."
                            } else {
                                i { class: "fa-solid fa-folder-open mr-2" }
                                "Select Folder"
                            }
                        }
                    }
                    if let Some(err) = error_msg.read().as_ref() {
                        p { class: "text-red-400 text-xs", "{err}" }
                    }
                }
            }
        }
    }
}

async fn process_folder_selection(
    mut config: Signal<AppConfig>,
    mut library: Signal<Library>,
) -> Result<usize, String> {
    let window = web_sys::window().ok_or("No window")?;
    let document = window.document().ok_or("No document")?;

    let input: HtmlInputElement = document
        .get_element_by_id("folder-input")
        .ok_or("No input element")?
        .dyn_into()
        .map_err(|_| "Not an input element")?;

    let files: FileList = input.files().ok_or("No files selected")?;
    let file_count = files.length();

    if file_count == 0 {
        return Err("No files found in folder".to_string());
    }

    let mut added_count = 0;

    let mut folder_name: Option<String> = None;
    if let Some(first_file) = files.get(0) {
        if let Ok(path) = Reflect::get(&first_file, &JsValue::from_str("webkitRelativePath")) {
            if let Some(path_str) = path.as_string() {
                if let Some(first_part) = path_str.split('/').next() {
                    folder_name = Some(first_part.to_string());
                }
            }
        }
    }

    let mut albums_map: HashMap<String, Album> = HashMap::new();

    let mut folder_covers: HashMap<String, String> = HashMap::new();

    for i in 0..file_count {
        if let Some(file) = files.get(i) {
            let name = file.name().to_lowercase();
            let file_type = file.type_();

            let is_cover_image = file_type.starts_with("image/")
                && (name.contains("cover")
                    || name.contains("folder")
                    || name.contains("album")
                    || name.contains("front")
                    || name == "artwork.jpg"
                    || name == "artwork.png"
                    || name == "art.jpg"
                    || name == "art.png"
                    || name.ends_with(".jpg")
                    || name.ends_with(".jpeg")
                    || name.ends_with(".png")
                    || name.ends_with(".webp"));

            if is_cover_image {
                let relative_path: String =
                    Reflect::get(&file, &JsValue::from_str("webkitRelativePath"))
                        .ok()
                        .and_then(|v| v.as_string())
                        .unwrap_or_default();

                if let Some(folder) = get_parent_folder(&relative_path) {
                    let should_use = if let Some(_existing) = folder_covers.get(&folder) {
                        let priority_names = ["cover", "folder", "front", "album"];
                        priority_names.iter().any(|pn| name.contains(pn))
                    } else {
                        true
                    };

                    if should_use {
                        web_audio_store::store_file(relative_path.clone(), file);
                        folder_covers.insert(folder, relative_path);
                    }
                }
            }
        }
    }

    for i in 0..file_count {
        if let Some(file) = files.get(i) {
            let name = file.name();
            let file_type = file.type_();

            let is_audio = file_type.starts_with("audio/")
                || name.ends_with(".mp3")
                || name.ends_with(".flac")
                || name.ends_with(".wav")
                || name.ends_with(".ogg")
                || name.ends_with(".m4a")
                || name.ends_with(".aac")
                || name.ends_with(".opus");

            if !is_audio {
                continue;
            }

            let relative_path: String =
                Reflect::get(&file, &JsValue::from_str("webkitRelativePath"))
                    .ok()
                    .and_then(|v| v.as_string())
                    .unwrap_or_else(|| name.clone());

            web_audio_store::store_file(relative_path.clone(), file);

            let (title, artist, album_name) = parse_filename(&name, &relative_path);

            let album_id = format!(
                "{}_{}",
                album_name.to_lowercase().replace(' ', "_"),
                artist.to_lowercase().replace(' ', "_")
            );

            let folder = get_parent_folder(&relative_path);
            let cover_path_key = folder.and_then(|f| folder_covers.get(&f).cloned());

            let track = Track {
                path: PathBuf::from(&relative_path),
                album_id: album_id.clone(),
                title,
                artist: artist.clone(),
                album: album_name.clone(),
                duration: 0,
                track_number: extract_track_number(&name),
                disc_number: None,
            };

            if !albums_map.contains_key(&album_id) {
                albums_map.insert(
                    album_id.clone(),
                    Album {
                        id: album_id,
                        title: album_name,
                        artist,
                        genre: String::new(),
                        year: 0,
                        cover_path: cover_path_key.map(PathBuf::from),
                    },
                );
            } else if cover_path_key.is_some() {
                if let Some(album) = albums_map.get_mut(&album_id) {
                    if album.cover_path.is_none() {
                        album.cover_path = cover_path_key.map(PathBuf::from);
                    }
                }
            }

            library.write().add_track(track);
            added_count += 1;
        }
    }

    for album in albums_map.into_values() {
        library.write().add_album(album);
    }

    {
        let mut cfg = config.write();
        cfg.last_folder_name = folder_name;
        cfg.has_loaded_folder = true;
    }

    let _ = library.read().save(&PathBuf::new());

    Ok(added_count)
}

fn get_parent_folder(path: &str) -> Option<String> {
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() >= 2 {
        Some(parts[..parts.len() - 1].join("/"))
    } else {
        None
    }
}

fn parse_filename(name: &str, relative_path: &str) -> (String, String, String) {
    let name_without_ext = name.rsplit_once('.').map(|(n, _)| n).unwrap_or(name);

    let path_parts: Vec<&str> = relative_path.split('/').collect();

    let (artist, album_name) = if path_parts.len() >= 3 {
        (
            path_parts[path_parts.len() - 3].to_string(),
            path_parts[path_parts.len() - 2].to_string(),
        )
    } else if path_parts.len() >= 2 {
        let folder = path_parts[path_parts.len() - 2];
        if folder.contains(" - ") {
            let parts: Vec<&str> = folder.splitn(2, " - ").collect();
            (
                parts[0].to_string(),
                parts.get(1).unwrap_or(&"Unknown Album").to_string(),
            )
        } else {
            ("Unknown Artist".to_string(), folder.to_string())
        }
    } else {
        ("Unknown Artist".to_string(), "Unknown Album".to_string())
    };

    let title = if let Some(captures) = extract_title(name_without_ext) {
        captures
    } else {
        name_without_ext.to_string()
    };

    (title, artist, album_name)
}

fn extract_title(name: &str) -> Option<String> {
    let trimmed = name.trim();

    if let Some(idx) = trimmed.find(" - ") {
        let before = &trimmed[..idx];
        if before.chars().all(|c| c.is_ascii_digit()) {
            return Some(trimmed[idx + 3..].trim().to_string());
        }
    }

    if let Some(idx) = trimmed.find(". ") {
        let before = &trimmed[..idx];
        if before.chars().all(|c| c.is_ascii_digit()) {
            return Some(trimmed[idx + 2..].trim().to_string());
        }
    }

    None
}

fn extract_track_number(name: &str) -> Option<u32> {
    let trimmed = name.trim();

    let digits: String = trimmed.chars().take_while(|c| c.is_ascii_digit()).collect();

    if !digits.is_empty() {
        digits.parse().ok()
    } else {
        None
    }
}

#[component]
fn SettingItem(title: &'static str, description: String, control: Element) -> Element {
    rsx! {
        div { class: "flex items-center justify-between py-2",
            div {
                p { class: "text-white font-medium", "{title}" }
                p { class: "text-sm text-slate-500", "{description}" }
            }
            {control}
        }
    }
}

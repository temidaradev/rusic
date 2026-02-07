use crate::config::AppConfig;
use crate::reader::Library;
use crate::web_audio_store;
use dioxus::prelude::*;
use js_sys::Reflect;
use std::collections::HashMap;
use std::path::PathBuf;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{FileList, HtmlInputElement};

#[component]
pub fn FolderReloadPrompt(
    config: Signal<AppConfig>,
    library: Signal<Library>,
    on_dismiss: EventHandler<()>,
) -> Element {
    let mut is_loading = use_signal(|| false);
    let mut error_msg = use_signal(|| Option::<String>::None);

    let folder_name = config
        .read()
        .last_folder_name
        .clone()
        .unwrap_or_else(|| "your music folder".to_string());

    rsx! {
        div {
            class: "fixed inset-0 bg-black/80 backdrop-blur-sm z-50 flex items-center justify-center",
            onclick: move |_| on_dismiss.call(()),

            div {
                class: "bg-stone-900 border border-white/10 rounded-xl p-8 max-w-md w-full mx-4 shadow-2xl",
                onclick: move |e| e.stop_propagation(),

                div {
                    class: "text-center mb-6",
                    div {
                        class: "w-16 h-16 bg-white/5 rounded-full flex items-center justify-center mx-auto mb-4",
                        i { class: "fa-solid fa-folder-open text-3xl text-white/60" }
                    }
                    h2 { class: "text-xl font-bold text-white mb-2", "Reload Music Folder" }
                    p { class: "text-slate-400 text-sm",
                        "Your library data was saved, but audio files need to be reloaded."
                    }
                    p { class: "text-slate-500 text-sm mt-2",
                        "Last folder: "
                        span { class: "text-white/70 font-medium", "{folder_name}" }
                    }
                }

                div {
                    class: "space-y-3",
                    div {
                        class: "relative",
                        input {
                            r#type: "file",
                            "webkitdirectory": "true",
                            "directory": "true",
                            class: "absolute inset-0 w-full h-full opacity-0 cursor-pointer z-10",
                            id: "reload-folder-input",
                            disabled: *is_loading.read(),
                            onchange: move |_| {
                                is_loading.set(true);
                                error_msg.set(None);

                                spawn(async move {
                                    match reload_folder(config, library).await {
                                        Ok(_) => {
                                            is_loading.set(false);
                                            on_dismiss.call(());
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
                                "w-full bg-white/10 text-slate-500 py-3 rounded-lg font-medium flex items-center justify-center gap-2"
                            } else {
                                "w-full bg-green-600 hover:bg-green-500 text-white py-3 rounded-lg font-medium transition-colors flex items-center justify-center gap-2"
                            },
                            if *is_loading.read() {
                                i { class: "fa-solid fa-spinner fa-spin" }
                                "Loading..."
                            } else {
                                i { class: "fa-solid fa-folder-open" }
                                "Select Folder"
                            }
                        }
                    }

                    button {
                        class: "w-full bg-white/5 hover:bg-white/10 text-slate-400 py-3 rounded-lg font-medium transition-colors",
                        onclick: move |_| on_dismiss.call(()),
                        "Skip for Now"
                    }

                    if let Some(err) = error_msg.read().as_ref() {
                        p { class: "text-red-400 text-sm text-center", "{err}" }
                    }
                }

                p { class: "text-slate-600 text-xs text-center mt-4",
                    "Due to browser security, files must be re-selected after page reload."
                }
            }
        }
    }
}

async fn reload_folder(
    mut config: Signal<AppConfig>,
    mut library: Signal<Library>,
) -> Result<(), String> {
    let window = web_sys::window().ok_or("No window")?;
    let document = window.document().ok_or("No document")?;

    let input: HtmlInputElement = document
        .get_element_by_id("reload-folder-input")
        .ok_or("No input element")?
        .dyn_into()
        .map_err(|_| "Not an input element")?;

    let files: FileList = input.files().ok_or("No files selected")?;
    let file_count = files.length();

    if file_count == 0 {
        return Err("No files found in folder".to_string());
    }

    let mut folder_name = None;
    if let Some(first_file) = files.get(0) {
        if let Ok(path) = Reflect::get(&first_file, &JsValue::from_str("webkitRelativePath")) {
            if let Some(path_str) = path.as_string() {
                if let Some(first_part) = path_str.split('/').next() {
                    folder_name = Some(first_part.to_string());
                }
            }
        }
    }

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
                    || name.ends_with(".jpg")
                    || name.ends_with(".jpeg")
                    || name.ends_with(".png"));

            if is_cover_image {
                let relative_path: String =
                    Reflect::get(&file, &JsValue::from_str("webkitRelativePath"))
                        .ok()
                        .and_then(|v| v.as_string())
                        .unwrap_or_default();

                if let Some(folder) = get_parent_folder(&relative_path) {
                    let priority_names = ["cover", "folder", "front", "album"];
                    let should_use = folder_covers.get(&folder).is_none()
                        || priority_names.iter().any(|pn| name.contains(pn));

                    if should_use {
                        web_audio_store::store_file(relative_path.clone(), file);
                        folder_covers.insert(folder, relative_path);
                    }
                }
            }
        }
    }

    let mut processed_count = 0;
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
            processed_count += 1;
        }
    }

    {
        let mut lib = library.write();
        for album in &mut lib.albums {
            // Try to find a cover for this album based on folder structure
            for (folder, cover_path) in &folder_covers {
                // Check if this folder matches the album
                if folder.to_lowercase().contains(&album.title.to_lowercase())
                    || folder.to_lowercase().contains(&album.artist.to_lowercase())
                {
                    album.cover_path = Some(PathBuf::from(cover_path));
                    break;
                }
            }
        }
    }

    {
        let mut cfg = config.write();
        cfg.last_folder_name = folder_name;
        cfg.has_loaded_folder = true;
    }

    let _ = library.read().save(&PathBuf::new());

    if processed_count == 0 {
        return Err("No audio files found in the selected folder".to_string());
    }

    Ok(())
}

fn get_parent_folder(path: &str) -> Option<String> {
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() >= 2 {
        Some(parts[..parts.len() - 1].join("/"))
    } else {
        None
    }
}

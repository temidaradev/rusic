use crate::track_row::TrackRow;
use config::{AppConfig, MusicSource};
use dioxus::prelude::*;
use reader::{Library, Track};

#[derive(Props, Clone, PartialEq)]
pub struct ShowcaseProps {
    pub name: String,
    pub description: String,
    pub cover_url: Option<String>,
    pub tracks: Vec<Track>,
    pub library: Signal<Library>,
    pub on_play: EventHandler<usize>,
    pub on_add_to_playlist: Option<EventHandler<usize>>,
    pub on_delete_track: Option<EventHandler<usize>>,
    pub active_track: Option<std::path::PathBuf>,
    pub on_click_menu: Option<EventHandler<usize>>,
    pub on_close_menu: Option<EventHandler<()>>,
    pub actions: Option<Element>,
}

#[component]
pub fn Showcase(props: ShowcaseProps) -> Element {
    let config = use_context::<Signal<AppConfig>>();
    let total_seconds: u64 = props.tracks.iter().map(|t| t.duration).sum();
    let duration_min = total_seconds / 60;

    let lib = props.library.read();
    let is_jellyfin = config.read().active_source == MusicSource::Jellyfin;

    rsx! {
         div {
             div {
                 class: "flex flex-col md:flex-row items-center md:items-end gap-6 md:gap-8 mb-8 md:mb-12 text-center md:text-left",
                 div { class: "w-48 h-48 md:w-64 md:h-64 rounded-xl bg-stone-800 shadow-2xl overflow-hidden relative flex-shrink-0",
                     if let Some(url) = &props.cover_url {
                         img { src: "{url}", class: "w-full h-full object-cover" }
                     } else {
                         div { class: "w-full h-full flex flex-col items-center justify-center text-white/20",
                             i { class: "fa-solid fa-music text-4xl md:text-6xl mb-4" }
                         }
                     }
                 }
                 div { class: "flex-1 flex flex-col items-center md:items-start",
                     if !props.description.is_empty() {
                         h5 { class: "text-xs md:text-sm font-bold tracking-widest text-white/60 uppercase mb-2", "{props.description}" }
                     }
                     h1 { class: "text-3xl md:text-5xl lg:text-7xl font-bold text-white mb-4 md:mb-6", "{props.name}" }
                     div { class: "flex items-center gap-4 md:gap-6 text-slate-400 text-sm md:text-base mb-6 md:mb-0",
                         p { "{props.tracks.len()} songs" }
                         span { "•" }
                         p { "{duration_min} min" }
                     }
                 }

                div { class: "flex items-center gap-4 self-center md:self-auto",
                     if !props.tracks.is_empty() {
                         button {
                             class: "w-12 h-12 md:w-14 md:h-14 rounded-full bg-indigo-500 hover:bg-indigo-400 text-black flex items-center justify-center transition-transform hover:scale-105 shadow-xl",
                             onclick: move |_| props.on_play.call(0),
                             i { class: "fa-solid fa-play text-lg md:text-xl ml-1" }
                         }
                     }
                     if let Some(actions) = props.actions {
                         {actions}
                     }
                 }
             }

             div { class: "space-y-1",
                 if props.tracks.is_empty() {
                     div { class: "py-12 flex flex-col items-center justify-center text-slate-600",
                         i { class: "fa-regular fa-folder-open text-4xl mb-4" }
                         p { class: "text-lg", "No songs here." }
                     }
                 } else {
                     div { class: "hidden md:grid grid-cols-[auto_1fr_1fr_auto_auto] gap-4 px-4 py-2 border-b border-white/5 text-sm font-medium text-slate-500 mb-2 uppercase tracking-wider",
                          div { class: "w-8 text-center", "#" }
                          div { "Title" }
                          div { "Album" }
                     }

                     for (idx, track) in props.tracks.iter().enumerate() {
                         {
                             let cover_url = if is_jellyfin {
                                 if let Some(server) = &config.read().server {
                                     let path_str = track.path.to_string_lossy();
                                     let parts: Vec<&str> = path_str.split(':').collect();
                                     if parts.len() >= 2 {
                                         let id = parts[1];
                                         let mut url = format!("{}/Items/{}/Images/Primary", server.url, id);
                                         let mut params = Vec::new();
                                         if parts.len() >= 3 { params.push(format!("tag={}", parts[2])); }
                                         if let Some(token) = &server.access_token { params.push(format!("api_key={}", token)); }
                                         if !params.is_empty() {
                                             url.push('?');
                                             url.push_str(&params.join("&"));
                                         }
                                         Some(url)
                                     } else { None }
                                 } else { None }
                             } else {
                                 lib.albums.iter()
                                    .find(|a| a.id == track.album_id)
                                    .and_then(|a| utils::format_artwork_url(a.cover_path.as_ref()))
                             };

                             rsx! {
                                 TrackRow {
                                     key: "{track.path.display()}",
                                     track: track.clone(),
                                     cover_url: cover_url,
                                     is_menu_open: props.active_track.as_ref() == Some(&track.path),
                                     on_click_menu: move |_| {
                                        if let Some(handler) = &props.on_click_menu {
                                            handler.call(idx);
                                        }
                                     },
                                     on_add_to_playlist: move |_| {
                                        if let Some(handler) = &props.on_add_to_playlist {
                                            handler.call(idx);
                                        }
                                     },
                                     on_close_menu: move |_| {
                                        if let Some(handler) = &props.on_close_menu {
                                            handler.call(());
                                        }
                                     },
                                     on_delete: move |_| {
                                        if let Some(handler) = &props.on_delete_track {
                                            handler.call(idx);
                                        }
                                     },
                                     on_play: move |_| props.on_play.call(idx)
                                 }
                             }
                         }
                     }
                 }
             }
         }
    }
}

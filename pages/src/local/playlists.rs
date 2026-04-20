use config::AppConfig;
use dioxus::prelude::*;
use reader::{Library, PlaylistStore};

#[component]
pub fn LocalPlaylists(
    playlist_store: Signal<PlaylistStore>,
    library: Signal<Library>,
    config: Signal<AppConfig>,
    mut selected_playlist_id: Signal<Option<String>>,
) -> Element {
    let store = playlist_store.read();

    rsx! {
        div {
            if store.playlists.is_empty() {
                div { class: "flex flex-col items-center justify-center h-64 text-slate-500",
                    i { class: "fa-regular fa-folder-open text-4xl mb-4 opacity-50" }
                    p { "{rust_i18n::t!(\"no_playlists_yet\")}" }
                }
            } else {
                div { class: "grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-6",
                    {store.playlists.iter().map(|playlist| {
                        let cover_url = if let Some(first_track_path) = playlist.tracks.first() {
                            let lib = library.peek();
                            lib.tracks.iter()
                                .find(|t| t.path == *first_track_path)
                                .and_then(|t| {
                                    lib.albums.iter()
                                        .find(|a| a.id == t.album_id)
                                        .and_then(|a| a.cover_path.as_ref())
                                        .and_then(|cp| utils::format_artwork_url(Some(cp)))
                                })
                        } else {
                            None
                        };

                        rsx! {
                            div {
                                key: "{playlist.id}",
                                class: "bg-white/5 border border-white/5 rounded-2xl p-6 hover:bg-white/10 transition-all cursor-pointer group",
                                onclick: {
                                    let id = playlist.id.clone();
                                    move |_| selected_playlist_id.set(Some(id.clone()))
                                },
                                div {
                                    class: "mb-4 w-full aspect-square rounded-xl flex items-center justify-center overflow-hidden transition-all bg-white/5",
                                    if let Some(url) = cover_url {
                                        img {
                                            src: "{url}",
                                            class: "w-full h-full object-cover group-hover:scale-105 transition-transform duration-500"
                                        }
                                    } else {
                                        div {
                                            class: "w-full h-full flex items-center justify-center",
                                            style: "background: color-mix(in srgb, var(--color-indigo-500), transparent 80%); color: var(--color-indigo-400)",
                                            i { class: "fa-solid fa-list-ul text-2xl" }
                                        }
                                    }
                                }
                                h3 { class: "text-xl font-bold text-white mb-1 truncate", "{playlist.name}" }
                                {
                                    let count = playlist.tracks.len();
                                    let track_text = rust_i18n::t!("playlist_track_count", count = count).to_string();
                                    rsx! { 
                                        p { class: "text-sm text-slate-400", "{track_text}" }
                                    }
                                }
                            }
                        }
                    })}
                }
            }
        }
    }
}

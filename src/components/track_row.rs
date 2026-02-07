use crate::reader::models::Track;
use dioxus::prelude::*;

#[component]
pub fn TrackRow(
    track: Track,
    cover_url: Option<String>,
    on_click_menu: EventHandler<()>,
    is_menu_open: bool,
    on_add_to_playlist: EventHandler<()>,
    on_close_menu: EventHandler<()>,
    on_play: EventHandler<()>,
    on_delete: EventHandler<()>,
) -> Element {
    rsx! {
        div {
            class: "flex items-center p-2 rounded-lg hover:bg-white/5 group transition-colors relative",
            onclick: move |_| on_play.call(()),
            div { class: "w-10 h-10 bg-white/5 rounded overflow-hidden flex items-center justify-center mr-4 shrink-0",
                if let Some(url) = cover_url {
                    img {
                        src: "{url}",
                        class: "w-full h-full object-cover",
                        loading: "lazy",
                        decoding: "async",
                    }
                } else {
                    i { class: "fa-solid fa-music text-white/20" }
                }
            }
            div { class: "flex-1 min-w-0 pr-4",
                p { class: "text-sm font-medium text-white/90 truncate",
                    "{track.title}"
                }
                p { class: "text-xs text-slate-500 truncate",
                    "{track.artist}"
                }
            }
            div { class: "relative",
                button {
                    class: "w-8 h-8 flex items-center justify-center rounded-full hover:bg-white/10 text-slate-400 hover:text-white transition-colors opacity-0 group-hover:opacity-100 focus:opacity-100",
                    onclick: move |evt| {
                        evt.stop_propagation();
                        on_click_menu.call(());
                    },
                    i { class: "fa-solid fa-ellipsis-vertical" }
                }

                if is_menu_open {
                    div {
                        class: "absolute right-0 top-full mt-1 w-48 bg-neutral-900 border border-white/10 rounded-lg shadow-xl z-20 py-1",
                        onclick: move |evt| evt.stop_propagation(),
                        button {
                            class: "w-full text-left px-4 py-2 text-sm text-white hover:bg-white/10 flex items-center gap-2",
                            onclick: move |_| {
                                on_add_to_playlist.call(());
                            },
                            i { class: "fa-solid fa-plus" }
                            "Add to Playlist"
                        }
                        button {
                            class: "w-full text-left px-4 py-2 text-sm text-red-500 hover:bg-white/10 flex items-center gap-2",
                            onclick: move |_| {
                                on_delete.call(());
                            },
                            i { class: "fa-solid fa-trash" }
                            "Delete Song"
                        }
                    }
                    div {
                        class: "fixed inset-0 z-10",
                        onclick: move |evt| {
                            evt.stop_propagation();
                            on_close_menu.call(());
                        }
                    }
                }
            }
        }
    }
}

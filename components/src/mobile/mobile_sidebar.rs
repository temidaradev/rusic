use config::MusicSource;
use dioxus::prelude::*;
use rusic_route::Route;

#[derive(PartialEq, Clone, Copy)]
struct SidebarItem {
    name: &'static str,
    route: Route,
    icon: &'static str,
}

const TOP_MENU: &[SidebarItem] = &[
    SidebarItem {
        name: "Home",
        route: Route::Home,
        icon: "fa-solid fa-house",
    },
    SidebarItem {
        name: "Search",
        route: Route::Search,
        icon: "fa-solid fa-magnifying-glass",
    },
    SidebarItem {
        name: "Library",
        route: Route::Library,
        icon: "fa-solid fa-book",
    },
    SidebarItem {
        name: "Album",
        route: Route::Album,
        icon: "fa-solid fa-music",
    },
    SidebarItem {
        name: "Artist",
        route: Route::Artist,
        icon: "fa-solid fa-user",
    },
    SidebarItem {
        name: "Playlists",
        route: Route::Playlists,
        icon: "fa-solid fa-list",
    },
    SidebarItem {
        name: "Logs",
        route: Route::Logs,
        icon: "fa-solid fa-clipboard-list",
    },
];

const BOTTOM_MENU: &[SidebarItem] = &[SidebarItem {
    name: "Settings",
    route: Route::Settings,
    icon: "fa-solid fa-gear",
}];

#[derive(Props, Clone, PartialEq)]
pub struct MobileSidebarProps {
    current_route: Signal<Route>,
    is_sidebar_open: Signal<bool>,
    on_navigate: EventHandler<Route>,
}

#[component]
pub fn MobileSidebar(props: MobileSidebarProps) -> Element {
    let mut config = use_context::<Signal<config::AppConfig>>();
    let mut is_sidebar_open = props.is_sidebar_open;

    let is_jellyfin = config.read().active_source == MusicSource::Jellyfin;
    let local_class = if !is_jellyfin {
        "text-white"
    } else {
        "text-slate-500 hover:text-slate-300"
    };
    let jellyfin_class = if is_jellyfin {
        "text-white"
    } else {
        "text-slate-500 hover:text-slate-300"
    };
    let slider_style = if is_jellyfin {
        "left: calc(50% + 2px); width: calc(50% - 4px);"
    } else {
        "left: 4px; width: calc(50% - 4px);"
    };

    let sidebar_open = *is_sidebar_open.read();

    if !sidebar_open {
        return rsx! { div { class: "hidden" } };
    }

    rsx! {
        // Mobile backdrop
        div {
            class: "fixed inset-0 bg-black/60 z-40",
            onclick: move |_| is_sidebar_open.set(false),
        }

        div {
            class: "fixed inset-y-0 left-0 z-50 w-64 h-full bg-[#121212]/95 backdrop-blur-xl text-slate-400 flex-col flex-shrink-0 select-none transition-transform duration-300 ease-out border-r border-white/5",

            div {
                class: "h-20 flex items-center justify-between px-6 mb-4 transition-all",
                h2 {
                    class: "text-lg font-bold tracking-widest text-white/90 uppercase",
                    style: "font-family: 'JetBrains Mono', monospace;",
                    "RUSIC"
                }

                button {
                    class: "p-2 rounded-lg hover:bg-white/5 text-slate-500 hover:text-white transition-all active:scale-95 flex items-center justify-center shrink-0",
                    onclick: move |_| is_sidebar_open.set(false),
                    i { class: "fa-solid fa-xmark w-6 h-6 flex items-center justify-center text-xl" }
                }
            }

            div {
                class: "flex-1 flex flex-col overflow-y-auto overflow-x-hidden",

                div {
                    class: "px-4 mb-6",
                    div {
                        class: "bg-white/5 p-1 rounded-xl flex relative h-10 items-center border border-white/5",
                        div {
                            class: "absolute h-8 bg-white/10 rounded-lg transition-all duration-300 ease-out",
                            style: "{slider_style}"
                        }
                        button {
                            class: "flex-1 text-[11px] font-bold z-10 transition-colors duration-300 {local_class}",
                            onclick: move |_| config.write().active_source = MusicSource::Local,
                            "LOCAL"
                        }
                        button {
                            class: "flex-1 text-[11px] font-bold z-10 transition-colors duration-300 {jellyfin_class}",
                            onclick: move |_| config.write().active_source = MusicSource::Jellyfin,
                            "JELLYFIN"
                        }
                    }
                }

                nav {
                    class: "flex-1 px-3 space-y-1",
                    for item in TOP_MENU {
                        MobileSidebarLink {
                            item: *item,
                            active: *props.current_route.read() == item.route,
                            onclick: move |_| {
                                props.on_navigate.call(item.route);
                                is_sidebar_open.set(false);
                            }
                        }
                    }
                    div { class: "h-px bg-white/5 my-4 mx-3" }
                    for item in BOTTOM_MENU {
                        MobileSidebarLink {
                            item: *item,
                            active: *props.current_route.read() == item.route,
                            onclick: move |_| {
                                props.on_navigate.call(item.route);
                                is_sidebar_open.set(false);
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn MobileSidebarLink(
    item: SidebarItem,
    active: bool,
    onclick: EventHandler<MouseEvent>,
) -> Element {
    let active_class = if active {
        "bg-white/10 text-white"
    } else {
        "text-slate-400 hover:text-white/90 hover:bg-white/5"
    };

    let opacity_class = if active {
        "opacity-100"
    } else {
        "opacity-70 group-hover:opacity-100"
    };

    rsx! {
        a {
            class: "flex items-center justify-start px-3 group relative p-3 rounded-lg transition-all duration-200 cursor-pointer {active_class}",
            onclick: move |evt| onclick.call(evt),

            div {
                class: "flex items-center justify-center w-6 h-6 shrink-0 transition-transform group-active:scale-95",
                i { class: "{item.icon} text-lg" }
            }

            span {
                class: "ml-4 text-sm font-medium tracking-tight {opacity_class} transition-opacity",
                "{item.name}"
            }

            div {
                class: if active {
                    "absolute left-0 w-0.5 rounded-r-full transition-all duration-300 h-6 bg-white"
                } else {
                    "absolute left-0 w-0.5 rounded-r-full transition-all duration-300 h-0 bg-white/40 group-hover:h-4"
                }
            }
        }
    }
}

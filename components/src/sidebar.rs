use config::MusicSource;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use dioxus::desktop::use_window;
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
        name: "Favorites",
        route: Route::Favorites,
        icon: "fa-solid fa-heart",
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
pub struct SidebarProps {
    current_route: Signal<Route>,
    on_navigate: EventHandler<Route>,
}

#[component]
pub fn Sidebar(props: SidebarProps) -> Element {
    let mut config = use_context::<Signal<config::AppConfig>>();
    let default_collapsed = use_signal(|| cfg!(any(target_os = "android", target_os = "ios")));
    let mut is_collapsed = try_consume_context::<Signal<bool>>().unwrap_or(default_collapsed);
    let mut width = use_signal(|| 240);
    let mut is_resizing = use_signal(|| false);

    let is_mobile = cfg!(any(target_os = "android", target_os = "ios"));

    let current_width = if *is_collapsed.read() {
        if is_mobile { 0 } else { 72 }
    } else {
        *width.read()
    };

    let onmousemove = move |evt: MouseEvent| {
        if *is_resizing.read() {
            let new_width = evt.client_coordinates().x as i32;
            if new_width > 180 && new_width < 450 {
                width.set(new_width);
            }
        }
    };

    let onmouseup = move |_| {
        is_resizing.set(false);
    };

    let header_class = if *is_collapsed.read() {
        "justify-center px-0"
    } else {
        "justify-between px-6"
    };

    let extra_padding = if cfg!(target_os = "macos") {
        "pt-10"
    } else {
        ""
    };

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

    let mobile_sidebar_style = if is_mobile {
        "position: fixed; left: 0; top: 0; z-index: 100; height: 100%;"
    } else {
        ""
    };

    let collapse_icon = if *is_collapsed.read() {
        "fa-angles-right"
    } else if is_mobile {
        "fa-xmark"
    } else {
        "fa-angles-left"
    };

    rsx! {
        if !is_mobile && *is_resizing.read() {
             div {
                 class: "fixed inset-0 z-[100] cursor-col-resize",
                 onmousemove: onmousemove,
                 onmouseup: onmouseup,
             }
        }

        if is_mobile && !*is_collapsed.read() {
            div {
                class: "fixed inset-0 bg-black/60 backdrop-blur-sm z-[90] animate-fade-in",
                onclick: move |_| is_collapsed.set(true)
            }
        }

        div {
            class: "h-full bg-black/40 text-slate-400 flex flex-col flex-shrink-0 select-none relative transition-all duration-300 ease-out border-r border-white/5 overflow-hidden {extra_padding}",
            style: "width: {current_width}px; {mobile_sidebar_style}",

            div {
                class: "absolute top-0 left-0 w-full h-10 z-50",
                onmousedown: move |_| {
                    #[cfg(not(any(target_os = "android", target_os = "ios")))]
                    if cfg!(target_os = "macos") {
                        use_window().drag();
                    }
                }
            }

            div {
                class: "h-20 flex items-center mb-4 transition-all {header_class}",
                onmousedown: move |_| {
                    #[cfg(not(any(target_os = "android", target_os = "ios")))]
                    if cfg!(target_os = "macos") {
                        use_window().drag();
                    }
                },
                if !*is_collapsed.read() {
                    h2 {
                        class: "text-lg font-bold tracking-widest text-white/90 uppercase",
                        style: "font-family: 'JetBrains Mono', monospace;",
                        "RUSIC"
                    }
                }

                button {
                    class: "p-2 rounded-lg hover:bg-white/5 text-slate-500 hover:text-white transition-all active:scale-95 flex items-center justify-center shrink-0",
                    onclick: move |_| is_collapsed.toggle(),
                    i { class: "fa-solid {collapse_icon} w-6 h-6 flex items-center justify-center text-xl" }
                }
            }

            div {
                class: "flex-1 flex flex-col overflow-y-auto overflow-x-hidden",

                if !*is_collapsed.read() {
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
                }

                nav {
                    class: "flex-1 px-3 space-y-1",
                    for item in TOP_MENU {
                        SidebarLink {
                            item: *item,
                            collapsed: is_collapsed,
                            active: *props.current_route.read() == item.route,
                            onclick: move |_| props.on_navigate.call(item.route)
                        }
                    }
                    div { class: "h-px bg-white/5 my-4 mx-3" }
                    for item in BOTTOM_MENU {
                        SidebarLink {
                            item: *item,
                            collapsed: is_collapsed,
                            active: *props.current_route.read() == item.route,
                            onclick: move |_| props.on_navigate.call(item.route)
                        }
                    }
                }
            }

            if !*is_collapsed.read() {
                div {
                    class: "absolute top-0 right-0 w-1 h-full cursor-col-resize group/handle z-50",
                    onmousedown: move |_| is_resizing.set(true),
                    div { class: "absolute inset-y-0 right-0 w-px bg-white/0 group-hover/handle:bg-white/10 transition-colors" }
                }
            }
        }
    }
}

#[component]
fn SidebarLink(
    item: SidebarItem,
    collapsed: Signal<bool>,
    active: bool,
    onclick: EventHandler<MouseEvent>,
) -> Element {
    let is_collapsed = *collapsed.read();
    let alignment_class = if is_collapsed {
        "justify-center"
    } else {
        "justify-start px-3"
    };

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
            class: "flex items-center {alignment_class} group relative p-3 rounded-lg transition-all duration-200 cursor-pointer {active_class}",
            title: if is_collapsed { item.name } else { "" },
            onclick: move |evt| onclick.call(evt),

            div {
                class: "flex items-center justify-center w-6 h-6 shrink-0 transition-transform group-active:scale-95",
                i { class: "{item.icon} text-lg" }
            }

            if !is_collapsed {
                span {
                    class: "ml-4 text-sm font-medium tracking-tight {opacity_class} transition-opacity",
                    "{item.name}"
                }
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

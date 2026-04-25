use config::{AppConfig, MusicServer};
use dioxus::prelude::*;
use rfd::AsyncFileDialog;

#[component]
pub fn SettingItem(title: String, control: Element) -> Element {
    rsx! {
        div { class: "flex items-center justify-between py-2",
            p { class: "text-white font-medium", "{title}" }
            {control}
        }
    }
}

#[component]
pub fn LanguageSelector(current_language: String, on_change: EventHandler<String>) -> Element {
    rsx! {
        select {
            class: "bg-white/5 border border-white/10 rounded px-3 py-1 text-sm text-white focus:outline-none focus:border-white/20",
            value: "{current_language}",
            onchange: move |evt| on_change.call(evt.value()),
            option { value: "en", "{rust_i18n::t!(\"english\")}" }
            option { value: "ru", "{rust_i18n::t!(\"russian\")}" }
            option { value: "tok", "{rust_i18n::t!(\"toki pona\")}" }
        }
    }
}

#[component]
pub fn ThemeSelector(current_theme: String, on_change: EventHandler<String>) -> Element {
    let config = use_context::<Signal<AppConfig>>();
    let mut custom: Vec<(String, String)> = config
        .read()
        .custom_themes
        .iter()
        .map(|(id, ct)| (id.clone(), ct.name.clone()))
        .collect();
    custom.sort_by(|a, b| a.1.cmp(&b.1));

    rsx! {
        select {
            class: "bg-white/5 border border-white/10 rounded px-3 py-1 text-sm text-white focus:outline-none focus:border-white/20",
            value: "{current_theme}",
            onchange: move |evt| on_change.call(evt.value()),
            optgroup { label: "{rust_i18n::t!(\"theme_group_dynamic\")}",
                option { value: "album-art", "{rust_i18n::t!(\"album_art_gradient\")}" }
            }
            optgroup { label: "{rust_i18n::t!(\"theme_group_dark\")}",
                option { value: "default", "{rust_i18n::t!(\"default_theme\")}" }
                option { value: "gruvbox", "{rust_i18n::t!(\"gruvbox_material\")}" }
                option { value: "gruvbox-classic", "{rust_i18n::t!(\"gruvbox_classic\")}" }
                option { value: "gruvbox-dark-soft", "{rust_i18n::t!(\"gruvbox_dark_soft\")}" }
                option { value: "dracula", "{rust_i18n::t!(\"dracula\")}" }
                option { value: "nord", "{rust_i18n::t!(\"nord\")}" }
                option { value: "catppuccin", "{rust_i18n::t!(\"catppuccin_mocha\")}" }
                option { value: "ef-night", "{rust_i18n::t!(\"ef_night\")}" }
                option { value: "ayu-dark", "{rust_i18n::t!(\"ayu_dark\")}" }
                option { value: "ayu-mirage", "{rust_i18n::t!(\"ayu_mirage\")}" }
                option { value: "vague", "{rust_i18n::t!(\"vague\")}" }
                option { value: "onedarkpro", "{rust_i18n::t!(\"one_dark_pro\")}" }
                option { value: "osmium", "{rust_i18n::t!(\"osmium\")}" }
                option { value: "kanagawa-dragon", "{rust_i18n::t!(\"kanagawa_dragon\")}" }
                option { value: "everforest", "{rust_i18n::t!(\"everforest\")}" }
                option { value: "rosepine", "{rust_i18n::t!(\"rosepine\")}" }
                option { value: "kettek16", "kettek16" }
            }
            optgroup { label: "{rust_i18n::t!(\"theme_group_light\")}",
                option { value: "default-light", "{rust_i18n::t!(\"default_light\")}" }
                option { value: "catppuccin-latte", "{rust_i18n::t!(\"catppuccin_latte\")}" }
                option { value: "rosepine-dawn", "{rust_i18n::t!(\"rosepine_dawn\")}" }
                option { value: "everforest-light", "{rust_i18n::t!(\"everforest_light\")}" }
                option { value: "ayu-light", "{rust_i18n::t!(\"ayu_light\")}" }
                option { value: "one-light", "{rust_i18n::t!(\"one_light\")}" }
                option { value: "gruvbox-light", "{rust_i18n::t!(\"gruvbox_light_soft\")}" }
            }
            if !custom.is_empty() {
                optgroup { label: "{rust_i18n::t!(\"theme_group_custom\")}",
                    for (id, name) in &custom {
                        option { value: "{id}", "{name}" }
                    }
                }
            }
        }
    }
}

#[component]
pub fn DirectoryPicker(current_path: String, on_change: EventHandler<std::path::PathBuf>) -> Element {
    rsx! {
        div { class: "flex items-center gap-3",
            span { class: "text-xs text-slate-500 font-mono truncate max-w-xs", "{current_path}" }
            button {
                onclick: move |_| {
                    #[cfg(not(target_arch = "wasm32"))]
                    spawn(async move {
                        if let Some(handle) = AsyncFileDialog::new().pick_folder().await {
                            let path = handle.path().to_path_buf();
                            on_change.call(path);
                        }
                    });
                },
                class: "bg-white/10 hover:bg-white/20 px-3 py-1 rounded text-sm text-white transition-colors shrink-0",
                "{rust_i18n::t!(\"change\")}"
            }
        }
    }
}

#[component]
pub fn ServerSettings(
    server: Option<MusicServer>,
    on_add: EventHandler<()>,
    on_delete: EventHandler<()>,
    on_login: EventHandler<()>,
) -> Element {
    let login_text = rust_i18n::t!("login").to_string();
    let delete_text = rust_i18n::t!("delete").to_string();

    rsx! {
        div { class: "flex flex-col gap-2",
            if let Some(server) = server {
                div { class: "flex items-center justify-between gap-4 bg-white/5 p-2 rounded w-full",
                    div {
                        p { class: "text-sm font-medium text-white", "{server.name}" }
                        p { class: "text-xs text-white/60", "Service: {server.service.display_name()}" }
                        p { class: "text-xs text-white/60", "{server.url}" }
                        if server.access_token.is_some() {
                            p { class: "text-xs text-green-400 mt-1", "● Connected" }
                        } else {
                            div { class: "flex items-center gap-2 mt-1",
                                p { class: "text-xs text-red-400", "● Disconnected" }
                                button {
                                    onclick: move |_| on_login.call(()),
                                    class: "text-xs bg-white/10 hover:bg-white/20 px-2 py-0.5 rounded text-white transition-colors",
                                    "{login_text}"
                                }
                            }
                        }
                    }
                    button {
                        onclick: move |_| on_delete.call(()),
                        class: "text-red-400 hover:text-red-300 text-sm px-2 py-1 transition-colors",
                        "{delete_text}"
                    }
                }
            } else {
                button {
                    onclick: move |_| on_add.call(()),
                    class: "bg-white/10 hover:bg-white/20 px-3 py-1 rounded text-sm text-white transition-colors self-start",
                    "{rust_i18n::t!(\"add_server\")}"
                }
            }
        }
    }
}

#[component]
pub fn DiscordPresenceSettings(enabled: bool, on_change: EventHandler<bool>) -> Element {
    let slider_style = if enabled {
        "left: 4px; width: calc(50% - 4px);"
    } else {
        "left: calc(50% + 2px); width: calc(50% - 4px);"
    };

    let enable_class = if enabled {
        "text-white"
    } else {
        "text-slate-500 hover:text-slate-300"
    };

    let disable_class = if !enabled {
        "text-white"
    } else {
        "text-slate-500 hover:text-slate-300"
    };

    rsx! {
        div {
            class: "bg-white/5 p-1 rounded-xl flex relative h-10 items-center border border-white/5 w-48",
            div {
                class: "absolute h-8 bg-white/10 rounded-lg transition-all duration-300 ease-out",
                style: "{slider_style}"
            }
            button {
                class: "flex-1 text-[11px] font-bold z-10 transition-colors duration-300 cursor-pointer {enable_class}",
                onclick: move |_| on_change.call(true),
                "{rust_i18n::t!(\"enabled\")}"
            }
            button {
                class: "flex-1 text-[11px] font-bold z-10 transition-colors duration-300 cursor-pointer {disable_class}",
                onclick: move |_| on_change.call(false),
                "{rust_i18n::t!(\"disabled\")}"
            }
        }
    }
}

#[component]
pub fn ToggleSetting(enabled: bool, on_change: EventHandler<bool>) -> Element {
    let slider_style = if enabled {
        "left: 4px; width: calc(50% - 4px);"
    } else {
        "left: calc(50% + 2px); width: calc(50% - 4px);"
    };

    let enable_class = if enabled {
        "text-white"
    } else {
        "text-slate-500 hover:text-slate-300"
    };

    let disable_class = if !enabled {
        "text-white"
    } else {
        "text-slate-500 hover:text-slate-300"
    };

    rsx! {
        div {
            class: "bg-white/5 p-1 rounded-xl flex relative h-10 items-center border border-white/5 w-48",
            div {
                class: "absolute h-8 bg-white/10 rounded-lg transition-all duration-300 ease-out",
                style: "{slider_style}"
            }
            button {
                class: "flex-1 text-[11px] font-bold z-10 transition-colors duration-300 cursor-pointer {enable_class}",
                onclick: move |_| on_change.call(true),
                "{rust_i18n::t!(\"enabled\")}"
            }
            button {
                class: "flex-1 text-[11px] font-bold z-10 transition-colors duration-300 cursor-pointer {disable_class}",
                onclick: move |_| on_change.call(false),
                "{rust_i18n::t!(\"disabled\")}"
            }
        }
    }
}

#[component]
pub fn MusicBrainzSettings(current: String, on_save: EventHandler<String>) -> Element {
    let mut input = use_signal(move || current.clone());

    rsx! {
        div {
            class: "flex items-center gap-2 w-full max-w-xl",
            div {
                class: "flex-1 bg-white/5 p-1 rounded-xl border border-white/5",
                input {
                    class: "bg-transparent w-full px-3 py-2 text-sm text-white placeholder:text-white/50 outline-none",
                    placeholder: "{rust_i18n::t!(\"listenbrainz_token_placeholder\")}",
                    value: "{input()}",
                    oninput: move |evt| {
                        input.set(evt.value());
                        on_save.call(evt.value());
                    },
                    r#type: "password",
                }
            }
        }
    }
}

// #[component]
// pub fn LastFmSettings(current: String, on_save: EventHandler<String>) -> Element {
//     let mut input = use_signal(move || current.clone());

//     rsx! {
//         div { class: "flex items-center gap-2 w-full max-w-xl",
//             div { class: "flex-1 bg-white/5 p-1 rounded-xl border border-white/5",
//                 input {
//                     class: "bg-transparent w-full px-3 py-2 text-sm text-white placeholder:text-white/50 outline-none",
//                     placeholder: "Enter your last.fm token",
//                     value: "{input()}",
//                     oninput: move |evt| {
//                         input.set(evt.value());
//                         on_save.call(evt.value());
//                     },
//                     r#type: "text",
//                 }
//             }
//         }
//     }
// }

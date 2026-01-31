use config::AppConfig;
use dioxus::prelude::*;
use rfd::AsyncFileDialog;

#[component]
pub fn Settings(config: Signal<AppConfig>) -> Element {
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
                        SettingItem {
                            title: "Music Directory",
                            description: format!("Current path: {}", config.read().music_directory.display()),
                            control: rsx! {
                                button {
                                    onclick: move |_| {
                                        spawn(async move {
                                            if let Some(handle) = AsyncFileDialog::new().pick_folder().await {
                                                let path = handle.path().to_path_buf();
                                                config.write().music_directory = path;
                                            }
                                        });
                                    },
                                    class: "bg-white/10 hover:bg-white/20 px-3 py-1 rounded text-sm text-white transition-colors",
                                    "Change"
                                }
                            }
                        }
                    }
                }
            }
        }
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

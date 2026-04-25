use ::server::provider::ProviderClient;
use crate::theme_editor::ThemeEditorPage;
use components::settings_items::{
    DirectoryPicker, DiscordPresenceSettings, LanguageSelector, MusicBrainzSettings,
    ServerSettings, SettingItem, ThemeSelector, ToggleSetting,
};
use components::settings_popups::{AddServerPopup, LoginPopup};
use config::{AppConfig, MusicService};
use dioxus::prelude::*;

#[component]
pub fn Settings(config: Signal<AppConfig>) -> Element {
    let mut show_add_server = use_signal(|| false);
    let mut show_login = use_signal(|| false);

    let mut server_name = use_signal(|| String::new());
    let mut server_url = use_signal(|| String::new());
    let mut server_service = use_signal(|| MusicService::Jellyfin);

    let mut username = use_signal(|| String::new());
    let mut password = use_signal(|| String::new());

    let mut error = use_signal(|| Option::<String>::None);
    let mut login_error = use_signal(|| Option::<String>::None);
    let mut is_loading = use_signal(|| false);

    let handle_add_server = move |_| {
        if !server_url().starts_with("http") {
            error.set(Some(rust_i18n::t!("invalid_server_url").to_string()));
            return;
        }

        let selected_service = server_service();

        let new_server = config::MusicServer::new_with_service(
            if server_name().is_empty() {
                format!("Local {}", selected_service.display_name())
            } else {
                server_name()
            },
            server_url(),
            selected_service,
        );

        config.write().server = Some(new_server);

        server_name.set(String::new());
        server_url.set(String::new());
        server_service.set(MusicService::Jellyfin);
        error.set(None);
        show_add_server.set(false);

        show_login.set(true);
    };

    let handle_login = move |_| {
        if username().is_empty() || password().is_empty() {
            login_error.set(Some(rust_i18n::t!("username_and_password_required").to_string()));
            return;
        }

        if let Some(server) = &config.read().server {
            let service = server.service;
            let server_url = server.url.clone();
            let device_id = config.read().device_id.clone();
            let user = username();
            let pass = password();

            is_loading.set(true);
            login_error.set(None);

            spawn(async move {
                let remote = ProviderClient::new(service, server_url, device_id);
                let result = remote.login(&user, &pass).await;

                is_loading.set(false);

                match result {
                    Ok(session) => {
                        if let Some(server) = config.write().server.as_mut() {
                            server.access_token = Some(session.access_token);
                            server.user_id = Some(session.user_id);
                        }
                        username.set(String::new());
                        password.set(String::new());
                        login_error.set(None);
                        show_login.set(false);
                    }
                    Err(e) => {
                        login_error.set(Some(rust_i18n::t!("login_failed", error = e.to_string()).to_string()));
                    }
                }
            });
        }
    };

    rsx! {
        div { class: "p-8 max-w-4xl",
            h1 { class: "text-3xl font-bold text-white mb-6", "{rust_i18n::t!(\"settings\")}" }

            div { class: "space-y-8",
                section {
                    h2 {
                        class: "text-lg font-semibold text-white/80 mb-4 border-b border-white/5 pb-2",
                        "{rust_i18n::t!(\"general\")}"
                    }

                    div { class: "space-y-4",
                        SettingItem {
                            title: rust_i18n::t!("language").to_string(),
                            control: rsx! {
                                LanguageSelector {
                                    current_language: config.read().language.clone(),
                                    on_change: move |lang: String| {
                                        config.write().language = lang.clone();
                                        rust_i18n::set_locale(&lang);
                                    }
                                }
                            }
                        }

                        SettingItem {
                            title: rust_i18n::t!("appearance").to_string(),
                            control: rsx! {
                                ThemeSelector {
                                    current_theme: config.read().theme.clone(),
                                    on_change: move |theme| {
                                        config.write().theme = theme;
                                    }
                                }
                            }
                        }

                        if !cfg!(target_arch = "wasm32") {
                            SettingItem {
                                title: rust_i18n::t!("music_directory").to_string(),
                                    control: rsx! {
                                    DirectoryPicker {
                                        current_path: config.read().music_directory.display().to_string(),
                                        on_change: move |path| {
                                            config.write().music_directory = path;
                                        }
                                    }
                                }
                            }
                        }

                        SettingItem {
                            title: rust_i18n::t!("media_server").to_string(),
                            control: rsx! {
                                ServerSettings {
                                    server: config.read().server.clone(),
                                    on_add: move |_| show_add_server.set(true),
                                    on_delete: move |_| config.write().server = None,
                                    on_login: move |_| show_login.set(true),
                                }
                            }
                        }
                        if !cfg!(target_arch = "wasm32") {
                            SettingItem {
                                title: rust_i18n::t!("discord_presence").to_string(),
                                    control: rsx! {
                                    DiscordPresenceSettings {
                                        enabled: config.read().discord_presence.unwrap_or(true),
                                        on_change: move |val| config.write().discord_presence = Some(val),
                                    }
                                }
                            }
                        }
                        SettingItem {
                            title: rust_i18n::t!("reduce_animations").to_string(),
                            control: rsx! {
                                ToggleSetting {
                                    enabled: config.read().reduce_animations,
                                    on_change: move |val| config.write().reduce_animations = val,
                                }
                            }
                        }
                        if !cfg!(target_arch = "wasm32") {
                            SettingItem {
                                title: rust_i18n::t!("show_source_toggle").to_string(),
                                    control: rsx! {
                                    ToggleSetting {
                                        enabled: config.read().show_source_toggle,
                                        on_change: move |val| config.write().show_source_toggle = val,
                                    }
                                }
                            }
                        }
                        SettingItem {
                            title: rust_i18n::t!("listenbrainz").to_string(),
                            control: rsx! {
                                MusicBrainzSettings {
                                    current: config.read().musicbrainz_token.clone(),
                                    on_save: move |token: String| {
                                        config.write().musicbrainz_token = token;
                                    },
                                }
                            }
                        }
                        // SettingItem {
                        //     title: "Last.fm",
                        //     description: "Enter you last.fm token".to_string(),
                        //     control: rsx! {
                        //         LastFmSettings {
                        //             current: config.read().lastfm_token.clone(),
                        //             on_save: move |token: String| {
                        //                 config.write().lastfm_token = token;
                        //             },
                        //         }
                        //     }
                        // }
                    }
                }

                section {
                    h2 {
                        class: "text-lg font-semibold text-white/80 mb-4 border-b border-white/5 pb-2",
                        "{rust_i18n::t!(\"theme_editor\")}"
                    }
                    ThemeEditorPage { config, embedded: true }
                }

                if show_add_server() {
                    AddServerPopup {
                        server_name,
                        server_url,
                        server_service,
                        error,
                        on_close: move |_| show_add_server.set(false),
                        on_save: handle_add_server
                    }
                }

                if show_login() {
                    LoginPopup {
                        username,
                        password,
                        service_name: config
                            .read()
                            .server
                            .as_ref()
                            .map(|server| server.service.display_name().to_string())
                            .unwrap_or_else(|| rust_i18n::t!("server").to_string()),
                        error: login_error,
                        loading: is_loading,
                        on_close: move |_| {
                            show_login.set(false);
                            username.set(String::new());
                            password.set(String::new());
                            login_error.set(None);
                        },
                        on_save: handle_login
                    }
                }
            }
        }
    }
}

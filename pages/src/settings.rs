use crate::theme_editor::ThemeEditorPage;
use ::server::provider::ProviderClient;
use ::server::youtube_music;
use components::settings_items::{
    BackBehaviorSelector, DirectoryPicker, DiscordPresenceSettings, LanguageSelector,
    MusicBrainzSettings, ServerSettings, SettingItem, ThemeSelector, ToggleSetting,
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

    let mut ytm_user_code = use_signal(|| Option::<String>::None);
    let mut ytm_verify_url = use_signal(|| Option::<String>::None);
    let mut ytm_error = use_signal(|| Option::<String>::None);
    let mut ytm_loading = use_signal(|| false);

    let handle_ytm_connect = move |_| {
        ytm_loading.set(true);
        ytm_error.set(None);
        ytm_user_code.set(None);
        ytm_verify_url.set(None);

        spawn(async move {
            match youtube_music::start_device_auth().await {
                Ok(auth) => {
                    let device_code = auth.device_code.clone();
                    let interval = auth.interval;
                    ytm_user_code.set(Some(auth.user_code));
                    ytm_verify_url.set(Some(auth.verification_url));
                    ytm_loading.set(false);

                    loop {
                        utils::sleep(std::time::Duration::from_secs(interval.max(5))).await;
                        match youtube_music::poll_device_token(&device_code).await {
                            Ok(Some(tokens)) => {
                                let mut cfg = config.write();
                                cfg.ytm_access_token = Some(tokens.access_token);
                                cfg.ytm_refresh_token = Some(tokens.refresh_token);
                                ytm_user_code.set(None);
                                ytm_verify_url.set(None);
                                break;
                            }
                            Ok(None) => continue,
                            Err(e) => {
                                ytm_error.set(Some(e));
                                ytm_user_code.set(None);
                                ytm_verify_url.set(None);
                                break;
                            }
                        }
                    }
                }
                Err(e) => {
                    ytm_error.set(Some(e));
                    ytm_loading.set(false);
                }
            }
        });
    };

    let handle_add_server = move |_| {
        if !server_url().starts_with("http") {
            error.set(Some(i18n::t("invalid_server_url").to_string()));
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
            login_error.set(Some(i18n::t("username_and_password_required").to_string()));
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
                        login_error.set(Some(i18n::t_with(
                            "login_failed",
                            &[("error", e.to_string())],
                        )));
                    }
                }
            });
        }
    };

    rsx! {
        div { class: "p-8 max-w-4xl",
            h1 { class: "text-3xl font-bold text-white mb-6", "{i18n::t(\"settings\")}" }

            div { class: "space-y-8",
                section {
                    h2 {
                        class: "text-lg font-semibold text-white/80 mb-4 border-b border-white/5 pb-2",
                        "{i18n::t(\"general\")}"
                    }

                    div { class: "space-y-4",
                        SettingItem {
                            title: i18n::t("language").to_string(),
                            control: rsx! {
                                LanguageSelector {
                                    current_language: config.read().language.clone(),
                                    on_change: move |lang: String| {
                                        config.write().language = lang.clone();
                                        i18n::set_locale(&lang);
                                    }
                                }
                            }
                        }

                        SettingItem {
                            title: i18n::t("appearance").to_string(),
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
                                title: i18n::t("music_directory").to_string(),
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
                            title: i18n::t("media_server").to_string(),
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
                                title: i18n::t("discord_presence").to_string(),
                                    control: rsx! {
                                    DiscordPresenceSettings {
                                        enabled: config.read().discord_presence.unwrap_or(true),
                                        on_change: move |val| config.write().discord_presence = Some(val),
                                    }
                                }
                            }
                        }
                        SettingItem {
                            title: i18n::t("reduce_animations").to_string(),
                            control: rsx! {
                                ToggleSetting {
                                    enabled: config.read().reduce_animations,
                                    on_change: move |val| config.write().reduce_animations = val,
                                }
                            }
                        }
                        if !cfg!(target_arch = "wasm32") {
                            SettingItem {
                                title: i18n::t("show_source_toggle").to_string(),
                                    control: rsx! {
                                    ToggleSetting {
                                        enabled: config.read().show_source_toggle,
                                        on_change: move |val| config.write().show_source_toggle = val,
                                    }
                                }
                            }
                        }
                        SettingItem {
                            title: i18n::t("back_behavior").to_string(),
                            control: rsx! {
                                BackBehaviorSelector {
                                    current: config.read().back_behavior,
                                    on_change: move |val| config.write().back_behavior = val,
                                }
                            }
                        }
                        SettingItem {
                            title: i18n::t("listenbrainz").to_string(),
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
                        "YouTube Music"
                    }
                    div { class: "space-y-4",
                        {
                            let ytm_connected = config.read().ytm_access_token.is_some();

                            rsx! {
                                if ytm_connected {
                                    div { class: "flex items-center justify-between",
                                        div { class: "flex items-center gap-3",
                                            div { class: "w-8 h-8 rounded-full bg-red-500/20 flex items-center justify-center",
                                                i { class: "fa-brands fa-youtube text-red-400 text-sm" }
                                            }
                                            span { class: "text-white text-sm font-medium", "YouTube Music connected" }
                                        }
                                        button {
                                            class: "text-xs text-slate-400 hover:text-red-400 transition-colors px-3 py-1.5 rounded-lg hover:bg-white/5",
                                            onclick: move |_| {
                                                let mut cfg = config.write();
                                                cfg.ytm_access_token = None;
                                                cfg.ytm_refresh_token = None;
                                            },
                                            "Disconnect"
                                        }
                                    }
                                } else if ytm_user_code().is_some() {
                                    div { class: "space-y-3",
                                        p { class: "text-slate-400 text-sm", "Visit the URL below and enter this code:" }
                                        div { class: "bg-white/5 rounded-xl p-4 space-y-2 border border-white/10",
                                            if let Some(url) = ytm_verify_url() {
                                                a {
                                                    class: "text-blue-400 text-sm font-medium hover:underline block",
                                                    href: "{url}",
                                                    "{url}"
                                                }
                                            }
                                            if let Some(code) = ytm_user_code() {
                                                div { class: "font-mono text-2xl font-bold text-white tracking-widest text-center py-2",
                                                    "{code}"
                                                }
                                            }
                                        }
                                        p { class: "text-slate-500 text-xs", "Waiting for authorization..." }
                                    }
                                } else {
                                    div { class: "flex items-center justify-between",
                                        p { class: "text-slate-400 text-sm", "Connect your YouTube Music account via Google OAuth." }
                                        button {
                                            class: "flex items-center gap-2 bg-red-500/10 hover:bg-red-500/20 text-red-400 text-sm font-medium px-4 py-2 rounded-xl transition-colors border border-red-500/20 disabled:opacity-50",
                                            disabled: ytm_loading(),
                                            onclick: handle_ytm_connect,
                                            i { class: "fa-brands fa-youtube" }
                                            if ytm_loading() { "Connecting..." } else { "Connect YouTube Music" }
                                        }
                                    }
                                    if let Some(err) = ytm_error() {
                                        p { class: "text-red-400 text-xs mt-2", "{err}" }
                                    }
                                }
                            }
                        }
                    }
                }

                section {
                    h2 {
                        class: "text-lg font-semibold text-white/80 mb-4 border-b border-white/5 pb-2",
                        "{i18n::t(\"theme_editor\")}"
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
                            .unwrap_or_else(|| i18n::t("server").to_string()),
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

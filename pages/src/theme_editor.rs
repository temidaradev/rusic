use config::{AppConfig, CustomTheme};
use dioxus::prelude::*;
use std::collections::HashMap;

const VAR_LABELS: &[&str] = &[
    "bg",
    "raised",
    "surface",
    "text",
    "text-muted",
    "accent",
    "accent-soft",
    "accent-alt",
    "accent-deep",
    "highlight",
    "highlight-dark",
    "progress",
    "danger",
];

const DEFAULT_VARS: &[(&str, &str)] = &[
    ("bg", "#0f0f17"),
    ("raised", "#1a1a2a"),
    ("surface", "#282838"),
    ("text", "#e2e2f0"),
    ("text-muted", "#7878a0"),
    ("accent", "#5f8aff"),
    ("accent-soft", "#8faeff"),
    ("accent-alt", "#3a5fd9"),
    ("accent-deep", "#0a0a1a"),
    ("highlight", "#c77dff"),
    ("highlight-dark", "#9d4edd"),
    ("progress", "#5f8aff"),
    ("danger", "#ff6b6b"),
];

fn default_vars_map() -> HashMap<String, String> {
    DEFAULT_VARS
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

fn get_color_label(key: &str) -> String {
    match key {
        "bg" => rust_i18n::t!("bg").to_string(),
        "raised" => rust_i18n::t!("raised").to_string(),
        "surface" => rust_i18n::t!("surface").to_string(),
        "text" => rust_i18n::t!("text").to_string(),
        "text-muted" => rust_i18n::t!("text-muted").to_string(),
        "accent" => rust_i18n::t!("accent").to_string(),
        "accent-soft" => rust_i18n::t!("accent-soft").to_string(),
        "accent-alt" => rust_i18n::t!("accent-alt").to_string(),
        "accent-deep" => rust_i18n::t!("accent-deep").to_string(),
        "highlight" => rust_i18n::t!("highlight").to_string(),
        "highlight-dark" => rust_i18n::t!("highlight-dark").to_string(),
        "progress" => rust_i18n::t!("progress").to_string(),
        "danger" => rust_i18n::t!("danger").to_string(),
        _ => String::new(),
    }
}

#[component]
pub fn ThemeEditorPage(config: Signal<AppConfig>) -> Element {
    let mut selected_id: Signal<Option<String>> = use_signal(|| None);
    let mut editing_name = use_signal(String::new);
    let mut editing_vars: Signal<HashMap<String, String>> = use_signal(default_vars_map);

    // When selection changes, load the theme into the editor
    use_effect(move || {
        let id = selected_id.read().clone();
        match id {
            Some(ref id) => {
                let cfg = config.read();
                if let Some(ct) = cfg.custom_themes.get(id) {
                    editing_name.set(ct.name.clone());
                    editing_vars.set(ct.vars.clone());
                }
            }
            None => {
                editing_name.set(String::new());
                editing_vars.set(default_vars_map());
            }
        }
    });

    let themes_list: Vec<(String, String)> = {
        let mut v: Vec<(String, String)> = config
            .read()
            .custom_themes
            .iter()
            .map(|(id, ct)| (id.clone(), ct.name.clone()))
            .collect();
        v.sort_by(|a, b| a.1.cmp(&b.1));
        v
    };

    // Live preview: build the preview style string from editing_vars
    let preview_style = {
        let vars = editing_vars.read();
        let bg = vars.get("bg").cloned().unwrap_or_default();
        let raised = vars.get("raised").cloned().unwrap_or_default();
        let surface = vars.get("surface").cloned().unwrap_or_default();
        let text = vars.get("text").cloned().unwrap_or_default();
        let text_muted = vars.get("text-muted").cloned().unwrap_or_default();
        let accent = vars.get("accent").cloned().unwrap_or_default();
        let highlight = vars.get("highlight").cloned().unwrap_or_default();
        let progress = vars.get("progress").cloned().unwrap_or_default();
        let danger = vars.get("danger").cloned().unwrap_or_default();
        format!(
            "--preview-bg:{bg};--preview-raised:{raised};--preview-surface:{surface};\
             --preview-text:{text};--preview-muted:{text_muted};--preview-accent:{accent};\
             --preview-highlight:{highlight};--preview-progress:{progress};--preview-danger:{danger};"
        )
    };

    rsx! {
        div { class: "p-8 max-w-5xl pb-32",
            h1 { class: "text-3xl font-bold text-white mb-6", "{rust_i18n::t!(\"theme_editor\")}" }

            div { class: "flex gap-6",

                // ── Left: saved themes list ──────────────────────────────
                div { class: "w-52 shrink-0 flex flex-col gap-2",
                    button {
                        class: "w-full px-3 py-2 bg-white/10 hover:bg-white/15 rounded text-sm text-white transition-colors text-left",
                        onclick: move |_| selected_id.set(None),
                        "+ {rust_i18n::t!(\"new_theme\")}"
                    }
                    div { class: "space-y-1",
                        for (id, name) in &themes_list {
                            {
                                let id = id.clone();
                                let name = name.clone();
                                let is_active = *selected_id.read() == Some(id.clone());
                                rsx! {
                                    button {
                                        key: "{id}",
                                        class: if is_active {
                                            "w-full text-left px-3 py-2 rounded text-sm bg-white/15 text-white"
                                        } else {
                                            "w-full text-left px-3 py-2 rounded text-sm text-slate-400 hover:bg-white/5 hover:text-white transition-colors"
                                        },
                                        onclick: move |_| selected_id.set(Some(id.clone())),
                                        "{name}"
                                    }
                                }
                            }
                        }
                    }
                }

                // ── Right: editor ────────────────────────────────────────
                div { class: "flex-1 flex flex-col gap-5",

                    // Name input
                    div { class: "bg-white/5 rounded-xl p-5",
                        label { class: "block text-xs text-slate-400 mb-1 uppercase tracking-wider", "{rust_i18n::t!(\"theme_name\")}" }
                        input {
                            class: "bg-white/5 border border-white/10 rounded px-3 py-1.5 text-sm text-white w-full focus:outline-none focus:border-white/30",
                            placeholder: "{rust_i18n::t!(\"my_custom_theme\")}",
                            value: "{editing_name}",
                            oninput: move |e| editing_name.set(e.value()),
                        }
                    }

                    // Color pickers grid
                    div { class: "bg-white/5 rounded-xl p-5",
                        p { class: "text-xs text-slate-400 uppercase tracking-wider mb-4", "{rust_i18n::t!(\"colors\")}" }
                        div { class: "grid grid-cols-2 gap-x-10 gap-y-3",
                            for key in VAR_LABELS {
                                {
                                    let key_str = key.to_string();
                                    let current = editing_vars
                                        .read()
                                        .get(&key_str)
                                        .cloned()
                                        .unwrap_or_else(|| "#000000".to_string());
                                    let label = get_color_label(key);
                                    rsx! {
                                        div { class: "flex items-center justify-between",
                                            span { class: "text-sm text-slate-300", "{label}" }
                                            div { class: "flex items-center gap-2",
                                                input {
                                                    r#type: "color",
                                                    class: "w-8 h-8 rounded cursor-pointer border border-white/10 bg-transparent",
                                                    value: "{current}",
                                                    oninput: move |e| {
                                                        editing_vars.write().insert(key_str.clone(), e.value());
                                                    }
                                                }
                                                span { class: "text-xs text-slate-500 font-mono w-[4.5rem]",
                                                    "{current}"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Live preview
                    div { class: "bg-white/5 rounded-xl p-5",
                        p { class: "text-xs text-slate-400 uppercase tracking-wider mb-3", "{rust_i18n::t!(\"preview\")}" }
                        div {
                            class: "rounded-lg p-4 flex flex-col gap-2",
                            style: "background: var(--preview-bg); {preview_style}",
                            div { class: "flex items-center justify-between",
                                span {
                                    class: "text-sm font-semibold",
                                    style: "color: var(--preview-text)",
                                    "{rust_i18n::t!(\"track_title\")}"
                                }
                                span {
                                    class: "text-xs",
                                    style: "color: var(--preview-muted)",
                                    "3:42"
                                }
                            }
                            div {
                                class: "h-1 rounded-full w-full",
                                style: "background: var(--preview-surface)",
                                div {
                                    class: "h-1 rounded-full w-2/3",
                                    style: "background: var(--preview-progress)"
                                }
                            }
                            div { class: "flex gap-2 mt-1",
                                span {
                                    class: "text-xs px-2 py-0.5 rounded-full",
                                    style: "background: var(--preview-raised); color: var(--preview-accent)",
                                    "{rust_i18n::t!(\"accent\")}"
                                }
                                span {
                                    class: "text-xs px-2 py-0.5 rounded-full",
                                    style: "background: var(--preview-raised); color: var(--preview-highlight)",
                                    "{rust_i18n::t!(\"highlight\")}"
                                }
                                span {
                                    class: "text-xs px-2 py-0.5 rounded-full",
                                    style: "background: var(--preview-raised); color: var(--preview-danger)",
                                    "{rust_i18n::t!(\"danger\")}"
                                }
                            }
                        }
                    }

                    // Actions
                    div { class: "flex gap-3",
                        button {
                            class: "px-4 py-2 bg-indigo-600 hover:bg-indigo-500 rounded text-sm text-white transition-colors",
                            onclick: move |_| {
                                let name = editing_name.read().trim().to_string();
                                if name.is_empty() { return; }
                                let vars = editing_vars.read().clone();
                                let id = selected_id.read().clone().unwrap_or_else(|| {
                                    // Generate a unique id — avoid silently overwriting another theme
                                    let slug = format!("custom-{}", name.to_lowercase().replace(' ', "-"));
                                    let existing = &config.read().custom_themes;
                                    if !existing.contains_key(&slug) {
                                        slug
                                    } else {
                                        let mut n = 1u32;
                                        loop {
                                            let candidate = format!("{slug}-{n}");
                                            if !existing.contains_key(&candidate) {
                                                break candidate;
                                            }
                                            n += 1;
                                        }
                                    }
                                });
                                config.write().custom_themes.insert(id.clone(), CustomTheme { name, vars });
                                selected_id.set(Some(id));
                            },
                            "{rust_i18n::t!(\"save_theme\")}"
                        }
                        if selected_id.peek().is_some() {
                            button {
                                class: "px-4 py-2 bg-red-500/20 hover:bg-red-500/30 rounded text-sm text-red-400 transition-colors",
                                onclick: move |_| {
                                    if let Some(id) = selected_id.write().take() {
                                        let mut cfg = config.write();
                                        cfg.custom_themes.remove(&id);
                                        // If the deleted theme was active, fall back to default
                                        if cfg.theme == id {
                                            cfg.theme = "default".to_string();
                                        }
                                    }
                                },
                                "{rust_i18n::t!(\"delete\")}"
                            }
                        }
                    }
                }
            }
        }
    }
}
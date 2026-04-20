use dioxus::prelude::*;

#[component]
pub fn AddPlaylistPopup(
    playlist_name: Signal<String>,
    error: Signal<Option<String>>,
    on_close: EventHandler<()>,
    on_save: EventHandler<()>,
) -> Element {
    rsx! {
        div {
            class: "overlay",
            onclick: move |_| on_close.call(()),

            div {
                class: "popup",
                onclick: |e| e.stop_propagation(),

                h2 { "{rust_i18n::t!(\"add_playlist\")}" }

                if let Some(err) = error() {
                    p { class: "error", "{err}" }
                }

                input {
                    placeholder: "{rust_i18n::t!(\"playlist_name_placeholder\")}",
                    value: "{playlist_name()}",
                    oninput: move |e| playlist_name.set(e.value()),
                    onkeydown: move |e| e.stop_propagation()
                }

                div { class: "actions",
                    button {
                        onclick: move |_| on_close.call(()),
                        "{rust_i18n::t!(\"cancel\")}"
                    }
                    button {
                        onclick: move |_| on_save.call(()),
                        "{rust_i18n::t!(\"save\")}"
                    }
                }
            }
        }
    }
}

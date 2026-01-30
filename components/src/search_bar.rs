use dioxus::prelude::*;

#[component]
pub fn SearchBar(search_query: Signal<String>) -> Element {
    rsx! {
        div {
            class: "relative max-w-2xl mb-8",
            i { class: "fa-solid fa-magnifying-glass absolute left-4 top-1/2 -translate-y-1/2 text-slate-500" }
            input {
                r#type: "text",
                placeholder: "Search for artists, albums or tracks...",
                class: "w-full bg-white/5 border border-white/10 rounded-full py-3 pl-12 pr-4 text-white focus:outline-none focus:border-white/20 transition-colors",
                value: "{search_query}",
                oninput: move |evt| search_query.set(evt.value())
            }
        }
    }
}

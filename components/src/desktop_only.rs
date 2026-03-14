use dioxus::prelude::*;

#[component]
pub fn DesktopOnly(children: Element) -> Element {
    if !cfg!(any(target_os = "android", target_os = "ios")) {
        children
    } else {
        rsx! { div {} }
    }
}

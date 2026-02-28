use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::fa_solid_icons::FaGear;

const SIDEBAR_CSS: Asset = asset!("/assets/chat/sidebar.css");

#[component]
pub fn Sidebar(on_settings_click: EventHandler<()>) -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: SIDEBAR_CSS }
        div { id: "sidebar",
            // window dragging area
            "data-tauri-drag-region": "true",
            div { class: "avatar", "A" }

            div { class: "spacer" }

            button { class: "settings-btn", onclick: move |_| on_settings_click.call(()),
                Icon { width: 20, height: 20, icon: FaGear }
            }
        }
    }
}

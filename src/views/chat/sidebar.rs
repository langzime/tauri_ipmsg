use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::fa_solid_icons::FaGear;

const SIDEBAR_CSS: Asset = asset!("/assets/chat/sidebar.css");

use crate::models::OnlineUser;

#[component]
pub fn Sidebar(user: Option<OnlineUser>, on_settings_click: EventHandler<()>) -> Element {
    let initial = user.as_ref().and_then(|u| u.name.chars().next()).unwrap_or('?');
    let tooltip = user.as_ref().map(|u| format!("{} ({})", u.name, u.group)).unwrap_or_default();

    rsx! {
        document::Link { rel: "stylesheet", href: SIDEBAR_CSS }
        div { id: "sidebar",
            // window dragging area
            "data-tauri-drag-region": "true",
            div { class: "avatar", title: "{tooltip}", "{initial}" }

            div { class: "spacer" }

            button { class: "settings-btn", onclick: move |_| on_settings_click.call(()),
                Icon { width: 20, height: 20, icon: FaGear }
            }
        }
    }
}

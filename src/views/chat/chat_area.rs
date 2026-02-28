use dioxus::prelude::*;
use crate::models::{ChatMessage, OnlineUser};
use crate::tauri::invoke_no_args;
use super::message_list::MessageList;
use super::input_area::InputArea;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::fa_solid_icons::{FaBell, FaEllipsis, FaEllipsisVertical, FaMagnifyingGlass, FaMinus, FaWindowMaximize, FaXmark};

const CHAT_AREA_CSS: Asset = asset!("/assets/chat/chat_area.css");

#[component]
pub fn ChatArea(
    current: Option<OnlineUser>,
    messages: Vec<ChatMessage>,
) -> Element {
    let platform_resource = use_resource(move || async move {
        invoke_no_args::<String>("get_platform").await.unwrap_or_else(|_| "unknown".to_string())
    });
    let show_controls = platform_resource.cloned().map(|p| p != "macos").unwrap_or(false);

    rsx! {
        document::Link { rel: "stylesheet", href: CHAT_AREA_CSS }
        div { id: "main-panel",
            div { class: "topbar",
                "data-tauri-drag-region": "true",
                {
                    if show_controls {
                        rsx! {
                            div { class: "window-controls",
                                button { class: "win-btn min", onclick: |_| { spawn(async move { let _ = invoke_no_args::<()>("minimize_window").await; }); }, Icon { width: 12, height: 12, icon: FaMinus } }
                                button { class: "win-btn max", onclick: |_| { spawn(async move { let _ = invoke_no_args::<()>("maximize_window").await; }); }, Icon { width: 12, height: 12, icon: FaWindowMaximize } }
                                button { class: "win-btn close", onclick: |_| { spawn(async move { let _ = invoke_no_args::<()>("close_window").await; }); }, Icon { width: 12, height: 12, icon: FaXmark } }
                            }
                        }
                    } else {
                        rsx!({})
                    }
                }
            }
            div { class: "main-header",
                "data-tauri-drag-region": "true",
                h2 { {current.as_ref().map(|c| c.host.clone()).unwrap_or_else(|| "会话".into())} }
                div { class: "tools", span { Icon { width: 20, height: 20, icon: FaEllipsis } } }
            }
            MessageList {
                current: current.clone(),
                messages: messages
            }
            InputArea {
                current: current.clone()
            }
        }
    }
}

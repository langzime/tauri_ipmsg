use dioxus::prelude::*;
use crate::models::OnlineUser;
use crate::tauri::invoke;

const CONV_LIST_CSS: Asset = asset!("/assets/chat/conversation_list.css");

#[component]
pub fn ConversationList(
    conversations: Vec<OnlineUser>,
    unread_counts: std::collections::HashMap<String, u32>,
    active_addr: Option<String>,
    on_select: EventHandler<String>,
    list_w: Signal<i32>,
    on_resize: EventHandler<dioxus::html::geometry::ClientPoint>,
) -> Element {
    let conv_items: Vec<(OnlineUser, u32)> = conversations
        .iter()
        .cloned()
        .map(|conv| {
            let unread = unread_counts.get(&conv.addr.to_string()).copied().unwrap_or(0); 
            (conv, unread)
        })
        .collect();

    rsx! {
        document::Link { rel: "stylesheet", href: CONV_LIST_CSS }
        div { id: "list-panel",
            div { class: "splitter",
                onmousedown: move |ev| {
                    on_resize.call(ev.client_coordinates());
                }
            }
            div {
                class: "list-topbar",
                "data-tauri-drag-region": "true",
             }
            div { class: "list-header",
                input {
                    r#type: "text",
                    placeholder: "搜索",
                }
                button { "+" }
            }
            div { class: "conv-list",
                for (conv, unread) in conv_items.into_iter() {
                    {
                        let addr = conv.addr.to_string();
                        let is_active = active_addr.as_ref().map(|a| a == &addr).unwrap_or(false);
                        rsx! {
                            div {
                                key: "{addr}",
                                class: if is_active { "conv-item active" } else { "conv-item" },
                                onclick: move |_| {
                                    on_select.call(addr.clone());
                                    // clear unread
                                    let addr_clone = addr.clone();
                                    spawn(async move {
                                        use serde::{Serialize, Deserialize};
                                        #[derive(Serialize)]
                                        struct ClearUnreadArgs {
                                            addr: String,
                                        }
                                        let _ = invoke::<()>("clear_unread", ClearUnreadArgs { addr: addr_clone }).await;
                                    });
                                },
                                div { class: "avatar",
                                    {conv.name.chars().next().unwrap_or('C').to_string()}
                                    if unread > 0 && !is_active {
                                        div { class: "avatar-badge" }
                                    }
                                }
                                div { class: "meta",
                                    div { class: "top",
                                        span { class: "name", {conv.name.clone()} }
                                        span { class: "time", "" }
                                    }
                                    div { class: "bottom",
                                        span { class: "preview", {conv.host.clone()} }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

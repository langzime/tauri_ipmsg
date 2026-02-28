use dioxus::prelude::*;
use crate::models::OnlineUser;
use crate::tauri::invoke;

const CONV_LIST_CSS: Asset = asset!("/assets/chat/conversation_list.css");

#[component]
pub fn ConversationList(
    conversations: Vec<OnlineUser>,
    unread_counts: std::collections::HashMap<String, u32>,
    active_idx: usize,
    on_select: EventHandler<usize>,
    list_w: Signal<i32>,
    on_resize: EventHandler<dioxus::html::geometry::ClientPoint>,
) -> Element {
    let conv_items: Vec<(usize, OnlineUser, u32)> = conversations
        .iter()
        .cloned()
        .enumerate()
        .map(|(idx, conv)| {
            let unread = unread_counts.get(&conv.addr.to_string()).copied().unwrap_or(0); 
            (idx, conv, unread)
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
                for (idx, conv, unread) in conv_items.into_iter() {
                    div {
                        class: if active_idx == idx { "conv-item active" } else { "conv-item" },
                        onclick: move |_| {
                            on_select.call(idx);
                            // clear unread
                            spawn(async move {
                                let _ = invoke::<()>("clear_unread", conv.addr.to_string()).await;
                            });
                        },
                        div { class: "avatar",
                            {conv.name.chars().next().unwrap_or('C').to_string()}
                            if unread > 0 && active_idx != idx {
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

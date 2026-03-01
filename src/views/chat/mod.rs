use dioxus::prelude::*;
use crate::models::{ChatMessage, OnlineUser};
use crate::tauri::{invoke_no_args, listen};
use std::collections::HashMap;

pub mod sidebar;
pub mod conversation_list;
pub mod chat_area;
pub mod message_list;
pub mod input_area;

use sidebar::Sidebar;
use conversation_list::ConversationList;
use chat_area::ChatArea;

const COMMON_CSS: Asset = asset!("/assets/chat/common.css");

#[component]
pub fn Chat() -> Element {
    let mut list_w = use_signal(|| 200i32);
    let mut dragging = use_signal(|| false);
    let mut start_x = use_signal(|| 0f64);
    let mut start_w = use_signal(|| 260i32);

    let mut active_conv_addr = use_signal(|| Option::<String>::None);
    
    // Event trigger
    let mut version = use_signal(|| 0u64);
    use_effect(move || {
        let mut _listener = None;
        spawn(async move {
            let res = listen("state-changed", move |_| {
                web_sys::console::log_1(&"State changed event received".into());
                version += 1;
            }).await;

            match res {
                Ok(l) => {
                    web_sys::console::log_1(&"Listener registered successfully".into());
                    _listener = Some(l);
                },
                Err(e) => {
                    web_sys::console::log_1(&format!("Failed to register listener: {}", e).into());
                }
            }
            
            // Keep the listener alive by suspending
            if _listener.is_some() {
                std::future::pending::<()>().await;
            }
        });
    });

    // Resources
    let conversations_resource = use_resource(move || async move {
        let _ = version(); // dependency
        invoke_no_args::<Vec<OnlineUser>>("list_users").await.unwrap_or_default()
    });

    let messages_resource = use_resource(move || async move {
        let _ = version(); // dependency
        web_sys::console::log_1(&"Fetching messages...".into());
        let msgs = invoke_no_args::<Vec<ChatMessage>>("list_messages").await.unwrap_or_default();
        web_sys::console::log_1(&format!("Got {} messages", msgs.len()).into());
        msgs
    });

    let unread_counts_resource = use_resource(move || async move {
        let _ = version(); // dependency
        invoke_no_args::<HashMap<String, u32>>("get_unread_counts").await.unwrap_or_default()
    });

    let self_resource = use_resource(move || async move {
        let _ = version(); // dependency
        invoke_no_args::<Option<OnlineUser>>("get_self").await.unwrap_or_default()
    });

    // Computed data
    let conversations = conversations_resource.cloned().unwrap_or_default();
    let all_msgs = messages_resource.cloned().unwrap_or_default();
    let unread_counts = unread_counts_resource.cloned().unwrap_or_default();
    let self_addr_info = self_resource.cloned().unwrap_or_default();
    let self_addr = self_addr_info.as_ref().map(|u| u.addr);

    let current = if let Some(addr) = active_conv_addr() {
        conversations.iter().find(|c| c.addr.to_string() == addr).cloned()
    } else {
        conversations.first().cloned()
    };
    
    // Auto select first conversation if none selected (with delay for stability)
    let conversations_clone = conversations.clone();
    let mut has_auto_selected = use_signal(|| false);
    let mut is_stable = use_signal(|| false);
    let mut timer_started = use_signal(|| false);

    use_effect(move || {
        // Start timer once
        if !*timer_started.peek() {
            timer_started.set(true);
            spawn(async move {
                gloo_timers::future::TimeoutFuture::new(500).await;
                is_stable.set(true);
            });
        }
        
        // Auto select logic when stable
        if *is_stable.read() && !*has_auto_selected.peek() {
            if let Some(first) = conversations_clone.first() {
                if active_conv_addr.peek().is_none() {
                    active_conv_addr.set(Some(first.addr.to_string()));
                }
                has_auto_selected.set(true);
            }
        }
    });

    let view_msgs: Vec<ChatMessage> = if let Some(curr) = &current {
        all_msgs
            .into_iter()
            .filter(|m| {
                if let Some(sa) = self_addr {
                    if curr.addr == sa {
                        m.from == sa && m.to == sa
                    } else {
                        (m.from == curr.addr && m.to == sa) || (m.to == curr.addr && m.from == sa)
                    }
                } else {
                    m.from == curr.addr || m.to == curr.addr
                }
            })
            .collect()
    } else {
        Vec::new()
    };

    rsx! {
        document::Link { rel: "stylesheet", href: COMMON_CSS }

        div {
            id: "chat-app",
            style: format!("grid-template-columns: {}px {}px 1fr", 64, list_w()),
            onmousemove: move |ev| {
                if dragging() {
                    let current_x = ev.client_coordinates().x;
                    let diff = current_x - start_x();
                    let new_w = start_w() + diff as i32;
                    if new_w > 150 && new_w < 500 {
                        list_w.set(new_w);
                    }
                }
            },
            onmouseup: move |_| {
                dragging.set(false);
            },
            Sidebar {
                user: self_addr_info,
                on_settings_click: move |_| {
                    spawn(async move {
                        let _ = invoke_no_args::<()>("open_settings_window").await;
                    });
                }
            }

            ConversationList {
                conversations: conversations,
                unread_counts: unread_counts,
                active_addr: active_conv_addr(),
                list_w: list_w,
                on_select: move |addr| active_conv_addr.set(Some(addr)),
                on_resize: move |pos: dioxus::html::geometry::ClientPoint| {
                    dragging.set(true);
                    start_x.set(pos.x);
                    start_w.set(list_w());
                }
            }

            ChatArea {
                current: current,
                messages: view_msgs
            }
        }
    }
}

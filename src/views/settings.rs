use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::fa_solid_icons::FaXmark;
use crate::tauri::{invoke, invoke_no_args};
use serde::{Deserialize, Serialize};

const SETTINGS_CSS: Asset = asset!("/assets/chat/settings_modal.css");

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
struct UserConfig {
    username: String,
    group: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
struct AppConfig {
    user: UserConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            user: UserConfig {
                username: "".to_string(),
                group: "".to_string(),
            },
        }
    }
}

#[derive(Serialize)]
struct SaveSettingsArgs {
    username: String,
    group: String,
}

#[component]
pub fn Settings() -> Element {
    let mut config = use_signal(|| AppConfig::default());
    let platform = use_resource(|| async move {
        invoke_no_args::<String>("get_platform").await.unwrap_or_default()
    });

    let p = platform.value().cloned().unwrap_or_default();
    let is_macos = p == "macos";

    use_effect(move || {
        spawn(async move {
            let _ = invoke_no_args::<()>("show_window").await;
            if let Ok(c) = invoke_no_args::<AppConfig>("get_config").await {
                config.set(c);
            }
        });
    });

    rsx! {
        document::Link { rel: "stylesheet", href: SETTINGS_CSS }
        div { class: "modal-content",
            div { class: if is_macos { "modal-header macos" } else { "modal-header windows" }, "data-tauri-drag-region": "true",
                if is_macos {
                    button { class: "close-btn macos", onclick: move |_| {
                        spawn(async move {
                            let _ = invoke_no_args::<()>("close_sub_window").await;
                        });
                    },
                        Icon { width: 10, height: 10, icon: FaXmark }
                    }
                    h3 { "设置" }
                    div { class: "spacer", style: "width: 12px;" } // Placeholder to balance layout
                } else {
                    h3 { "设置" }
                    button { class: "close-btn windows", onclick: move |_| {
                        spawn(async move {
                            let _ = invoke_no_args::<()>("close_sub_window").await;
                        });
                    },
                        Icon { width: 16, height: 16, icon: FaXmark }
                    }
                }
            }
            div { class: "modal-body",
                div { class: "setting-item",
                    label { "用户名" }
                    input { 
                        type: "text", 
                        value: "{config().user.username}",
                        oninput: move |evt| {
                            let mut c = config();
                            c.user.username = evt.value();
                            config.set(c);
                        }
                    }
                }
                div { class: "setting-item",
                    label { "组名" }
                    input { 
                        type: "text", 
                        value: "{config().user.group}",
                        oninput: move |evt| {
                            let mut c = config();
                            c.user.group = evt.value();
                            config.set(c);
                        }
                    }
                }
            }
            div { class: "modal-footer",
                button { class: "btn-primary", onclick: move |_| {
                    let c = config();
                    spawn(async move {
                        let args = SaveSettingsArgs {
                            username: c.user.username.clone(),
                            group: c.user.group.clone(),
                        };
                        let _ = invoke::<()>("save_settings", args).await;
                        let _ = invoke_no_args::<()>("close_sub_window").await;
                    });
                }, "确定" }
            }
        }
    }
}

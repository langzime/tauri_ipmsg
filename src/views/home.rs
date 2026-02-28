use dioxus::prelude::*;
use crate::views::Chat;

#[component]
pub fn Home() -> Element {
    rsx! {
        Chat {}
    }
}

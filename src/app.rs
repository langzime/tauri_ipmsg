#![allow(non_snake_case)]

use dioxus::prelude::*;
use crate::views::{Home, Settings};

#[derive(Clone, Routable, Debug, PartialEq)]
enum Route {
    #[route("/")]
    Home {},
    #[route("/settings")]
    Settings {},
}

pub fn App() -> Element {
    rsx! {
        Router::<Route> {}
    }
}

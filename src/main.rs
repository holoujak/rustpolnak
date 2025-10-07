#![allow(dead_code)]

use dioxus::desktop::{Config, LogicalSize, WindowBuilder};
use dioxus::prelude::*;
use tracing::Level;

use crate::components::app::App;

mod components;
mod config;
mod race;
mod restclient;
mod rfid_reader;
mod sort_table;
mod sorter;

const MAIN_CSS: Asset = asset!("/assets/main.css");
const BOOTSTRAP_CSS: Asset = asset!("/assets/bootstrap.css");

fn appconfig_default() -> Config {
    Config::default().with_window(
        WindowBuilder::new()
            .with_maximized(false)
            .with_title("rustpolnak")
            .with_min_inner_size(LogicalSize::new(1280, 768)),
    )
}

#[cfg(debug_assertions)]
fn appconfig() -> Config {
    appconfig_default()
}

#[cfg(not(debug_assertions))]
fn appconfig() -> Config {
    appconfig_default()
        .with_menu(None)
        .with_disable_context_menu(true)
}

fn main() {
    dioxus_logger::init(Level::INFO).expect("logger failed to init");
    let config = config::load_config();
    LaunchBuilder::new()
        .with_cfg(appconfig())
        .with_context(config)
        .launch(Window);
}

#[component]
fn Window() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: BOOTSTRAP_CSS }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        App {}
    }
}

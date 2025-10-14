#![allow(dead_code)]

use dioxus::desktop::{Config, WindowBuilder};
use dioxus::prelude::*;
use tracing::Level;

use crate::components::app::App;
use crate::restclient::RaceRestAPI;

mod components;
mod config;
mod printer;
mod race;
mod race_events;
mod restclient;
mod rfid_reader;

const MAIN_CSS: Asset = asset!("/assets/main.css");
const BOOTSTRAP_CSS: Asset = asset!("/assets/bootstrap.css");

fn appconfig_default() -> Config {
    Config::default().with_window(
        WindowBuilder::new()
            .with_maximized(false)
            .with_title("rustpolnak")
            .with_maximized(true),
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
    let restapi = RaceRestAPI::new(&config.api.url, &config.api.username, &config.api.token);

    LaunchBuilder::new()
        .with_cfg(appconfig())
        .with_context(config)
        .with_context(restapi)
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

#![allow(dead_code)]

use dioxus::desktop::{Config, LogicalSize, WindowBuilder};
use dioxus::prelude::*;

use crate::restclient::RaceRestAPI;
use crate::restclient::{Race, RaceField};
use crate::sort_table::Th;
use crate::sorter::Sorter;

mod restclient;
mod sort_table;
mod sorter;

const MAIN_CSS: Asset = asset!("/assets/main.css");

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
    LaunchBuilder::new().with_cfg(appconfig()).launch(App);
}

#[component]
fn App() -> Element {
    use_context_provider(|| {
        let server_url: &'static str = "http://localhost:8000";
        RaceRestAPI::new(server_url, "username", "password")
    });

    let races = use_resource(move || async move {
        let api = use_context::<RaceRestAPI>();
        api.races().await
    });

    let sorter = use_signal(|| Sorter::<RaceField>::new(RaceField::Id));

    rsx! {
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        match &*races.read() {
            Some(Ok(races)) => {
                let mut sorted = (*races).clone();
                let field = sorter.read().active;
                sorted.sort_by(|a, b| sorter.read().cmp_by(a, b, field, Race::cmp_by));

                rsx! {
                    table {
                        thead {
                            tr {
                                Th { sorter, field: RaceField::Id, "ID" }
                                Th { sorter, field: RaceField::Name, "Name" }
                                Th { sorter, field: RaceField::DateOfEvent, "Date" }
                            }
                        }
                        tbody {
                            for race in sorted.iter() {
                                tr {
                                    td { "{race.id}" }
                                    td { "{race.name}" }
                                    td { "{race.date_of_event}" }
                                }
                            }
                        }
                    }
                }
            },
            Some(Err(err)) => rsx! {
                div {"{err:?}"}
            },
            _ => rsx!{}
        }
    }
}

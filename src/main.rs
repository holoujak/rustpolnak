#![allow(dead_code)]

use dioxus::desktop::{Config, LogicalSize, WindowBuilder};
use dioxus::prelude::*;
use std::collections::HashSet;

use crate::restclient::RaceRestAPI;
use crate::restclient::{Category, Racer, RacerField};
use crate::sort_table::Th;
use crate::sorter::Sorter;

mod restclient;
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
    LaunchBuilder::new().with_cfg(appconfig()).launch(App);
}

#[component]
fn App() -> Element {
    use_context_provider(|| {
        let server_url: &'static str = "http://localhost:8000";
        RaceRestAPI::new(server_url, "username", "password")
    });

    let selected_race_id = use_signal(|| Option::<u32>::None);

    rsx! {
        document::Link { rel: "stylesheet", href: BOOTSTRAP_CSS }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        div {
            RacesList{ selected_race_id }
            if let Some(race_id) = *selected_race_id.read() {
                Registrations{ race_id }
            }

        }
    }
}

#[component]
fn RacesList(selected_race_id: Signal<Option<u32>>) -> Element {
    let races = use_resource(move || async move {
        let api = use_context::<RaceRestAPI>();
        api.races().await
    });

    rsx! {
        match &*races.read() {
            Some(Ok(races)) => rsx! {
                select {
                    class: "form-select mb-1",
                    onchange: move |e| {
                        let val = e.value().parse::<u32>().ok();
                        selected_race_id.set(val);
                    },
                    option { disabled: true, selected: true, "Select race" }
                    for race in races.iter() {
                        option { value: "{race.id}", "{race.name}" }
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

#[component]
fn Registrations(race_id: ReadOnlySignal<u32>) -> Element {
    let registrations = use_resource(move || async move {
        let api = use_context::<RaceRestAPI>();
        api.registrations(*race_id.read()).await
    });

    let selected_category_id = use_signal(|| Option::<u32>::None);
    let sorter = use_signal(|| Sorter::<RacerField>::new(RacerField::StartNumber));

    let x = match &*registrations.read() {
        Some(Ok(racers)) => {
            let mut sorted = (*racers).clone();
            let field = sorter.read().active;
            sorted.sort_by(|a, b| sorter.read().cmp_by(a, b, field, Racer::cmp_by));

            rsx! {
                table {
                    class: "table table-striped table-hover table-sm",
                    thead {
                        class: "table-dark",
                        tr {
                            Th { sorter, field: RacerField::StartNumber, "Start number" }
                            Th { sorter, field: RacerField::FirstName, "First name" }
                            Th { sorter, field: RacerField::LastName, "Last name" }
                            Th { sorter, field: RacerField::Track, "Track" }
                            th { CategoriesList { racers: racers.clone(), selected_category_id: selected_category_id } }
                        }
                    }
                    tbody {
                        for racer in sorted.iter() {
                            if (*selected_category_id.read()).is_none_or(|cat_id|
                                racer.categories.iter().any(|c| c.id == cat_id)
                            ) {
                                tr {
                                    td { "{racer.start_number.unwrap_or(0)}" }
                                    td { "{racer.first_name}" }
                                    td { "{racer.last_name}" }
                                    td { "{racer.track.name}" }
                                    td {
                                        for category in racer.categories.clone() {
                                            "{category.name} "
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Some(Err(err)) => rsx! {
            div {"{err:?}"}
        },
        _ => rsx! {},
    };
    x
}

#[component]
fn CategoriesList(racers: Vec<Racer>, selected_category_id: Signal<Option<u32>>) -> Element {
    let mut categories: HashSet<Category> = HashSet::new();

    for racer in racers {
        for category in racer.categories.into_iter() {
            categories.insert(category);
        }
    }

    let mut sorted_categories: Vec<_> = categories.iter().cloned().collect();
    sorted_categories.sort_by(|a, b| a.name.cmp(&b.name));

    rsx! {
        select {
            onchange: move |e| {
                let val = e.value().parse::<u32>().ok();
                selected_category_id.set(val);
            },
            option { disabled: false, selected: true, "All" }
            for c in sorted_categories.iter() {
                option { value: "{c.id}", "{c.name}" }
            }
        }
    }
}

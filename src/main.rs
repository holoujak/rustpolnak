#![allow(dead_code)]

use chrono::{DateTime, Local, NaiveTime, TimeZone};
use dioxus::desktop::{Config, LogicalSize, WindowBuilder};
use dioxus::logger::tracing::warn;
use dioxus::prelude::*;
use tracing::Level;

use crate::race::{Race, RacerField};
use crate::restclient::RaceRestAPI;
use crate::sort_table::Th;
use crate::sorter::Sorter;

mod config;
mod race;
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
    dioxus_logger::init(Level::INFO).expect("logger failed to init");
    let config = config::load_config();
    LaunchBuilder::new()
        .with_cfg(appconfig())
        .with_context(config)
        .launch(App);
}

#[component]
fn App() -> Element {
    let config: config::Config = use_context();
    use_context_provider(|| {
        RaceRestAPI::new(&config.api.url, &config.api.username, &config.api.token)
    });

    let selected_race =
        use_signal(|| Option::<Result<race::Race, Box<dyn std::error::Error>>>::None);

    rsx! {
        document::Link { rel: "stylesheet", href: BOOTSTRAP_CSS }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        div {
            RacesList { selected_race }
            match &*selected_race.read() {
                Some(Ok(race)) => rsx! {
                    RaceComponent { race: race.clone() }
                },
                Some(Err(err)) => rsx! {
                "{err:#?}"
                },
                None => rsx! {},
            }
        }
    }
}

#[component]
fn RacesList(
    selected_race: Signal<Option<Result<race::Race, Box<dyn std::error::Error>>>>,
) -> Element {
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
                        let race_id = e.value().parse::<u32>().ok().unwrap();
                        spawn(async move {
                            let race = race::Race::load(use_context::<RaceRestAPI>(), race_id).await;
                            selected_race.set(Some(race));
                        });
                    },
                    option { disabled: true, selected: true, "Select race" }
                    for race in races.iter() {
                        option { value: "{race.id}", "{race.name}" }
                    }
                }
            },
            Some(Err(err)) => rsx! {
                div { "{err:?}" }
            },
            _ => rsx! {},
        }
    }
}

#[component]
fn TrackStart(track: String) -> Element {
    let mut start: Signal<Option<DateTime<Local>>> = use_signal(|| None);

    rsx! {
        div { class: "input-group", style: "width: 220px",
            span { class: "input-group-text", style: "width: 80px", "{track}" }
            input {
                class: "form-control form-control-sm",
                r#type: "time",
                value: match *start.read() {
                    Some(start) => start.format("%H:%M:%S").to_string(),
                    None => "".to_string(),
                },
                oninput: move |event| {
                    match NaiveTime::parse_from_str(&event.value(), "%H:%M:%S") {
                        Ok(time) => {
                            match Local
                                .from_local_datetime(&(Local::now().date_naive().and_time(time)))
                                .single()
                            {
                                Some(local_dt) => {
                                    *start.write() = Some(local_dt);
                                }
                                None => {
                                    warn! {
                                        "Failed to create datetime"
                                    }
                                }
                            }
                        }
                        Err(err) => {
                            warn!("Failed to parse: {err:?}");
                        }
                    }
                },
            }
            button {
                class: "btn btn-success",
                onclick: move |_| { *start.write() = Some(Local::now()) },
                "Start"
            }
        }
    }
}

#[component]
fn RaceComponent(race: Race) -> Element {
    rsx! {
        div { class: "d-flex flex-row column-gap-1 mb-1",
            for track in race.clone().tracks {
                TrackStart { track: track.clone() }
            }
        }
        Registrations { race: race.clone() }
        ManualStartNumberInput {}
    }
}

#[component]
fn Registrations(race: race::Race) -> Element {
    let selected_category_id = use_signal(|| Option::<String>::None);
    let sorter = use_signal(|| Sorter::<RacerField>::new(RacerField::StartNumber));

    let mut sorted = race.racers.clone();
    let field = sorter.read().active;
    sorted.sort_by(|a, b| sorter.read().cmp_by(a, b, field, race::Racer::cmp_by));

    rsx! {
        table { class: "table table-striped table-hover table-sm",
            thead { class: "table-dark",
                tr {
                    Th { sorter, field: RacerField::StartNumber, "Start number" }
                    Th { sorter, field: RacerField::FirstName, "First name" }
                    Th { sorter, field: RacerField::LastName, "Last name" }
                    Th { sorter, field: RacerField::Track, "Track" }
                    th {
                        CategoriesList {
                            categories: race.categories,
                            selected_category_id,
                        }
                    }
                }
            }
            tbody {
                for racer in sorted.iter() {
                    if (selected_category_id.read().clone())
                        .is_none_or(|cat_id| racer.categories.contains(&cat_id))
                    {
                        tr {
                            td { "{racer.start_number}" }
                            td { "{racer.first_name}" }
                            td { "{racer.last_name}" }
                            td { "{racer.track}" }
                            td {
                                for category in racer.categories.clone() {
                                    "{category} "
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn CategoriesList(
    categories: Vec<String>,
    selected_category_id: Signal<Option<String>>,
) -> Element {
    let mut sorted_categories: Vec<_> = categories.to_vec();
    sorted_categories.sort();

    rsx! {
        select {
            onchange: move |e| {
                let val = e.value().parse::<String>().ok();
                selected_category_id
                    .set(if val == Some("All".to_string()) { None } else { val });
            },
            option { disabled: false, selected: true, "All" }
            for c in sorted_categories.iter() {
                option { value: "{c}", "{c}" }
            }
        }
    }
}

#[component]
fn ManualStartNumberInput() -> Element {
    let mut start_number = use_signal(|| "".to_string());

    rsx! {
        form {
            onsubmit: move |event| {
                event.prevent_default();
                println!("Enter pressed: {}", start_number.read());
                start_number.set(String::from(""));
            },
            input {
                class: "form-control",
                placeholder: "Start number",
                r#type: "number",
                value: start_number,
                onkeydown: move |event| {
                    let key = event.key().to_string();
                    if key.chars().all(|c| c.is_ascii_digit()) {
                        let current_start_number = start_number.read().clone();
                        start_number.set(current_start_number + &key);
                    } else if event.key() != Key::Enter {
                        event.prevent_default();
                        start_number.set(String::from(""));
                    }
                },
            }
        }
    }
}

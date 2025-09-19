#![allow(dead_code)]

use dioxus::prelude::*;
use maplit::hashmap;
use table_rs::dioxus::table::Table;
use table_rs::dioxus::types::Column;

use crate::restclient::RaceRestAPI;

mod restclient;

const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    dioxus::launch(App);
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

    let columns = vec![
        Column {
            id: "id",
            header: "ID",
            sortable: true,
            ..Default::default()
        },
        Column {
            id: "name",
            header: "Name",
            sortable: true,
            ..Default::default()
        },
        Column {
            id: "date_of_event",
            header: "Date",
            sortable: true,
            ..Default::default()
        },
    ];

    rsx! {
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        "Vyhlal!",

        match &*races.read() {
            Some(Ok(races)) => {
                let data: Vec<_> = races.iter().map(|race| {
                    hashmap! {
                        "id" => race.id.to_string(),
                        "name" => race.name.clone(),
                        "date_of_event" => race.date_of_event.format("%d.%m.%Y").to_string(),
                    }
                }).collect();

                rsx! {
                    Table {
                        columns: columns,
                        data: data,
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

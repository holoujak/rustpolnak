#![allow(dead_code)]

use dioxus::prelude::*;

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

    rsx! {
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        "Vyhlal!",

        match &*races.read() {
            Some(Ok(races)) => {
                rsx! {
                    table {
                        thead {
                            tr {
                                th { "Id" }
                                th { "Name" }
                                th { "Date" }
                            }
                        }
                        tbody {
                            for race in races.iter() {
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

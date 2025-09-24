#![allow(dead_code)]

use dioxus::prelude::*;

use crate::restclient::RaceRestAPI;
use crate::restclient::{sort_races, RaceSortKey, SortOrder};

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

    let mut sort_key = use_signal(|| RaceSortKey::Id);
    let mut sort_order = use_signal(|| SortOrder::Asc);

    rsx! {
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        "Vyhlal!",

        match &*races.read() {
            Some(Ok(races)) => {
                let mut sorted = (*races).clone();
                sort_races(&mut sorted, *sort_key.read(), *sort_order.read());

                rsx! {
                    table {
                        thead {
                            tr {
                                th {
                                    onclick: move |_| {
                                        if *sort_key.read() == RaceSortKey::Id {
                                            let current = *sort_order.read();   // immutable borrow se hned uvolní
                                            let new_value = match current {
                                                SortOrder::Asc => SortOrder::Desc,
                                                SortOrder::Desc => SortOrder::Asc,
                                            };
                                            sort_order.set(new_value);          // teď už je volný pro mutable borrow
                                        } else {
                                            sort_key.set(RaceSortKey::Id);
                                            sort_order.set(SortOrder::Asc);
                                        }
                                    },
                                    "Id"
                                }
                                th {
                                    onclick: move |_| {
                                        if *sort_key.read() == RaceSortKey::Name {
                                            let current = *sort_order.read();   // immutable borrow se hned uvolní
                                            let new_value = match current {
                                                SortOrder::Asc => SortOrder::Desc,
                                                SortOrder::Desc => SortOrder::Asc,
                                            };
                                            sort_order.set(new_value);          // teď už je volný pro mutable borrow
                                        } else {
                                            sort_key.set(RaceSortKey::Name);
                                            sort_order.set(SortOrder::Asc);
                                        }
                                    },
                                    "Name"
                                }
                                th {
                                    onclick: move |_| {
                                        if *sort_key.read() == RaceSortKey::Date {
                                            let current = *sort_order.read();   // immutable borrow se hned uvolní
                                            let new_value = match current {
                                                SortOrder::Asc => SortOrder::Desc,
                                                SortOrder::Desc => SortOrder::Asc,
                                            };
                                            sort_order.set(new_value);          // teď už je volný pro mutable borrow
                                        } else {
                                            sort_key.set(RaceSortKey::Date);
                                            sort_order.set(SortOrder::Asc);
                                        }
                                    },
                                    "Date"
                                }
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

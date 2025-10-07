use std::error::Error;

use dioxus::prelude::*;

use crate::{race::Race, restclient::RaceRestAPI};

#[component]
pub fn RacesList(
    selected_race: Signal<Option<Result<Race, Box<dyn std::error::Error>>>>,
) -> Element {
    let races: Resource<Result<Vec<crate::restclient::Race>, Box<dyn Error>>> =
        use_resource(move || async move {
            let api = use_context::<RaceRestAPI>();
            let mut races = api.races().await?;
            races.sort_by(|a, b| b.date_of_event.cmp(&a.date_of_event));
            Ok(races)
        });

    rsx! {
        match &*races.read() {
            Some(Ok(races)) => rsx! {
                select {
                    class: "form-select mb-1",
                    onchange: move |e| {
                        let race_id = e.value().parse::<u32>().ok().unwrap();
                        spawn(async move {
                            let race = Race::load(use_context::<RaceRestAPI>(), race_id).await;
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

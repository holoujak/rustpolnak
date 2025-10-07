use std::error::Error;

use dioxus::prelude::*;

use crate::{race::Race, restclient::RaceRestAPI};

type SignalRace = Signal<Option<Result<Race, Box<dyn std::error::Error>>>>;

async fn load_race(mut selected_race: SignalRace, id: u32) {
    selected_race.set(None);
    let race = Race::load(use_context::<RaceRestAPI>(), id).await;
    selected_race.set(Some(race));
}

#[component]
pub fn RacesList(selected_race: SignalRace) -> Element {
    let races: Resource<Result<Vec<crate::restclient::Race>, Box<dyn Error>>> =
        use_resource(move || async move {
            let api = use_context::<RaceRestAPI>();
            let mut races = api.races().await?;
            races.sort_by(|a, b| b.date_of_event.cmp(&a.date_of_event));

            if let Some(earliest_race) = races.first() {
                load_race(selected_race, earliest_race.id).await;
            }
            Ok(races)
        });

    rsx! {
        match &*races.read() {
            Some(Ok(races)) => rsx! {
                select {
                    class: "form-select mb-1",
                    onchange: move |e| {
                        e.prevent_default();
                        let race_id = e.value().parse::<u32>().ok().unwrap();
                        spawn(async move {
                            load_race(selected_race, race_id).await;
                        });
                    },
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

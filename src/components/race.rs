use dioxus::prelude::*;

use crate::{
    components::{
        manual_start_number_input::ManualStartNumberInput, racers::Racers, track_start::TrackStart,
        upload_results::UploadResults,
    },
    race::Race,
};

#[component]
pub fn RaceComponent(race: Race) -> Element {
    rsx! {
        div { class: "d-flex flex-row column-gap-1 mb-1",
            for track in race.clone().tracks {
                TrackStart { track: track.clone() }
            }
            UploadResults { race: race.clone() }
        }
        Racers { race: race.clone() }
        ManualStartNumberInput {}
    }
}

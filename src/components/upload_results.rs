use dioxus::prelude::*;
use tracing::{error, info};

use crate::{
    race::{Race, Racer},
    restclient::{RaceRestAPI, RacerResult},
};

enum SubmitState {
    Idle,
    Submitting,
    Success,
    Error,
}

fn racer_to_result(racer: &Racer) -> Option<RacerResult> {
    match (racer.start, racer.finish) {
        (Some(start_time), Some(finish_time)) => Some(RacerResult {
            registration_id: racer.id,
            start_time,
            finish_time,
        }),
        (Some(_start_time), None) => {
            error!("Skipping submit for {}, no finish time", racer.start_number);
            None
        }
        (None, Some(_finish_time)) => {
            error!("Skipping submit for {}, no start time", racer.start_number);
            None
        }
        (None, None) => {
            error!(
                "Skipping submit for {}, no start and no finish time",
                racer.start_number
            );
            None
        }
    }
}

fn upload_results(race: &Race, mut state: Signal<SubmitState>, api: RaceRestAPI) {
    let results: Vec<_> = race.racers.iter().filter_map(racer_to_result).collect();
    if results.is_empty() {
        error!("There are no results!");
        state.set(SubmitState::Error);
    } else {
        state.set(SubmitState::Submitting);
        let id = race.id;
        spawn(async move {
            state.set(match api.results(id, results).await {
                Ok(_) => {
                    info!("Results succesfully uploaded");
                    SubmitState::Success
                }
                Err(err) => {
                    error!("Uploading of results failed: {err:?}");
                    SubmitState::Error
                }
            })
        });
    }
}

#[component]
pub fn UploadResults(race: Race) -> Element {
    let api = use_context::<RaceRestAPI>();
    let upload_state = use_signal(|| SubmitState::Idle);

    let classes = match *upload_state.read() {
        SubmitState::Success => "btn-success",
        SubmitState::Submitting => "btn-warning",
        SubmitState::Error => "btn-danger",
        _ => "",
    };

    rsx! {
        button {
            class: ["btn btn-primary", classes].join(" "),
            onclick: move |_evt| { upload_results(&race, upload_state, api.clone()) },
            dangerous_inner_html: iconify::svg!("mdi:upload"),
        }
    }
}

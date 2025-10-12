use chrono::{DateTime, Utc};
use dioxus::prelude::*;

use crate::components::{app::Action, time_input::TimeInput};

#[component]
pub fn TrackStart(track: String) -> Element {
    let mut start: Signal<Option<DateTime<Utc>>> = use_signal(|| None);
    let mut editing = use_signal(|| false);
    let track2 = track.clone();

    rsx! {
        div { class: "input-group", style: "width: 400px",
            span {
                class: "input-group-text justify-content-end",
                style: "width: 150px",
                "{track}"
            }
            TimeInput {
                time: start,
                editing,
                onsave: move |time| {
                    start.set(Some(time));
                    use_coroutine_handle::<Action>().send(Action::Start(track.clone(), time));
                },
            }
            button {
                class: "btn",
                class: if editing() { "btn-danger" } else { "btn-outline-secondary" },
                onclick: move |_evt| {
                    if *editing.read() {
                        editing.set(false);
                    } else {
                        editing.set(true);
                    }
                },
                dangerous_inner_html: if editing() { iconify::svg!("mdi:times") } else { iconify::svg!("mdi:edit") },
            }
            button {
                class: "btn btn-success",
                onclick: move |_| {
                    start.set(Some(Utc::now()));
                    use_coroutine_handle::<Action>().send(Action::Start(track2.clone(), Utc::now()));
                },
                "Start"
            }
        }
    }
}

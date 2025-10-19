use std::time::Duration;

use crate::{race::TrackStats, time_utils::format_time_delta_secs};
use chrono::Utc;
use dioxus::prelude::*;

use crate::components::{app::Action, time_input::TimeInput};

#[component]
pub fn TrackStart(track: TrackStats) -> Element {
    let mut editing = use_signal(|| false);
    let track2 = track.track.clone();
    let time_since_start = use_signal(|| None);

    use_hook(move || {
        spawn({
            to_owned![time_since_start];
            async move {
                loop {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    time_since_start.set(
                        track
                            .start
                            .map(|start| Some(Utc::now().signed_duration_since(start))),
                    );
                }
            }
        });
    });

    rsx! {
        div { class: "input-group", style: "width: 580px",
            span {
                class: "input-group-text justify-content-end",
                style: "width: 150px",
                "{track.track}"
            }
            span {
                class: "input-group-text justify-content-end",
                style: "width: 90px",
                "{track.finished}/{track.racers}"
            }
            span { class: "input-group-text", style: "width: 90px",
                if let Some(time_since_start) = time_since_start() {
                    "{format_time_delta_secs(time_since_start)}"
                }
            }
            TimeInput {
                time: track.start,
                editing,
                span_class: "input-group-text flex-grow-1",
                onsave: move |time_option| {
                    if let Some(time) = time_option {
                        use_coroutine_handle::<Action>()
                            .send(Action::Start(track.track.clone(), time));
                    }
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
                    use_coroutine_handle::<Action>().send(Action::Start(track2.clone(), Utc::now()));
                },
                "Start"
            }
        }
    }
}

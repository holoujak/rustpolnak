use chrono::{DateTime, Local, NaiveTime, TimeZone};
use dioxus::prelude::*;
use tracing::warn;

use crate::components::app::Action;

#[component]
pub fn TrackStart(track: String) -> Element {
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
                onclick: move |_| {
                    *start.write() = Some(Local::now());
                    use_coroutine_handle::<Action>().send(Action::Start(track.clone()));
                },
                "Start"
            }
        }
    }
}

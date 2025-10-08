use chrono::{DateTime, Local, NaiveTime, TimeZone, Utc};
use dioxus::prelude::*;
use tracing::warn;

use crate::components::app::Action;

fn parse_time(str: &str) -> Option<DateTime<Utc>> {
    match NaiveTime::parse_from_str(str, "%H:%M:%S") {
        Ok(naive_time) => {
            let time = Local
                .from_local_datetime(&(Local::now().date_naive().and_time(naive_time)))
                .single();

            time.map(|time| time.to_utc())
        }
        Err(err) => {
            warn!("Failed to parse: {err:?}");
            None
        }
    }
}

#[component]
pub fn TrackStart(track: String) -> Element {
    let mut start: Signal<Option<DateTime<Utc>>> = use_signal(|| None);

    rsx! {
        div { class: "input-group", style: "width: 220px",
            span { class: "input-group-text", style: "width: 80px", "{track}" }
            input {
                class: "form-control form-control-sm",
                r#type: "time",
                value: match *start.read() {
                    Some(start) => start.with_timezone(&Local).format("%H:%M:%S").to_string(),
                    None => "".to_string(),
                },
                oninput: move |event| {
                    *start.write() = parse_time(&event.value());
                },
            }
            button {
                class: "btn btn-success",
                onclick: move |_| {
                    *start.write() = Some(Utc::now());
                    use_coroutine_handle::<Action>().send(Action::Start(track.clone()));
                },
                "Start"
            }
        }
    }
}

use chrono::{DateTime, Local, NaiveTime, TimeZone, Utc};
use dioxus::prelude::*;

const TIME_FORMAT: &str = "%H:%M:%S";

fn parse_time(str: &str) -> Option<DateTime<Utc>> {
    let naive_time = NaiveTime::parse_from_str(str, TIME_FORMAT).ok()?;
    let local_time = Local
        .from_local_datetime(&(Local::now().date_naive().and_time(naive_time)))
        .single()?;

    Some(local_time.to_utc())
}

fn format_time(time: DateTime<Utc>) -> String {
    time.with_timezone(&Local).format(TIME_FORMAT).to_string()
}

#[component]
pub fn TimeInput(
    time: Option<DateTime<Utc>>,
    editing: Signal<bool>,
    onsave: EventHandler<DateTime<Utc>>,
    span_class: Option<String>,
) -> Element {
    let mut text = use_signal(|| "".to_string());

    use_effect(move || {
        if editing() {
            text.set(match time {
                Some(start) => format_time(start),
                None => "".to_string(),
            });
        }
    });

    rsx! {
        if editing() {
            input {
                class: "form-control form-control-sm",
                class: if parse_time(&text()).is_none() { "is-invalid" },
                autofocus: true,
                value: text,
                onkeydown: move |evt| {
                    if evt.key() == Key::Enter {
                        if let Some(parsed) = parse_time(&text()) {
                            editing.set(false);
                            onsave(parsed);
                        }
                    }
                },
                oninput: move |event| {
                    text.set(event.value());
                },
            }
        } else {
            span {
                class: span_class,
                ondoubleclick: move |_evt| {
                    editing.set(true);
                },
                if let Some(start) = time {
                    {format_time(start)}
                }
            }
        }
    }
}

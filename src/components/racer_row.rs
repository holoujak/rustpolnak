use dioxus::prelude::*;

use crate::components::app::Action;
use crate::components::time_input::TimeInput;
use crate::race::Racer;
use crate::time_utils::{format_time, format_time_delta_millis};

#[component]
pub fn RacerRow(racer: Racer) -> Element {
    let editing = use_signal(|| false);
    let start_number = racer.start_number.clone();

    rsx! {
        tr {
            td { "{racer.start_number}" }
            td { "{racer.first_name}" }
            td { "{racer.last_name}" }
            td { "{racer.track}" }
            td { "{format_time(racer.start)}" }
            td { width: "124px",
                TimeInput {
                    time: racer.finish,
                    editing,
                    remove_button: true,
                    onsave: move |time| {
                        use_coroutine_handle::<Action>()
                            .send(Action::FinishEdit(start_number.clone(), time));
                    },
                }
            }
            td { "{format_time_delta_millis(racer.time)}" }
            td { "{racer.track_rank.map(|rank| rank.to_string()).unwrap_or_default() }" }
            td {
                for category in racer.categories.clone() {
                    "{category} "
                }
            }
            td {
                for category_rank in &racer.categories_rank {
                    span { class: "me-2", "{category_rank.0}: {category_rank.1}" }
                }
            }
        }
    }
}

use dioxus::prelude::*;

use crate::race::Racer;
use crate::time_utils::{format_time, format_time_delta};

#[component]
pub fn RacerRow(racer: Racer) -> Element {
    rsx! {
        tr {
            td { "{racer.start_number}" }
            td { "{racer.first_name}" }
            td { "{racer.last_name}" }
            td { "{racer.track}" }
            td { "{format_time(racer.start)}" }
            td { "{format_time(racer.finish)}" }
            td { "{format_time_delta(racer.time)}" }
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

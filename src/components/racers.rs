use chrono::{DateTime, Local, Utc};
use dioxus::prelude::*;

use crate::{
    components::categories_list::CategoriesList,
    race::{Race, Racer, RacerField},
    sort_table::Th,
    sorter::Sorter,
};

fn format_time(datetime: Option<DateTime<Utc>>) -> String {
    match datetime {
        Some(datetime) => datetime
            .with_timezone(&Local)
            .format("%H:%M:%S%.3f")
            .to_string(),
        None => "".to_string(),
    }
}

#[component]
pub fn Racers(race: Race) -> Element {
    let selected_category_id = use_signal(|| Option::<String>::None);
    let sorter = use_signal(|| Sorter::<RacerField>::new(RacerField::StartNumber));

    let mut sorted = race.racers.clone();
    let field = sorter.read().active;
    sorted.sort_by(|a, b| sorter.read().cmp_by(a, b, field, Racer::cmp_by));

    rsx! {
        div { class: "overflow-y-scroll",
            table { class: "table table-striped table-hover table-sm",
                thead { class: "table-dark",
                    tr {
                        Th { sorter, field: RacerField::StartNumber, "Start number" }
                        Th { sorter, field: RacerField::FirstName, "First name" }
                        Th { sorter, field: RacerField::LastName, "Last name" }
                        Th { sorter, field: RacerField::Track, "Track" }
                        th { "Start" }
                        th { "Finish" }
                        th { "Track rank" }
                        th {
                            CategoriesList {
                                categories: race.categories,
                                selected_category_id,
                            }
                        }
                    }
                }
                tbody {

                    for racer in sorted.iter() {
                        if (selected_category_id.read().clone())
                            .is_none_or(|cat_id| racer.categories.contains(&cat_id))
                        {
                            tr {
                                td { "{racer.start_number}" }
                                td { "{racer.first_name}" }
                                td { "{racer.last_name}" }
                                td { "{racer.track}" }
                                td { "{format_time(racer.start)}" }
                                td { "{format_time(racer.finish)}" }
                                td {
                                    "{race.tracks_rank.get(&racer.track)
                                    .and_then(|m| m.get(&racer.start_number))
                                    .map(|rank| rank.to_string())
                                    .unwrap_or_default() }"
                                }
                                td {
                                    for category in racer.categories.clone() {
                                        "{category} "
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

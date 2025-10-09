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

    let start_number_filter = use_signal(|| None::<String>);
    let first_name_filter = use_signal(|| None::<String>);
    let last_name_filter = use_signal(|| None::<String>);

    rsx! {
        table { class: "table table-striped table-hover table-sm",
            thead { class: "table-dark",
                tr {
                    Th {
                        sorter,
                        field: RacerField::StartNumber,
                        filter: start_number_filter,
                        "Start number"
                    }
                    Th {
                        sorter,
                        field: RacerField::FirstName,
                        filter: first_name_filter,
                        "First name"
                    }
                    Th {
                        sorter,
                        field: RacerField::LastName,
                        filter: last_name_filter,
                        "Last name"
                    }
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
                        && start_number_filter
                            .read()
                            .as_ref()
                            .is_none_or(|f| racer.start_number.to_string().contains(f))
                        && first_name_filter
                            .read()
                            .as_ref()
                            .is_none_or(|f| racer.first_name.to_lowercase().contains(&f.to_lowercase()))
                        && last_name_filter
                            .read()
                            .as_ref()
                            .is_none_or(|f| racer.last_name.to_lowercase().contains(&f.to_lowercase()))
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
                                    .map_or(String::new(), |rank| rank.to_string())}"
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

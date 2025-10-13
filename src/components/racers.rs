use chrono::{DateTime, Local, TimeDelta, Utc};
use dioxus::prelude::*;

use crate::{
    components::{
        categories_list::CategoriesList,
        th::{Sorter, Th},
    },
    race::{Category, Race, Racer, RacerField},
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

pub fn format_time_delta(delta: Option<TimeDelta>) -> String {
    let delta = match delta {
        Some(delta) => delta,
        _ => return "".to_string(),
    };

    let total_millis = delta.num_milliseconds();
    let hours = total_millis / 1000 / 3600;
    let mins = (total_millis / 1000 / 60) % 60;
    let secs = (total_millis / 1000) % 60;
    let millis = total_millis % 1000;

    format!("{hours:02}:{mins:02}:{secs:02}.{millis:03}")
}

// Checks whether a racer matches all active filters in the map.
// The `filters` map contains RacerField -> Option<String>. Only non-empty Some()
// values are applied. Matching is case-insensitive for string like types.
fn matches_filters(
    racer: &Racer,
    filters: &std::collections::HashMap<RacerField, String>,
    category: Option<Category>,
) -> bool {
    if let Some(cat) = category {
        if !racer.categories.contains(&cat) {
            return false;
        }
    }

    for (field, val) in filters.iter() {
        let filter_lowercase = val.trim().to_lowercase();
        if filter_lowercase.is_empty() {
            continue;
        }

        match field {
            RacerField::StartNumber => {
                if !racer.start_number.to_string().contains(&filter_lowercase) {
                    return false;
                }
            }
            RacerField::FirstName => {
                if !racer.first_name.to_lowercase().contains(&filter_lowercase) {
                    return false;
                }
            }
            RacerField::LastName => {
                if !racer.last_name.to_lowercase().contains(&filter_lowercase) {
                    return false;
                }
            }
            RacerField::TagId => {
                if !racer.tag.to_lowercase().contains(&filter_lowercase) {
                    return false;
                }
            }
            RacerField::Track => {
                if !racer.track.0.to_lowercase().contains(&filter_lowercase) {
                    return false;
                }
            }
            RacerField::TrackRank => {
                let rank_str = racer
                    .track_rank
                    .map(|rank| rank.to_string())
                    .unwrap_or_default();
                if !rank_str.contains(&filter_lowercase) {
                    return false;
                }
            }
            RacerField::Start => {
                if !format_time(racer.start)
                    .to_lowercase()
                    .contains(&filter_lowercase)
                {
                    return false;
                }
            }
            RacerField::Finish => {
                if !format_time(racer.finish)
                    .to_lowercase()
                    .contains(&filter_lowercase)
                {
                    return false;
                }
            }
            RacerField::Time => {
                if !format_time_delta(racer.time)
                    .to_lowercase()
                    .contains(&filter_lowercase)
                {
                    return false;
                }
            }
            _ => {
                tracing::error!("Unimplemented filter for field {:?}", field);
            }
        }
    }

    true
}

#[component]
pub fn Racers(race: Race) -> Element {
    let selected_category_id = use_signal(|| Option::<Category>::None);
    let sorter = use_signal(|| Sorter::<RacerField>::new(RacerField::StartNumber));

    let filters = use_signal(std::collections::HashMap::<RacerField, String>::new);

    let mut sorted = race.racers.clone();
    let field = sorter().active;
    sorted.sort_by(|a, b| sorter().cmp_by(a, b, field, Racer::cmp_by));

    rsx! {
        div { class: "overflow-y-scroll",
            table { class: "table table-striped table-hover table-sm",
                thead { class: "table-dark",
                    tr {
                        Th {
                            sorter,
                            field: RacerField::StartNumber,
                            filters,
                            "Start number"
                        }
                        Th {
                            sorter,
                            field: RacerField::FirstName,
                            filters,
                            "First name"
                        }
                        Th {
                            sorter,
                            field: RacerField::LastName,
                            filters,
                            "Last name"
                        }
                        Th {
                            sorter,
                            field: RacerField::Track,
                            filters,
                            "Track"
                        }
                        Th { sorter, field: RacerField::Start, "Start" }
                        Th { sorter, field: RacerField::Finish, "Finish" }
                        Th { sorter, field: RacerField::Time, "Time" }
                        Th { sorter, field: RacerField::TrackRank, "Track rank" }
                        th {
                            CategoriesList {
                                categories: race.categories.clone(),
                                selected_category_id,
                            }
                        }
                        Th { sorter, field: RacerField::CategoriesRank, "Categories rank" }
                    }
                }
                tbody {
                    for racer in sorted
                        .iter()
                        .filter(|racer| matches_filters(racer, &filters(), selected_category_id()))
                    {
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
            }
        }
    }
}

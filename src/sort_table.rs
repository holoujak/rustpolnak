use std::hash::Hash;

use crate::sorter::{Direction, Sorter};
use dioxus::prelude::*;

/// Custom <Th> component that toggles sorting on click
#[component]
pub fn Th<F: Copy + Eq + Hash + 'static>(
    sorter: Signal<Sorter<F>>,
    field: F,
    filters: Option<Signal<std::collections::HashMap<F, String>>>,
    children: Element,
) -> Element {
    rsx! {
        th { role: "button",
            span {
                onclick: move |_| {
                    sorter.write().toggle(field);
                },
                {children}
            }

            {
                let s = sorter();
                if s.active == field {
                    match s.direction {
                        Direction::Asc => rsx! {
                            span { " ↑" }
                        },
                        Direction::Desc => rsx! {
                            span { " ↓" }
                        },
                    }
                } else {
                    rsx! {
                        span { class: "invisible", " ↑" }
                    }
                }
            }
            {
                if let Some(mut filters_signal) = filters {
                    rsx! {
                        div { class: "mt-1",
                            input {
                                r#type: "text",
                                placeholder: "Filter",
                                // read current value for this field from the map
                                value: filters_signal().get(&field).map(|s| s.as_str()).unwrap_or(""),
                                oninput: move |e| {
                                    let v = e.value().trim().to_string();
                                    filters_signal.write().insert(field, v);
                                },
                            }
                        }
                    }
                } else {
                    rsx! {}
                }
            }
        }
    }
}

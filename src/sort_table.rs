use crate::sorter::{Direction, Sorter};
use dioxus::prelude::*;

/// Table header component that toggles sorting when clicked
#[derive(Props, PartialEq, Clone)]
pub struct ThProps<F: Copy + Eq + std::hash::Hash + 'static> {
    pub sorter: Signal<Sorter<F>>,
    pub field: F,
    #[props(optional)]
    pub filters: Option<Signal<std::collections::HashMap<F, String>>>,
    pub children: Element,
}

/// Custom <Th> component that toggles sorting on click
#[allow(non_snake_case)]
pub fn Th<F: Copy + Eq + std::hash::Hash + 'static>(props: ThProps<F>) -> Element {
    let mut sorter = props.sorter;
    let filters = props.filters;
    let field = props.field;

    rsx! {
        th { role: "button",
            span {
                onclick: move |_| {
                    sorter.write().toggle(field);
                },
                {props.children}
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

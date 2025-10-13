use std::cmp::Ordering;
use std::hash::Hash;

use dioxus::prelude::*;

/// Direction of sorting
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Direction {
    Asc,
    Desc,
}

#[derive(Clone)]
pub struct Sorter<F: Copy + Eq + Hash> {
    pub active: F,
    pub direction: Direction,
}

impl<F: Copy + Eq + Hash> Sorter<F> {
    pub fn new(field: F) -> Self {
        Self {
            active: field,
            direction: Direction::Asc,
        }
    }

    /// Toggle sorting for a given field
    pub fn toggle(&mut self, field: F) {
        if field == self.active {
            self.direction = match self.direction {
                Direction::Asc => Direction::Desc,
                Direction::Desc => Direction::Asc,
            };
        } else {
            self.active = field;
            self.direction = Direction::Asc;
        }
    }

    /// Compare two values using a custom comparator
    pub fn cmp_by<T>(
        &self,
        a: &T,
        b: &T,
        field: F,
        cmp_fn: impl Fn(&T, &T, F) -> Ordering,
    ) -> Ordering {
        match (self.active, self.direction) {
            (active_field, Direction::Desc) if active_field == field => {
                cmp_fn(a, b, field).reverse()
            }
            _ => cmp_fn(a, b, field),
        }
    }
}

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

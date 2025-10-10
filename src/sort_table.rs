use crate::sorter::{Direction, Sorter};
use dioxus::prelude::*;

/// Table header component that toggles sorting when clicked
#[derive(Props, PartialEq, Clone)]
pub struct ThProps<F: Copy + Eq + std::hash::Hash + 'static> {
    pub sorter: Signal<Sorter<F>>,
    pub field: F,
    #[props(optional)]
    pub filter: Option<Signal<Option<String>>>,
    pub children: Element,
}

/// Custom <Th> component that toggles sorting on click
#[allow(non_snake_case)]
pub fn Th<F: Copy + Eq + std::hash::Hash + 'static>(props: ThProps<F>) -> Element {
    let mut sorter = props.sorter;
    let filter = props.filter;
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
                let s = sorter.read();
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
                if let Some(mut f) = filter {
                    rsx! {
                        br {}
                        input {
                            r#type: "text",
                            placeholder: "Filter",
                            value: f.read().as_deref().unwrap_or(""),
                            oninput: move |e| {
                                let value = e.value().clone();
                                if value.is_empty() {
                                    f.set(None);
                                } else {
                                    f.set(Some(value));
                                }
                            },
                        }
                    }
                } else {
                    rsx! {}
                }
            }
        }
    }
}

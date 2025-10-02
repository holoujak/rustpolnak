use crate::sorter::{Direction, Sorter};
use dioxus::prelude::*;

/// Table header component that toggles sorting when clicked
#[derive(Props, PartialEq, Clone)]
pub struct ThProps<F: Copy + Eq + std::hash::Hash + 'static> {
    pub sorter: Signal<Sorter<F>>,
    pub field: F,
    #[props(optional)]
    pub children: Element,
}

/// Custom <Th> component that toggles sorting on click
#[allow(non_snake_case)]
pub fn Th<F: Copy + Eq + std::hash::Hash + 'static>(props: ThProps<F>) -> Element {
    let mut sorter = props.sorter;
    let field = props.field;

    rsx! {
        th {
            role: "button",
            onclick: move |_| {
                sorter.write().toggle(field);
            },
            {props.children}
            {
                // Optional arrow indicator
                let s = sorter.read();
                if s.active == field {
                    match s.direction {
                        Direction::Asc => rsx!(span { " ↑" }),
                        Direction::Desc => rsx!(span { " ↓" }),
                    }
                } else {
                    rsx!(span { class: "invisible", " ↑"  })
                }
            }
        }
    }
}

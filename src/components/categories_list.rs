use dioxus::prelude::*;

use crate::race::Category;

#[component]
pub fn CategoriesList(
    categories: Vec<Category>,
    selected_category_id: Signal<Option<Category>>,
) -> Element {
    rsx! {
        select {
            onchange: move |e| {
                let val = e.value().parse::<String>().ok();
                selected_category_id
                    .set(if val == Some("All".to_string()) { None } else { val.map(Category) });
            },
            option { disabled: false, selected: true, "All" }
            for c in categories.iter() {
                option { value: "{c}", "{c}" }
            }
        }
    }
}

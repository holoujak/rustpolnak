use dioxus::prelude::*;

#[component]
pub fn CategoriesList(
    categories: Vec<String>,
    selected_category_id: Signal<Option<String>>,
) -> Element {
    rsx! {
        select {
            onchange: move |e| {
                let val = e.value().parse::<String>().ok();
                selected_category_id
                    .set(if val == Some("All".to_string()) { None } else { val });
            },
            option { disabled: false, selected: true, "All" }
            for c in categories.iter() {
                option { value: "{c}", "{c}" }
            }
        }
    }
}

use dioxus::prelude::*;

#[component]
pub fn CategoriesList(
    categories: Vec<String>,
    selected_category_id: Signal<Option<String>>,
) -> Element {
    let mut sorted_categories: Vec<_> = categories.to_vec();
    sorted_categories.sort();

    rsx! {
        select {
            onchange: move |e| {
                let val = e.value().parse::<String>().ok();
                selected_category_id
                    .set(if val == Some("All".to_string()) { None } else { val });
            },
            option { disabled: false, selected: true, "All" }
            for c in sorted_categories.iter() {
                option { value: "{c}", "{c}" }
            }
        }
    }
}

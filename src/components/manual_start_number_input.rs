use dioxus::prelude::*;

use crate::components::app::Action;

#[component]
pub fn ManualStartNumberInput() -> Element {
    let mut start_number = use_signal(|| "".to_string());

    rsx! {
        form {
            class: "mb-1",
            onsubmit: move |event| {
                event.prevent_default();
                if let Ok(start_number) = start_number.read().parse() {
                    use_coroutine_handle::<Action>()
                        .send(Action::FinishByStartNumber(start_number));
                }
                start_number.set(String::from(""));
            },
            input {
                class: "form-control",
                placeholder: "Finish racer by start number",
                r#type: "number",
                value: start_number,
                onkeydown: move |event| {
                    let key = event.key().to_string();
                    if key.chars().all(|c| c.is_ascii_digit()) {
                        let current_start_number = start_number.read().clone();
                        start_number.set(current_start_number + &key);
                    } else if event.key() != Key::Enter {
                        event.prevent_default();
                        start_number.set(String::from(""));
                    }
                },
            }
        }
    }
}

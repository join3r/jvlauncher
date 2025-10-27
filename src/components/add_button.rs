use dioxus::prelude::*;
use crate::components::AddModal;
use crate::state::AppState;

#[component]
pub fn AddButton() -> Element {
    let mut state = use_context::<Signal<AppState>>();

    let handle_click = move |_| {
        state.write().show_add_modal = true;
    };

    rsx! {
        div {
            button {
                class: "add-button",
                onclick: handle_click,
                style: "position: fixed; bottom: 20px; right: 20px; width: 60px; height: 60px; border-radius: 50%; background: #007bff; color: white; border: none; font-size: 32px; cursor: pointer; box-shadow: 0 4px 8px rgba(0,0,0,0.2); display: flex; align-items: center; justify-content: center;",
                "+"
            }
            
            if state.read().show_add_modal {
                AddModal {}
            }
        }
    }
}


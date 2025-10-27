use dioxus::prelude::*;
use crate::components::IconItem;
use crate::state::AppState;
use crate::types::App;

#[component]
pub fn AppGrid() -> Element {
    let mut state = use_context::<Signal<AppState>>();
    
    // Load apps on mount
    use_effect(move || {
        spawn(async move {
            #[cfg(target_family = "wasm")]
            {
                // For web/Tauri integration, call the backend
                if let Ok(apps) = invoke_get_all_apps().await {
                    state.write().apps = apps;
                }
            }
        });
    });

    // Handle keyboard navigation
    let handle_keydown = move |evt: Event<KeyboardData>| {
        let key = evt.key();
        let mut state_write = state.write();
        let grid_size = state_write.settings.grid_size as usize;
        let total_apps = state_write.apps.len();
        
        if total_apps == 0 {
            return;
        }

        let current_index = state_write.selected_index.unwrap_or(0);

        match key {
            Key::ArrowRight => {
                let next_index = (current_index + 1) % total_apps;
                state_write.selected_index = Some(next_index);
            }
            Key::ArrowLeft => {
                let prev_index = if current_index == 0 {
                    total_apps - 1
                } else {
                    current_index - 1
                };
                state_write.selected_index = Some(prev_index);
            }
            Key::ArrowDown => {
                let next_index = (current_index + grid_size).min(total_apps - 1);
                state_write.selected_index = Some(next_index);
            }
            Key::ArrowUp => {
                let prev_index = if current_index >= grid_size {
                    current_index - grid_size
                } else {
                    0
                };
                state_write.selected_index = Some(prev_index);
            }
            Key::Enter => {
                if let Some(app) = state_write.apps.get(current_index) {
                    let app_id = app.id;
                    drop(state_write); // Release the write lock
                    spawn(async move {
                        let _ = invoke_launch(app_id).await;
                    });
                }
            }
            Key::Escape => {
                drop(state_write);
                spawn(async move {
                    let _ = invoke_hide_main_window().await;
                });
            }
            _ => {}
        }
    };

    let apps = state.read().apps.clone();
    let grid_size = state.read().settings.grid_size;
    let selected_index = state.read().selected_index;

    rsx! {
        div {
            class: "app-grid",
            tabindex: 0,
            onkeydown: handle_keydown,
            style: "display: grid; grid-template-columns: repeat({grid_size}, 1fr); gap: 20px; padding: 20px;",
            
            for (index, app) in apps.iter().enumerate() {
                IconItem {
                    key: "{app.id}",
                    app: app.clone(),
                    is_selected: selected_index == Some(index),
                    index: index,
                }
            }
        }
    }
}

// Tauri command invocations
#[cfg(target_family = "wasm")]
async fn invoke_get_all_apps() -> Result<Vec<App>, String> {
    // This would be replaced with actual Tauri invoke
    use wasm_bindgen::prelude::*;
    
    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
        async fn invoke(cmd: &str, args: JsValue) -> JsValue;
    }
    
    let result = invoke("get_all_apps", JsValue::NULL).await;
    serde_wasm_bindgen::from_value(result).map_err(|e| e.to_string())
}

#[cfg(target_family = "wasm")]
async fn invoke_launch(app_id: i64) -> Result<(), String> {
    use wasm_bindgen::prelude::*;
    
    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
        async fn invoke(cmd: &str, args: JsValue) -> JsValue;
    }
    
    let args = serde_wasm_bindgen::to_value(&serde_json::json!({ "appId": app_id }))
        .map_err(|e| e.to_string())?;
    
    invoke("launch", args).await;
    Ok(())
}

#[cfg(target_family = "wasm")]
async fn invoke_hide_main_window() -> Result<(), String> {
    use wasm_bindgen::prelude::*;
    
    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
        async fn invoke(cmd: &str, args: JsValue) -> JsValue;
    }
    
    invoke("hide_main_window", JsValue::NULL).await;
    Ok(())
}


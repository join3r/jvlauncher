use dioxus::prelude::*;
use crate::types::App;
use crate::state::AppState;

#[component]
pub fn IconItem(app: App, is_selected: bool, index: usize) -> Element {
    let mut state = use_context::<Signal<AppState>>();
    let mut show_context_menu = use_signal(|| false);
    
    // Handle click to launch app
    let handle_click = move |_| {
        let app_id = app.id;
        spawn(async move {
            let _ = invoke_launch(app_id).await;
        });
    };

    // Handle right click to show context menu
    let handle_context_menu = move |evt: Event<MouseData>| {
        evt.prevent_default();
        show_context_menu.set(true);
    };

    // Handle edit
    let handle_edit = move |_| {
        state.write().edit_app = Some(app.clone());
        show_context_menu.set(false);
    };

    // Handle delete
    let handle_delete = move |_| {
        let app_id = app.id;
        show_context_menu.set(false);
        spawn(async move {
            let _ = invoke_delete_app(app_id).await;
            // Reload apps
            if let Ok(apps) = invoke_get_all_apps().await {
                // Update state (would need access to state here)
            }
        });
    };

    // Drag and drop handlers
    let handle_drag_start = move |evt: Event<DragData>| {
        evt.set_data("text/plain", &app.id.to_string());
    };

    let handle_drag_over = move |evt: Event<DragData>| {
        evt.prevent_default();
    };

    let handle_drop = move |evt: Event<DragData>| {
        evt.prevent_default();
        if let Some(dragged_id_str) = evt.get_data("text/plain") {
            if let Ok(dragged_id) = dragged_id_str.parse::<i64>() {
                // Reorder logic
                let mut state_write = state.write();
                if let Some(dragged_index) = state_write.apps.iter().position(|a| a.id == dragged_id) {
                    let app = state_write.apps.remove(dragged_index);
                    state_write.apps.insert(index, app);
                    
                    // Update positions in database
                    let app_ids: Vec<i64> = state_write.apps.iter().map(|a| a.id).collect();
                    drop(state_write);
                    
                    spawn(async move {
                        let _ = invoke_reorder_apps(app_ids).await;
                    });
                }
            }
        }
    };

    let selected_class = if is_selected { " selected" } else { "" };

    rsx! {
        div {
            class: "icon-item{selected_class}",
            draggable: true,
            ondragstart: handle_drag_start,
            ondragover: handle_drag_over,
            ondrop: handle_drop,
            onclick: handle_click,
            oncontextmenu: handle_context_menu,
            style: "cursor: pointer; text-align: center; padding: 10px; border: 2px solid {if is_selected { '#007bff' } else { 'transparent' }}; border-radius: 8px;",
            
            // Icon
            if let Some(icon_path) = &app.icon_path {
                img {
                    src: "asset://localhost/{icon_path}",
                    alt: "{app.name}",
                    style: "width: 64px; height: 64px; object-fit: contain;",
                }
            } else {
                div {
                    style: "width: 64px; height: 64px; background: #ddd; border-radius: 8px; display: flex; align-items: center; justify-content: center; margin: 0 auto;",
                    span { "{app.name.chars().next().unwrap_or('?').to_uppercase()}" }
                }
            }
            
            // Name
            div {
                class: "app-name",
                style: "margin-top: 8px; font-weight: bold; font-size: 14px;",
                "{app.name}"
            }
            
            // Shortcut
            if let Some(shortcut) = &app.shortcut {
                div {
                    class: "app-shortcut",
                    style: "margin-top: 4px; font-size: 12px; color: #666;",
                    "{shortcut}"
                }
            }
            
            // Context menu
            if *show_context_menu.read() {
                div {
                    class: "context-menu",
                    style: "position: absolute; background: white; border: 1px solid #ccc; border-radius: 4px; padding: 8px; z-index: 1000; box-shadow: 0 2px 8px rgba(0,0,0,0.15);",
                    
                    button {
                        onclick: handle_edit,
                        style: "display: block; width: 100%; padding: 8px; border: none; background: none; cursor: pointer; text-align: left;",
                        "Edit"
                    }
                    button {
                        onclick: handle_delete,
                        style: "display: block; width: 100%; padding: 8px; border: none; background: none; cursor: pointer; text-align: left; color: red;",
                        "Delete"
                    }
                }
            }
        }
    }
}

// Tauri command invocations
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
async fn invoke_delete_app(app_id: i64) -> Result<(), String> {
    use wasm_bindgen::prelude::*;
    
    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
        async fn invoke(cmd: &str, args: JsValue) -> JsValue;
    }
    
    let args = serde_wasm_bindgen::to_value(&serde_json::json!({ "appId": app_id }))
        .map_err(|e| e.to_string())?;
    
    invoke("delete_app", args).await;
    Ok(())
}

#[cfg(target_family = "wasm")]
async fn invoke_get_all_apps() -> Result<Vec<App>, String> {
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
async fn invoke_reorder_apps(app_ids: Vec<i64>) -> Result<(), String> {
    use wasm_bindgen::prelude::*;
    
    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
        async fn invoke(cmd: &str, args: JsValue) -> JsValue;
    }
    
    let args = serde_wasm_bindgen::to_value(&serde_json::json!({ "appIds": app_ids }))
        .map_err(|e| e.to_string())?;
    
    invoke("reorder_apps", args).await;
    Ok(())
}


use dioxus::prelude::*;
use crate::state::AppState;
use crate::types::App;

#[component]
pub fn EditModal(app: App) -> Element {
    let mut state = use_context::<Signal<AppState>>();
    let mut name = use_signal(|| app.name.clone());
    let mut binary_path = use_signal(|| app.binary_path.clone().unwrap_or_default());
    let mut cli_params = use_signal(|| app.cli_params.clone().unwrap_or_default());
    let mut url = use_signal(|| app.url.clone().unwrap_or_default());
    let mut shortcut = use_signal(|| app.shortcut.clone().unwrap_or_default());
    let mut icon_path = use_signal(|| app.icon_path.clone());

    let handle_close = move |_| {
        state.write().edit_app = None;
    };

    let handle_save = move |_| {
        let updated_app = App {
            id: app.id,
            app_type: app.app_type.clone(),
            name: name.read().clone(),
            icon_path: icon_path.read().clone(),
            position: app.position,
            shortcut: if shortcut.read().is_empty() { None } else { Some(shortcut.read().clone()) },
            binary_path: if binary_path.read().is_empty() { None } else { Some(binary_path.read().clone()) },
            cli_params: if cli_params.read().is_empty() { None } else { Some(cli_params.read().clone()) },
            url: if url.read().is_empty() { None } else { Some(url.read().clone()) },
            session_data_path: app.session_data_path.clone(),
        };

        spawn(async move {
            let _ = invoke_update_app(updated_app).await;
        });

        state.write().edit_app = None;
    };

    let handle_browse_icon = move |_| {
        spawn(async move {
            if let Ok(path) = invoke_open_file_dialog().await {
                if let Ok(saved_path) = invoke_save_icon_from_file(path, name.read().clone()).await {
                    icon_path.set(Some(saved_path));
                }
            }
        });
    };

    let handle_paste_icon = move |_| {
        spawn(async move {
            match invoke_paste_icon_from_clipboard(name.read().clone()).await {
                Ok(saved_path) => {
                    icon_path.set(Some(saved_path));
                }
                Err(e) => {
                    #[cfg(target_family = "wasm")]
                    {
                        use wasm_bindgen::prelude::*;
                        #[wasm_bindgen]
                        extern "C" {
                            #[wasm_bindgen(js_namespace = ["window"])]
                            fn alert(s: &str);
                        }
                        alert(&format!("Failed to paste icon from clipboard: {}", e));
                    }
                }
            }
        });
    };

    rsx! {
        div {
            class: "modal-overlay",
            style: "position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 2000;",
            onclick: handle_close,
            
            div {
                class: "modal-content",
                style: "background: white; padding: 30px; border-radius: 12px; max-width: 500px; width: 90%; max-height: 80vh; overflow-y: auto;",
                onclick: move |evt: Event<MouseData>| evt.stop_propagation(),
                
                h2 { style: "margin-top: 0;", "Edit Application" }
                
                // Name
                div {
                    style: "margin-bottom: 20px;",
                    label { style: "display: block; margin-bottom: 8px; font-weight: bold;", "Name:" }
                    input {
                        r#type: "text",
                        value: "{name}",
                        oninput: move |evt| name.set(evt.value()),
                        style: "width: 100%; padding: 8px; border: 1px solid #ccc; border-radius: 4px;",
                    }
                }
                
                // Conditional fields based on app type
                match app.app_type {
                    crate::types::AppType::Webapp => rsx! {
                        div {
                            style: "margin-bottom: 20px;",
                            label { style: "display: block; margin-bottom: 8px; font-weight: bold;", "URL:" }
                            input {
                                r#type: "text",
                                value: "{url}",
                                oninput: move |evt| url.set(evt.value()),
                                style: "width: 100%; padding: 8px; border: 1px solid #ccc; border-radius: 4px;",
                            }
                        }
                    },
                    _ => rsx! {
                        div {
                            style: "margin-bottom: 20px;",
                            label { style: "display: block; margin-bottom: 8px; font-weight: bold;", "Binary Path:" }
                            input {
                                r#type: "text",
                                value: "{binary_path}",
                                oninput: move |evt| binary_path.set(evt.value()),
                                style: "width: 100%; padding: 8px; border: 1px solid #ccc; border-radius: 4px;",
                            }
                        }
                        
                        div {
                            style: "margin-bottom: 20px;",
                            label { style: "display: block; margin-bottom: 8px; font-weight: bold;", "Command Line Parameters:" }
                            input {
                                r#type: "text",
                                value: "{cli_params}",
                                oninput: move |evt| cli_params.set(evt.value()),
                                style: "width: 100%; padding: 8px; border: 1px solid #ccc; border-radius: 4px;",
                            }
                        }
                    }
                }
                
                // Icon
                div {
                    style: "margin-bottom: 20px;",
                    label { style: "display: block; margin-bottom: 8px; font-weight: bold;", "Icon:" }
                    div {
                        style: "display: flex; gap: 8px;",
                        button {
                            onclick: handle_browse_icon,
                            style: "padding: 8px 16px; background: #007bff; color: white; border: none; border-radius: 4px; cursor: pointer;",
                            "Choose Icon"
                        }
                        button {
                            onclick: handle_paste_icon,
                            style: "padding: 8px 16px; background: #007bff; color: white; border: none; border-radius: 4px; cursor: pointer;",
                            "Paste Icon"
                        }
                    }
                    if let Some(icon) = icon_path.read().as_ref() {
                        div {
                            style: "margin-top: 8px;",
                            img {
                                src: "asset://localhost/{icon}",
                                style: "width: 48px; height: 48px; object-fit: contain;",
                            }
                        }
                    }
                }
                
                // Shortcut
                div {
                    style: "margin-bottom: 20px;",
                    label { style: "display: block; margin-bottom: 8px; font-weight: bold;", "Keyboard Shortcut:" }
                    input {
                        r#type: "text",
                        value: "{shortcut}",
                        oninput: move |evt| shortcut.set(evt.value()),
                        style: "width: 100%; padding: 8px; border: 1px solid #ccc; border-radius: 4px;",
                    }
                }
                
                // Actions
                div {
                    style: "display: flex; gap: 12px; justify-content: flex-end;",
                    button {
                        onclick: handle_close,
                        style: "padding: 10px 20px; background: #6c757d; color: white; border: none; border-radius: 4px; cursor: pointer;",
                        "Cancel"
                    }
                    button {
                        onclick: handle_save,
                        style: "padding: 10px 20px; background: #28a745; color: white; border: none; border-radius: 4px; cursor: pointer;",
                        "Save"
                    }
                }
            }
        }
    }
}

// Tauri command invocations
#[cfg(target_family = "wasm")]
async fn invoke_update_app(app: App) -> Result<(), String> {
    use wasm_bindgen::prelude::*;
    
    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
        async fn invoke(cmd: &str, args: JsValue) -> JsValue;
    }
    
    let args = serde_wasm_bindgen::to_value(&app).map_err(|e| e.to_string())?;
    invoke("update_app", args).await;
    Ok(())
}

#[cfg(target_family = "wasm")]
async fn invoke_open_file_dialog() -> Result<String, String> {
    use wasm_bindgen::prelude::*;
    
    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "dialog"])]
        async fn open(options: JsValue) -> JsValue;
    }
    
    let result = open(JsValue::NULL).await;
    serde_wasm_bindgen::from_value(result).map_err(|e| e.to_string())
}

#[cfg(target_family = "wasm")]
async fn invoke_save_icon_from_file(source_path: String, app_name: String) -> Result<String, String> {
    use wasm_bindgen::prelude::*;
    
    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
        async fn invoke(cmd: &str, args: JsValue) -> JsValue;
    }
    
    let args = serde_wasm_bindgen::to_value(&serde_json::json!({
        "sourcePath": source_path,
        "appName": app_name
    })).map_err(|e| e.to_string())?;

    let result = invoke("save_icon_from_file", args).await;
    serde_wasm_bindgen::from_value(result).map_err(|e| e.to_string())
}

#[cfg(target_family = "wasm")]
async fn invoke_paste_icon_from_clipboard(app_name: String) -> Result<String, String> {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
        async fn invoke(cmd: &str, args: JsValue) -> JsValue;
    }

    let args = serde_wasm_bindgen::to_value(&serde_json::json!({
        "appName": app_name
    })).map_err(|e| e.to_string())?;

    let result = invoke("paste_icon_from_clipboard", args).await;
    serde_wasm_bindgen::from_value(result).map_err(|e| e.to_string())
}


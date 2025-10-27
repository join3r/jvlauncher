use dioxus::prelude::*;
use crate::state::AppState;
use crate::types::{AppType, NewApp};

#[component]
pub fn AddModal() -> Element {
    let mut state = use_context::<Signal<AppState>>();
    let mut app_type = use_signal(|| AppType::App);
    let mut name = use_signal(|| String::new());
    let mut binary_path = use_signal(|| String::new());
    let mut cli_params = use_signal(|| String::new());
    let mut url = use_signal(|| String::new());
    let mut shortcut = use_signal(|| String::new());
    let mut icon_path = use_signal(|| None::<String>);

    let handle_close = move |_| {
        state.write().show_add_modal = false;
    };

    let handle_save = move |_| {
        let new_app = NewApp {
            app_type: app_type.read().clone(),
            name: name.read().clone(),
            icon_path: icon_path.read().clone(),
            shortcut: if shortcut.read().is_empty() { None } else { Some(shortcut.read().clone()) },
            binary_path: if binary_path.read().is_empty() { None } else { Some(binary_path.read().clone()) },
            cli_params: if cli_params.read().is_empty() { None } else { Some(cli_params.read().clone()) },
            url: if url.read().is_empty() { None } else { Some(url.read().clone()) },
        };

        spawn(async move {
            let _ = invoke_create_app(new_app).await;
            // Reload apps
            if let Ok(apps) = invoke_get_all_apps().await {
                // Update state
            }
        });

        state.write().show_add_modal = false;
    };

    let handle_browse_binary = move |_| {
        spawn(async move {
            if let Ok(path) = invoke_open_file_dialog().await {
                binary_path.set(path.clone());
                
                // Try to extract icon
                if let Ok(icon) = invoke_extract_icon(path.clone()).await {
                    icon_path.set(Some(icon));
                }
            }
        });
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

    rsx! {
        div {
            class: "modal-overlay",
            style: "position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 2000;",
            onclick: handle_close,
            
            div {
                class: "modal-content",
                style: "background: white; padding: 30px; border-radius: 12px; max-width: 500px; width: 90%; max-height: 80vh; overflow-y: auto;",
                onclick: move |evt: Event<MouseData>| evt.stop_propagation(),
                
                h2 { style: "margin-top: 0;", "Add New Application" }
                
                // App Type selector
                div {
                    style: "margin-bottom: 20px;",
                    label { style: "display: block; margin-bottom: 8px; font-weight: bold;", "Type:" }
                    select {
                        value: "{app_type:?}",
                        oninput: move |evt| {
                            let val = evt.value();
                            app_type.set(match val.as_str() {
                                "Webapp" => AppType::Webapp,
                                "Tui" => AppType::Tui,
                                _ => AppType::App,
                            });
                        },
                        style: "width: 100%; padding: 8px; border: 1px solid #ccc; border-radius: 4px;",
                        
                        option { value: "App", "Application" }
                        option { value: "Webapp", "Web Application" }
                        option { value: "Tui", "Terminal Application" }
                    }
                }
                
                // Name
                div {
                    style: "margin-bottom: 20px;",
                    label { style: "display: block; margin-bottom: 8px; font-weight: bold;", "Name:" }
                    input {
                        r#type: "text",
                        value: "{name}",
                        oninput: move |evt| name.set(evt.value()),
                        style: "width: 100%; padding: 8px; border: 1px solid #ccc; border-radius: 4px;",
                        placeholder: "Application name",
                    }
                }
                
                // Conditional fields based on app type
                match *app_type.read() {
                    AppType::Webapp => rsx! {
                        div {
                            style: "margin-bottom: 20px;",
                            label { style: "display: block; margin-bottom: 8px; font-weight: bold;", "URL:" }
                            input {
                                r#type: "text",
                                value: "{url}",
                                oninput: move |evt| url.set(evt.value()),
                                style: "width: 100%; padding: 8px; border: 1px solid #ccc; border-radius: 4px;",
                                placeholder: "https://example.com",
                            }
                        }
                    },
                    _ => rsx! {
                        div {
                            style: "margin-bottom: 20px;",
                            label { style: "display: block; margin-bottom: 8px; font-weight: bold;", "Binary Path:" }
                            div {
                                style: "display: flex; gap: 8px;",
                                input {
                                    r#type: "text",
                                    value: "{binary_path}",
                                    oninput: move |evt| binary_path.set(evt.value()),
                                    style: "flex: 1; padding: 8px; border: 1px solid #ccc; border-radius: 4px;",
                                    placeholder: "/path/to/binary",
                                }
                                button {
                                    onclick: handle_browse_binary,
                                    style: "padding: 8px 16px; background: #007bff; color: white; border: none; border-radius: 4px; cursor: pointer;",
                                    "Browse"
                                }
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
                                placeholder: "--flag value",
                            }
                        }
                    }
                }
                
                // Icon
                div {
                    style: "margin-bottom: 20px;",
                    label { style: "display: block; margin-bottom: 8px; font-weight: bold;", "Icon:" }
                    button {
                        onclick: handle_browse_icon,
                        style: "padding: 8px 16px; background: #007bff; color: white; border: none; border-radius: 4px; cursor: pointer;",
                        "Choose Icon"
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
                        placeholder: "Ctrl+1",
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
async fn invoke_create_app(new_app: NewApp) -> Result<i64, String> {
    use wasm_bindgen::prelude::*;
    
    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
        async fn invoke(cmd: &str, args: JsValue) -> JsValue;
    }
    
    let args = serde_wasm_bindgen::to_value(&new_app).map_err(|e| e.to_string())?;
    let result = invoke("create_app", args).await;
    serde_wasm_bindgen::from_value(result).map_err(|e| e.to_string())
}

#[cfg(target_family = "wasm")]
async fn invoke_get_all_apps() -> Result<Vec<crate::types::App>, String> {
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
async fn invoke_extract_icon(binary_path: String) -> Result<String, String> {
    use wasm_bindgen::prelude::*;
    
    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
        async fn invoke(cmd: &str, args: JsValue) -> JsValue;
    }
    
    let args = serde_wasm_bindgen::to_value(&serde_json::json!({ "binaryPath": binary_path }))
        .map_err(|e| e.to_string())?;
    
    let result = invoke("extract_icon_from_binary", args).await;
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


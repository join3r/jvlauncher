use dioxus::prelude::*;
use crate::state::AppState;
use crate::types::Settings;

#[component]
pub fn SettingsPanel() -> Element {
    let mut state = use_context::<Signal<AppState>>();
    let mut show_settings = use_signal(|| false);
    let mut theme = use_signal(|| "system".to_string());
    let mut grid_size = use_signal(|| 4);
    let mut start_at_login = use_signal(|| false);
    let mut global_shortcut = use_signal(|| "CommandOrControl+Space".to_string());

    // Load settings on mount
    use_effect(move || {
        spawn(async move {
            if let Ok(settings) = invoke_get_settings().await {
                theme.set(settings.theme);
                grid_size.set(settings.grid_size);
                start_at_login.set(settings.start_at_login);
                global_shortcut.set(settings.global_shortcut);
            }
        });
    });

    let handle_open_settings = move |_| {
        show_settings.set(true);
    };

    let handle_close = move |_| {
        show_settings.set(false);
    };

    let handle_save = move |_| {
        let theme_val = theme.read().clone();
        let grid_size_val = *grid_size.read();
        let start_at_login_val = *start_at_login.read();
        let shortcut_val = global_shortcut.read().clone();

        spawn(async move {
            let _ = invoke_update_setting("theme".to_string(), theme_val).await;
            let _ = invoke_update_setting("grid_size".to_string(), grid_size_val.to_string()).await;
            let _ = invoke_update_setting("start_at_login".to_string(), if start_at_login_val { "true" } else { "false" }.to_string()).await;
            let _ = invoke_update_setting("global_shortcut".to_string(), shortcut_val.clone()).await;
            
            // Update global shortcut
            let _ = invoke_update_global_shortcut(shortcut_val).await;
        });

        show_settings.set(false);
    };

    rsx! {
        div {
            // Settings icon button
            button {
                class: "settings-button",
                onclick: handle_open_settings,
                style: "position: fixed; top: 20px; right: 20px; width: 40px; height: 40px; border-radius: 50%; background: #f8f9fa; border: 1px solid #dee2e6; cursor: pointer; display: flex; align-items: center; justify-content: center; font-size: 20px;",
                "⚙️"
            }
            
            // Settings modal
            if *show_settings.read() {
                div {
                    class: "modal-overlay",
                    style: "position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 2000;",
                    onclick: handle_close,
                    
                    div {
                        class: "modal-content",
                        style: "background: white; padding: 30px; border-radius: 12px; max-width: 500px; width: 90%;",
                        onclick: move |evt: Event<MouseData>| evt.stop_propagation(),
                        
                        h2 { style: "margin-top: 0;", "Settings" }
                        
                        // Theme
                        div {
                            style: "margin-bottom: 20px;",
                            label { style: "display: block; margin-bottom: 8px; font-weight: bold;", "Theme:" }
                            select {
                                value: "{theme}",
                                oninput: move |evt| theme.set(evt.value()),
                                style: "width: 100%; padding: 8px; border: 1px solid #ccc; border-radius: 4px;",
                                
                                option { value: "system", "System" }
                                option { value: "light", "Light" }
                                option { value: "dark", "Dark" }
                            }
                        }
                        
                        // Grid Size
                        div {
                            style: "margin-bottom: 20px;",
                            label { style: "display: block; margin-bottom: 8px; font-weight: bold;", "Grid Size:" }
                            input {
                                r#type: "number",
                                value: "{grid_size}",
                                min: "2",
                                max: "10",
                                oninput: move |evt| {
                                    if let Ok(val) = evt.value().parse::<i32>() {
                                        grid_size.set(val);
                                    }
                                },
                                style: "width: 100%; padding: 8px; border: 1px solid #ccc; border-radius: 4px;",
                            }
                        }
                        
                        // Global Shortcut
                        div {
                            style: "margin-bottom: 20px;",
                            label { style: "display: block; margin-bottom: 8px; font-weight: bold;", "Global Shortcut:" }
                            input {
                                r#type: "text",
                                value: "{global_shortcut}",
                                oninput: move |evt| global_shortcut.set(evt.value()),
                                style: "width: 100%; padding: 8px; border: 1px solid #ccc; border-radius: 4px;",
                                placeholder: "CommandOrControl+Space",
                            }
                        }
                        
                        // Start at Login
                        div {
                            style: "margin-bottom: 20px;",
                            label {
                                style: "display: flex; align-items: center; cursor: pointer;",
                                input {
                                    r#type: "checkbox",
                                    checked: *start_at_login.read(),
                                    oninput: move |evt| start_at_login.set(evt.checked()),
                                    style: "margin-right: 8px;",
                                }
                                span { "Start at login" }
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
    }
}

// Tauri command invocations
#[cfg(target_family = "wasm")]
async fn invoke_get_settings() -> Result<Settings, String> {
    use wasm_bindgen::prelude::*;
    
    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
        async fn invoke(cmd: &str, args: JsValue) -> JsValue;
    }
    
    let result = invoke("get_settings", JsValue::NULL).await;
    serde_wasm_bindgen::from_value(result).map_err(|e| e.to_string())
}

#[cfg(target_family = "wasm")]
async fn invoke_update_setting(key: String, value: String) -> Result<(), String> {
    use wasm_bindgen::prelude::*;
    
    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
        async fn invoke(cmd: &str, args: JsValue) -> JsValue;
    }
    
    let args = serde_wasm_bindgen::to_value(&serde_json::json!({
        "key": key,
        "value": value
    })).map_err(|e| e.to_string())?;
    
    invoke("update_setting", args).await;
    Ok(())
}

#[cfg(target_family = "wasm")]
async fn invoke_update_global_shortcut(shortcut: String) -> Result<(), String> {
    // This would trigger a backend update
    invoke_update_setting("global_shortcut".to_string(), shortcut).await
}


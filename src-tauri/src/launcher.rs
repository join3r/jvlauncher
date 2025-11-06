use crate::database::{App, AppType, DbPool};
use crate::terminal::create_terminal_window;
use anyhow::{anyhow, Result};
use std::process::Command;
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

/// Launch an application based on its type
pub fn launch_app(app: &App, app_handle: &AppHandle, pool: &DbPool) -> Result<()> {
    match app.app_type {
        AppType::App => launch_application(app)?,
        AppType::Webapp => launch_webapp(app, app_handle, pool)?,
        AppType::Tui => launch_tui(app, app_handle)?,
    }
    Ok(())
}

/// Launch a native application
fn launch_application(app: &App) -> Result<()> {
    let binary_path = app.binary_path.as_ref()
        .ok_or_else(|| anyhow!("No binary path specified for application"))?;

    // Parse CLI parameters
    let args = if let Some(params) = &app.cli_params {
        shell_words::split(params).unwrap_or_default()
    } else {
        vec![]
    };

    // Launch the application
    #[cfg(target_os = "macos")]
    {
        // On macOS, use 'open' command for .app bundles
        if binary_path.ends_with(".app") {
            let mut cmd = Command::new("open");
            cmd.arg("-a").arg(binary_path);
            
            if !args.is_empty() {
                cmd.arg("--args");
                cmd.args(&args);
            }
            
            cmd.spawn()
                .map_err(|e| anyhow!("Failed to launch application: {}", e))?;
        } else {
            Command::new(binary_path)
                .args(&args)
                .spawn()
                .map_err(|e| anyhow!("Failed to launch application: {}", e))?;
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        Command::new(binary_path)
            .args(&args)
            .spawn()
            .map_err(|e| anyhow!("Failed to launch application: {}", e))?;
    }

    Ok(())
}

/// Launch a webapp in a dedicated webview window
fn launch_webapp(app: &App, app_handle: &AppHandle, pool: &DbPool) -> Result<()> {
    let url = app.url.as_ref()
        .ok_or_else(|| anyhow!("No URL specified for webapp"))?;

    let session_path = app.session_data_path.as_ref()
        .ok_or_else(|| anyhow!("No session data path specified"))?;

    // Create a unique window label for this webapp
    let window_label = format!("webapp_{}", app.id);

    // Check if window already exists
    if let Some(existing_window) = app_handle.get_webview_window(&window_label) {
        // If window exists, just show and focus it
        existing_window.show()?;
        existing_window.set_focus()?;
        return Ok(());
    }

    // Try to load saved window state
    let saved_state = crate::database::load_window_state(pool, app.id).ok().flatten();

    // Create new webview window with persistent session
    let mut builder = WebviewWindowBuilder::new(
        app_handle,
        &window_label,
        WebviewUrl::External(url.parse()?)
    )
    .title(&app.name)
    .resizable(true)
    .data_directory(std::path::PathBuf::from(session_path));

    // Add navigation bar initialization script if enabled (runs on every page load)
    if app.show_nav_controls.unwrap_or(false) {
        let original_url = url.clone();
        let nav_script = format!(r#"
(function() {{
    // Wait for DOM to be ready
    if (document.readyState === 'loading') {{
        document.addEventListener('DOMContentLoaded', initNavBar);
    }} else {{
        initNavBar();
    }}

    function initNavBar() {{
        // Check if nav bar already exists (to prevent duplicates on page navigation)
        if (document.getElementById('jvlauncher-nav-bar')) {{
            return;
        }}

        // Create navigation bar
        const navBar = document.createElement('div');
        navBar.id = 'jvlauncher-nav-bar';
        navBar.style.cssText = `
            position: fixed;
            top: 0;
            left: 0;
            right: 0;
            height: 44px;
            display: flex;
            align-items: center;
            gap: 8px;
            padding: 0 12px;
            background: rgba(245, 245, 247, 0.8);
            backdrop-filter: blur(20px);
            -webkit-backdrop-filter: blur(20px);
            border-bottom: 0.5px solid rgba(0, 0, 0, 0.1);
            z-index: 999999;
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
        `;

        // Dark mode support
        if (window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches) {{
            navBar.style.background = 'rgba(44, 44, 46, 0.8)';
            navBar.style.borderBottom = '0.5px solid rgba(255, 255, 255, 0.1)';
        }}

        // Create button helper
        function createButton(text, onClick) {{
            const btn = document.createElement('button');
            btn.textContent = text;
            btn.style.cssText = `
                appearance: none;
                border: none;
                background: rgba(0, 0, 0, 0.05);
                color: #1d1d1f;
                padding: 6px 12px;
                border-radius: 6px;
                font-size: 14px;
                cursor: pointer;
                transition: background 0.15s;
                font-weight: 500;
            `;

            if (window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches) {{
                btn.style.background = 'rgba(255, 255, 255, 0.1)';
                btn.style.color = '#f5f5f7';
            }}

            btn.addEventListener('mouseenter', () => {{
                if (window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches) {{
                    btn.style.background = 'rgba(255, 255, 255, 0.15)';
                }} else {{
                    btn.style.background = 'rgba(0, 0, 0, 0.08)';
                }}
            }});

            btn.addEventListener('mouseleave', () => {{
                if (window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches) {{
                    btn.style.background = 'rgba(255, 255, 255, 0.1)';
                }} else {{
                    btn.style.background = 'rgba(0, 0, 0, 0.05)';
                }}
            }});

            btn.addEventListener('click', onClick);
            return btn;
        }}

        // Create buttons
        const backBtn = createButton('←', () => window.history.back());
        const forwardBtn = createButton('→', () => window.history.forward());
        const homeBtn = createButton('⌂', () => window.location.href = '{}');

        // Create URL display
        const urlDisplay = document.createElement('div');
        urlDisplay.id = 'jvlauncher-url-display';
        urlDisplay.style.cssText = `
            flex: 1;
            margin-left: 12px;
            padding: 6px 12px;
            background: rgba(0, 0, 0, 0.03);
            border-radius: 6px;
            font-size: 12px;
            color: #6e6e73;
            overflow: hidden;
            text-overflow: ellipsis;
            white-space: nowrap;
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, monospace;
        `;

        if (window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches) {{
            urlDisplay.style.background = 'rgba(255, 255, 255, 0.05)';
            urlDisplay.style.color = '#98989d';
        }}

        // Update URL display
        function updateURL() {{
            urlDisplay.textContent = window.location.href;
        }}
        updateURL();

        // Listen for URL changes (for SPAs and history navigation)
        window.addEventListener('popstate', updateURL);

        // Override pushState and replaceState to catch SPA navigation
        const originalPushState = history.pushState;
        const originalReplaceState = history.replaceState;

        history.pushState = function() {{
            originalPushState.apply(this, arguments);
            updateURL();
        }};

        history.replaceState = function() {{
            originalReplaceState.apply(this, arguments);
            updateURL();
        }};

        navBar.appendChild(backBtn);
        navBar.appendChild(forwardBtn);
        navBar.appendChild(homeBtn);
        navBar.appendChild(urlDisplay);

        // Insert at the beginning of body
        if (document.body) {{
            document.body.insertBefore(navBar, document.body.firstChild);
            // Add padding to body to prevent content overlap
            const currentPadding = parseInt(window.getComputedStyle(document.body).paddingTop) || 0;
            document.body.style.paddingTop = (currentPadding + 44) + 'px';
        }}
    }}
}})();
"#, original_url);

        builder = builder.initialization_script(&nav_script);
    }

    // Apply saved window state if available, otherwise use defaults
    if let Some(state) = saved_state {
        builder = builder
            .inner_size(state.width as f64, state.height as f64)
            .position(state.x as f64, state.y as f64);
    } else {
        builder = builder
            .inner_size(1200.0, 800.0)
            .center();
    }

    let window = builder.build()?;

    // Set up event handler to save window state when it closes
    let app_id = app.id;
    let pool_clone = pool.clone();
    let window_clone = window.clone();
    window.on_window_event(move |event| {
        if let tauri::WindowEvent::CloseRequested { .. } = event {
            // Get the window's current position and size
            if let Ok(position) = window_clone.outer_position() {
                if let Ok(size) = window_clone.outer_size() {
                    let state = crate::database::WindowState {
                        x: position.x,
                        y: position.y,
                        width: size.width as i32,
                        height: size.height as i32,
                    };
                    // Save the window state to database
                    let _ = crate::database::save_window_state(&pool_clone, app_id, &state);
                }
            }
        }
    });

    // Note: Webapp windows close normally when user clicks X or presses Cmd+Q
    // The app won't quit because the system tray keeps it running

    Ok(())
}

/// Launch a TUI application in a terminal window
fn launch_tui(app: &App, app_handle: &AppHandle) -> Result<()> {
    let binary_path = app.binary_path.as_ref()
        .ok_or_else(|| anyhow!("No binary path specified for TUI application"))?;

    let args = if let Some(params) = &app.cli_params {
        shell_words::split(params).unwrap_or_default()
    } else {
        vec![]
    };

    // Launch in terminal window
    create_terminal_window(app_handle, &app.name, binary_path, &args)?;

    Ok(())
}

/// Helper module to parse shell-like command line strings
mod shell_words {
    pub fn split(input: &str) -> Option<Vec<String>> {
        let mut words = Vec::new();
        let mut current_word = String::new();
        let mut in_quotes = false;
        let mut quote_char = ' ';
        let mut escape_next = false;

        for ch in input.chars() {
            if escape_next {
                current_word.push(ch);
                escape_next = false;
                continue;
            }

            match ch {
                '\\' => {
                    escape_next = true;
                }
                '"' | '\'' => {
                    if in_quotes {
                        if ch == quote_char {
                            in_quotes = false;
                        } else {
                            current_word.push(ch);
                        }
                    } else {
                        in_quotes = true;
                        quote_char = ch;
                    }
                }
                ' ' | '\t' => {
                    if in_quotes {
                        current_word.push(ch);
                    } else if !current_word.is_empty() {
                        words.push(current_word.clone());
                        current_word.clear();
                    }
                }
                _ => {
                    current_word.push(ch);
                }
            }
        }

        if !current_word.is_empty() {
            words.push(current_word);
        }

        Some(words)
    }
}


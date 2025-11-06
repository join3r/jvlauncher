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
    .data_directory(std::path::PathBuf::from(session_path))
    // Set a standard browser user agent to avoid being blocked by sites like Cloudflare Access
    .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36");

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

        // Helper to check if dark mode is active
        // For webapp windows, we just use system preference since they don't have
        // permission to access app settings
        function isDarkMode() {{
            return window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches;
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
            backdrop-filter: blur(20px);
            -webkit-backdrop-filter: blur(20px);
            z-index: 2147483647;
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            pointer-events: auto;
        `;

        // Function to update nav bar theme
        function updateNavBarTheme() {{
            if (isDarkMode()) {{
                navBar.style.background = 'rgba(44, 44, 46, 0.95)';
                navBar.style.borderBottom = '0.5px solid rgba(255, 255, 255, 0.1)';
            }} else {{
                navBar.style.background = 'rgba(245, 245, 247, 0.95)';
                navBar.style.borderBottom = '0.5px solid rgba(0, 0, 0, 0.1)';
            }}
        }}

        // Set initial theme
        updateNavBarTheme();

        // Listen for theme changes via data-theme attribute
        const observer = new MutationObserver((mutations) => {{
            mutations.forEach((mutation) => {{
                if (mutation.type === 'attributes' && mutation.attributeName === 'data-theme') {{
                    updateNavBarTheme();
                }}
            }});
        }});
        observer.observe(document.documentElement, {{ attributes: true, attributeFilter: ['data-theme'] }});

        // Also listen for system theme changes (when theme is set to 'system')
        if (window.matchMedia) {{
            window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {{
                // Only update if data-theme is not set (system theme mode)
                if (!document.documentElement.hasAttribute('data-theme')) {{
                    updateNavBarTheme();
                }}
            }});
        }}

        // Create button helper
        function createButton(text, onClick) {{
            const btn = document.createElement('button');
            btn.textContent = text;
            btn.style.cssText = `
                appearance: none;
                border: none;
                padding: 6px 12px;
                border-radius: 6px;
                font-size: 14px;
                cursor: pointer;
                transition: background 0.15s;
                font-weight: 500;
            `;

            // Function to update button theme
            function updateButtonTheme() {{
                if (isDarkMode()) {{
                    btn.style.background = 'rgba(255, 255, 255, 0.1)';
                    btn.style.color = '#f5f5f7';
                }} else {{
                    btn.style.background = 'rgba(0, 0, 0, 0.05)';
                    btn.style.color = '#1d1d1f';
                }}
            }}

            // Set initial theme
            updateButtonTheme();

            // Listen for theme changes via data-theme attribute
            const btnObserver = new MutationObserver((mutations) => {{
                mutations.forEach((mutation) => {{
                    if (mutation.type === 'attributes' && mutation.attributeName === 'data-theme') {{
                        updateButtonTheme();
                    }}
                }});
            }});
            btnObserver.observe(document.documentElement, {{ attributes: true, attributeFilter: ['data-theme'] }});

            // Also listen for system theme changes
            if (window.matchMedia) {{
                window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {{
                    if (!document.documentElement.hasAttribute('data-theme')) {{
                        updateButtonTheme();
                    }}
                }});
            }}

            btn.addEventListener('mouseenter', () => {{
                if (isDarkMode()) {{
                    btn.style.background = 'rgba(255, 255, 255, 0.15)';
                }} else {{
                    btn.style.background = 'rgba(0, 0, 0, 0.08)';
                }}
            }});

            btn.addEventListener('mouseleave', () => {{
                if (isDarkMode()) {{
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
            border-radius: 6px;
            font-size: 12px;
            overflow: hidden;
            text-overflow: ellipsis;
            white-space: nowrap;
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, monospace;
        `;

        // Function to update URL display theme
        function updateUrlDisplayTheme() {{
            if (isDarkMode()) {{
                urlDisplay.style.background = 'rgba(255, 255, 255, 0.05)';
                urlDisplay.style.color = '#98989d';
            }} else {{
                urlDisplay.style.background = 'rgba(0, 0, 0, 0.03)';
                urlDisplay.style.color = '#6e6e73';
            }}
        }}

        // Set initial theme
        updateUrlDisplayTheme();

        // Listen for theme changes via data-theme attribute
        const urlObserver = new MutationObserver((mutations) => {{
            mutations.forEach((mutation) => {{
                if (mutation.type === 'attributes' && mutation.attributeName === 'data-theme') {{
                    updateUrlDisplayTheme();
                }}
            }});
        }});
        urlObserver.observe(document.documentElement, {{ attributes: true, attributeFilter: ['data-theme'] }});

        // Also listen for system theme changes
        if (window.matchMedia) {{
            window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {{
                if (!document.documentElement.hasAttribute('data-theme')) {{
                    updateUrlDisplayTheme();
                }}
            }});
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
            // Append to body (not insertBefore) so it overlays on top
            document.body.appendChild(navBar);

            // Inject comprehensive styles to push all content down
            const style = document.createElement('style');
            style.id = 'jvlauncher-nav-spacing';
            style.textContent = `
                /* Ensure the nav bar stays on top of everything */
                #jvlauncher-nav-bar {{
                    z-index: 2147483647 !important;
                    position: fixed !important;
                    top: 0 !important;
                    left: 0 !important;
                    right: 0 !important;
                }}

                /* Push all body content down by 44px to make room for nav bar */
                body {{
                    padding-top: 44px !important;
                    box-sizing: border-box !important;
                }}

                /* Adjust viewport height for fixed elements */
                html {{
                    scroll-padding-top: 44px !important;
                }}
            `;
            document.head.appendChild(style);

            // Function to adjust fixed/sticky/absolute positioned elements
            function adjustFixedElements() {{
                const allElements = document.querySelectorAll('*:not(#jvlauncher-nav-bar):not(#jvlauncher-nav-spacing)');
                allElements.forEach(el => {{
                    // Skip if already adjusted
                    if (el.getAttribute('data-jvlauncher-adjusted') === 'true') {{
                        return;
                    }}

                    const style = window.getComputedStyle(el);
                    const position = style.position;

                    if (position === 'fixed' || position === 'sticky' || position === 'absolute') {{
                        const currentTop = style.top;
                        const topValue = parseInt(currentTop) || 0;

                        // Get the element's bounding rect to check if it's actually at the top
                        const rect = el.getBoundingClientRect();

                        // Adjust if element is at or near the top of viewport (within 50px)
                        // This catches elements that might be slightly offset
                        if (rect.top >= -10 && rect.top < 50) {{
                            const newTop = (topValue + 44);
                            el.style.top = newTop + 'px';
                            el.setAttribute('data-jvlauncher-adjusted', 'true');
                        }}
                    }}
                }});
            }}

            // Run adjustment multiple times to catch dynamically loaded content
            setTimeout(adjustFixedElements, 50);
            setTimeout(adjustFixedElements, 100);
            setTimeout(adjustFixedElements, 300);
            setTimeout(adjustFixedElements, 500);
            setTimeout(adjustFixedElements, 1000);
            setTimeout(adjustFixedElements, 2000);

            // Also run on DOM changes
            const observer = new MutationObserver(() => {{
                setTimeout(adjustFixedElements, 50);
            }});
            observer.observe(document.body, {{ childList: true, subtree: true, attributes: true, attributeFilter: ['style', 'class'] }});
        }}
    }}
}})();
"#, original_url);

        builder = builder.initialization_script(&nav_script);
    }

    // Add external link handling script if enabled
    if app.open_external_links.unwrap_or(false) {
        let webapp_url = url.clone();
        let external_links_script = format!(r#"
(function() {{
    // Wait for Tauri API to be ready
    function waitForTauri(callback, maxAttempts = 50) {{
        let attempts = 0;
        const checkTauri = setInterval(() => {{
            attempts++;
            if (window.__TAURI__ && window.__TAURI__.shell) {{
                clearInterval(checkTauri);
                callback();
            }} else if (attempts >= maxAttempts) {{
                clearInterval(checkTauri);
                console.warn('Tauri API not available after', maxAttempts, 'attempts. External link handling disabled.');
            }}
        }}, 100);
    }}

    // Wait for DOM to be ready
    if (document.readyState === 'loading') {{
        document.addEventListener('DOMContentLoaded', () => waitForTauri(initExternalLinkHandler));
    }} else {{
        waitForTauri(initExternalLinkHandler);
    }}

    function initExternalLinkHandler() {{
        // Get the base domain of the webapp
        const webappUrl = new URL('{}');
        const webappDomain = webappUrl.hostname;

        // Function to check if a link should open externally
        function shouldOpenExternally(link) {{
            // Check if link has target="_blank" or similar
            const target = link.getAttribute('target');
            if (target && target !== '_self') {{
                return true;
            }}

            // Check if link is to a different domain
            try {{
                const linkUrl = new URL(link.href);
                if (linkUrl.hostname !== webappDomain) {{
                    return true;
                }}
            }} catch (e) {{
                // Invalid URL, let it handle normally
                return false;
            }}

            return false;
        }}

        // Handle click events on links
        document.addEventListener('click', function(e) {{
            // Find the closest anchor element
            let target = e.target;
            while (target && target.tagName !== 'A') {{
                target = target.parentElement;
            }}

            if (!target || target.tagName !== 'A') {{
                return;
            }}

            // Check if this link should open externally
            if (shouldOpenExternally(target)) {{
                e.preventDefault();
                e.stopPropagation();

                // Open in default browser using Tauri shell API
                if (window.__TAURI__ && window.__TAURI__.shell) {{
                    window.__TAURI__.shell.open(target.href).catch(err => {{
                        console.error('Failed to open external link:', err);
                    }});
                }} else {{
                    console.warn('Tauri shell API not available');
                }}
            }}
        }}, true); // Use capture phase to intercept before other handlers

        console.log('External link handler initialized for domain:', webappDomain);
    }}
}})();
"#, webapp_url);

        builder = builder.initialization_script(&external_links_script);
    }

    // OAuth support - when enabled, the webapp can handle OAuth flows
    // The enable_oauth setting is stored and can be used for future OAuth-specific handling
    // For now, OAuth flows work naturally within the webview with persistent sessions
    if app.enable_oauth.unwrap_or(false) {
        // OAuth is enabled for this webapp
        // The persistent session (data_directory) already handles cookies and tokens
        // Future enhancements could include:
        // - Custom OAuth redirect handling
        // - Token storage and management
        // - OAuth-specific security policies
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

    // Register window with activity tracker for auto-close feature
    if let Some(tracker) = app_handle.try_state::<crate::webapp_auto_close::WebappActivityTracker>() {
        tracker.register_window(window_label.clone(), app.auto_close_timeout);
    }

    // Register window with shortcut manager for toggle behavior
    crate::shortcut_manager::register_app_window(app.id, window_label.clone());

    // Set up event handler to save window state when it closes and handle auto-close
    let app_id = app.id;
    let pool_clone = pool.clone();
    let window_clone = window.clone();
    let window_label_for_events = window_label.clone();
    let app_handle_clone = app_handle.clone();
    window.on_window_event(move |event| {
        match event {
            tauri::WindowEvent::CloseRequested { .. } => {
                // Unregister from activity tracker
                if let Some(tracker) = app_handle_clone.try_state::<crate::webapp_auto_close::WebappActivityTracker>() {
                    tracker.unregister_window(&window_label_for_events);
                }

                // Unregister from shortcut manager
                crate::shortcut_manager::unregister_app_window(app_id);

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
            tauri::WindowEvent::Focused(focused) => {
                // Update activity tracker when window gains focus
                if *focused {
                    if let Some(tracker) = app_handle_clone.try_state::<crate::webapp_auto_close::WebappActivityTracker>() {
                        tracker.update_activity(&window_label_for_events);
                    }
                }
            }
            _ => {}
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

    // Create a unique window label for this TUI app
    let window_label = format!("tui_{}", app.id);

    // Check if window already exists
    if let Some(existing_window) = app_handle.get_webview_window(&window_label) {
        // If window exists, just show and focus it
        existing_window.show()?;
        existing_window.set_focus()?;
        return Ok(());
    }

    // Launch in terminal window
    create_terminal_window(app_handle, app.id, &window_label, &app.name, binary_path, &args)?;

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


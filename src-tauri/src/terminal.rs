use anyhow::Result;
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use std::io::Read;
use std::sync::{Arc, Mutex};
use std::thread;
use tauri::{AppHandle, Manager, Emitter};

/// Create a terminal window and run a command in it
pub fn create_terminal_window(
    app_handle: &AppHandle,
    app_id: i64,
    window_label: &str,
    title: &str,
    command: &str,
    args: &[String],
    always_on_top: bool,
) -> Result<()> {
    let window_label = window_label.to_string();

    // Create PTY with proper size
    let pty_system = native_pty_system();
    let pair = pty_system.openpty(PtySize {
        rows: 24,
        cols: 80,
        pixel_width: 0,
        pixel_height: 0,
    })?;

    // Build command with essential environment variables
    let mut cmd = CommandBuilder::new(command);
    for arg in args {
        cmd.arg(arg);
    }

    // Set essential environment variables for terminal applications
    cmd.env("TERM", "xterm-256color");
    cmd.env("COLORTERM", "truecolor");
    cmd.env("TERM_PROGRAM", "jvlauncher");

    // Preserve PATH and other important environment variables
    if let Ok(path) = std::env::var("PATH") {
        cmd.env("PATH", path);
    }
    if let Ok(home) = std::env::var("HOME") {
        cmd.env("HOME", home);
    }
    if let Ok(user) = std::env::var("USER") {
        cmd.env("USER", user);
    }
    if let Ok(shell) = std::env::var("SHELL") {
        cmd.env("SHELL", shell);
    }

    // Spawn command in PTY
    let child = pair.slave.spawn_command(cmd)?;

    // Read output from PTY using raw byte reading (not line-based)
    let mut reader = pair.master.try_clone_reader()?;
    let app_handle_clone = app_handle.clone();
    let window_label_clone = window_label.clone();

    thread::spawn(move || {
        let mut buffer = [0u8; 8192];

        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break, // EOF
                Ok(n) => {
                    // Convert bytes to string, preserving ANSI escape sequences
                    if let Ok(output) = String::from_utf8(buffer[..n].to_vec()) {
                        // Emit terminal output event
                        let _ = app_handle_clone.emit_to(
                            &window_label_clone,
                            "terminal-output",
                            output
                        );
                    }
                }
                Err(_) => break,
            }
        }
    });

    // Get writer from PTY master for input handling
    let writer = Arc::new(Mutex::new(pair.master.take_writer()?));

    // Store the master PTY for resizing (we need to keep it)
    let master = Arc::new(Mutex::new(pair.master));

    // Create window to display terminal
    let mut builder = tauri::WebviewWindowBuilder::new(
        app_handle,
        &window_label,
        tauri::WebviewUrl::App("terminal.html".into())
    )
    .title(title)
    .inner_size(800.0, 600.0);

    // Apply always on top setting
    if always_on_top {
        builder = builder.always_on_top(true);
    }

    let window = builder.build()?;

    // Ensure the new window is brought to front
    #[cfg(target_os = "macos")]
    crate::macos_delegate::bring_window_to_front(&window);

    // Register window with shortcut manager for toggle behavior
    crate::shortcut_manager::register_app_window(app_id, window_label.clone());

    // Store child process for cleanup
    let child = Arc::new(Mutex::new(child));

    // Store writer and master in window state for input handling and resizing
    let writer_clone = Arc::clone(&writer);
    let master_clone = Arc::clone(&master);
    let window_label_for_input = window_label.clone();

    // Get or create terminal state
    if let Some(state) = app_handle.try_state::<TerminalState>() {
        if let Ok(mut windows) = state.windows.lock() {
            windows.insert(window_label_for_input.clone(), TerminalHandle {
                writer: writer_clone,
                master: master_clone,
            });
        }
    }

    // Handle window close event to kill process and cleanup
    let child_clone = Arc::clone(&child);
    let app_handle_for_cleanup = app_handle.clone();
    let window_label_for_cleanup = window_label.clone();
    window.on_window_event(move |event| {
        if let tauri::WindowEvent::CloseRequested { .. } = event {
            // Kill the child process
            if let Ok(mut child) = child_clone.lock() {
                let _ = child.kill();
            }

            // Unregister from shortcut manager
            crate::shortcut_manager::unregister_app_window(app_id);

            // Remove from terminal state
            if let Some(state) = app_handle_for_cleanup.try_state::<TerminalState>() {
                if let Ok(mut windows) = state.windows.lock() {
                    windows.remove(&window_label_for_cleanup);
                }
            }
        }
    });

    Ok(())
}

/// State to manage terminal PTY writers for input handling
pub struct TerminalState {
    pub windows: Arc<Mutex<std::collections::HashMap<String, TerminalHandle>>>,
}

/// Handle to a terminal for sending input and resizing
pub struct TerminalHandle {
    pub writer: Arc<Mutex<Box<dyn std::io::Write + Send>>>,
    pub master: Arc<Mutex<Box<dyn portable_pty::MasterPty + Send>>>,
}


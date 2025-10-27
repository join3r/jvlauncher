use anyhow::Result;
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use std::io::{BufRead, BufReader};
use std::sync::{Arc, Mutex};
use std::thread;
use tauri::{AppHandle, Manager, Emitter};

/// Create a terminal window and run a command in it
pub fn create_terminal_window(
    app_handle: &AppHandle,
    title: &str,
    command: &str,
    args: &[String],
) -> Result<()> {
    let window_label = format!("terminal_{}", uuid::Uuid::new_v4());
    
    // Create PTY
    let pty_system = native_pty_system();
    let pair = pty_system.openpty(PtySize {
        rows: 24,
        cols: 80,
        pixel_width: 0,
        pixel_height: 0,
    })?;

    // Build command
    let mut cmd = CommandBuilder::new(command);
    for arg in args {
        cmd.arg(arg);
    }

    // Spawn command in PTY
    let child = pair.slave.spawn_command(cmd)?;
    
    // Read output from PTY
    let reader = pair.master.try_clone_reader()?;
    let app_handle_clone = app_handle.clone();
    let window_label_clone = window_label.clone();
    
    thread::spawn(move || {
        let mut buf_reader = BufReader::new(reader);
        let mut line = String::new();
        
        while buf_reader.read_line(&mut line).is_ok() {
            if line.is_empty() {
                break;
            }
            
            // Emit terminal output event
            let _ = app_handle_clone.emit_to(
                &window_label_clone,
                "terminal-output",
                line.clone()
            );
            
            line.clear();
        }
    });

    // Create window to display terminal
    let _window = tauri::WebviewWindowBuilder::new(
        app_handle,
        &window_label,
        tauri::WebviewUrl::App("terminal.html".into())
    )
    .title(title)
    .inner_size(800.0, 600.0)
    .build()?;

    // Store child process for cleanup
    let child = Arc::new(Mutex::new(child));
    
    // Handle window close event to kill process
    let window = app_handle.get_webview_window(&window_label);
    if let Some(window) = window {
        let child_clone = Arc::clone(&child);
        window.on_window_event(move |event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                if let Ok(mut child) = child_clone.lock() {
                    let _ = child.kill();
                }
            }
        });
    }

    Ok(())
}

// UUID generation helper
mod uuid {
    use std::time::{SystemTime, UNIX_EPOCH};
    
    pub struct Uuid;
    
    impl Uuid {
        pub fn new_v4() -> String {
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            format!("{:x}", nanos)
        }
    }
}


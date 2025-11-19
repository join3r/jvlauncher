#[cfg(target_os = "macos")]
use cocoa::appkit::NSApp;
#[cfg(target_os = "macos")]
use cocoa::base::{id, nil};
#[cfg(target_os = "macos")]
use objc::runtime::{Object, Sel};
#[cfg(target_os = "macos")]
use objc::{class, msg_send, sel, sel_impl};

#[cfg(target_os = "macos")]
pub fn prevent_app_termination() {
    unsafe {
        let app = NSApp();

        // Set up a custom delegate method to prevent termination
        let superclass = class!(NSObject);
        let mut decl = objc::declare::ClassDecl::new("JVLauncherDelegate", superclass).unwrap();

        // NSTerminateCancel = 0
        extern "C" fn application_should_terminate(
            _self: &Object,
            _cmd: Sel,
            _sender: id,
        ) -> u64 {
            // Close the key window (the focused window) instead of quitting the app
            // For the main launcher window, we'll hide it instead of closing
            unsafe {
                let app = NSApp();
                let key_window: id = msg_send![app, keyWindow];

                if key_window != nil {
                    // Get the window title to determine if it's the launcher
                    let title: id = msg_send![key_window, title];
                    let title_str: *const i8 = msg_send![title, UTF8String];
                    let title_string = if !title_str.is_null() {
                        std::ffi::CStr::from_ptr(title_str).to_string_lossy().to_string()
                    } else {
                        String::new()
                    };

                    if title_string == "jvlauncher" {
                        // Hide the launcher window instead of closing it
                        let _: () = msg_send![key_window, orderOut: nil];
                    } else {
                        // Close other windows (webapps, terminals)
                        let _: () = msg_send![key_window, close];
                    }
                }
            }

            // Always return NSTerminateCancel (0) to prevent app termination
            // The app only quits via the tray menu
            0
        }

        decl.add_method(
            sel!(applicationShouldTerminate:),
            application_should_terminate as extern "C" fn(&Object, Sel, id) -> u64,
        );

        let custom_delegate_class = decl.register();
        let custom_delegate: id = msg_send![custom_delegate_class, new];

        let _: () = msg_send![app, setDelegate: custom_delegate];
    }
}

#[cfg(not(target_os = "macos"))]
pub fn prevent_app_termination() {
    // No-op on non-macOS platforms
}

/// Get the bundle identifier of the currently frontmost (focused) application
/// Returns None if we can't determine the frontmost app or if it's jvlauncher itself
#[cfg(target_os = "macos")]
pub fn get_frontmost_app_bundle_id() -> Option<String> {
    unsafe {
        // Get shared workspace
        let workspace: id = msg_send![class!(NSWorkspace), sharedWorkspace];
        if workspace == nil {
            return None;
        }

        // Get frontmost application
        let frontmost_app: id = msg_send![workspace, frontmostApplication];
        if frontmost_app == nil {
            return None;
        }

        // Get bundle identifier
        let bundle_id: id = msg_send![frontmost_app, bundleIdentifier];
        if bundle_id == nil {
            return None;
        }

        // Convert NSString to Rust String
        let bundle_id_str: *const i8 = msg_send![bundle_id, UTF8String];
        if bundle_id_str.is_null() {
            return None;
        }

        let bundle_id_string = std::ffi::CStr::from_ptr(bundle_id_str)
            .to_string_lossy()
            .to_string();

        // Don't track if the frontmost app is jvlauncher itself
        if bundle_id_string == "com.jvlauncher.app" {
            return None;
        }

        Some(bundle_id_string)
    }
}

#[cfg(not(target_os = "macos"))]
pub fn get_frontmost_app_bundle_id() -> Option<String> {
    None
}

/// Activate (bring to front) an application by its bundle identifier
/// Returns true if successful, false otherwise
#[cfg(target_os = "macos")]
pub fn activate_app_by_bundle_id(bundle_id: &str) -> bool {
    unsafe {
        // Get shared workspace
        let workspace: id = msg_send![class!(NSWorkspace), sharedWorkspace];
        if workspace == nil {
            return false;
        }

        // Get all running applications
        let running_apps: id = msg_send![workspace, runningApplications];
        if running_apps == nil {
            return false;
        }

        // Get the count of running applications
        let count: usize = msg_send![running_apps, count];

        // Convert bundle_id to NSString
        let bundle_id_nsstring: id = msg_send![class!(NSString), stringWithUTF8String: bundle_id.as_ptr()];
        if bundle_id_nsstring == nil {
            return false;
        }

        // Iterate through running applications to find the one with matching bundle ID
        for i in 0..count {
            let app: id = msg_send![running_apps, objectAtIndex: i];
            if app == nil {
                continue;
            }

            let app_bundle_id: id = msg_send![app, bundleIdentifier];
            if app_bundle_id == nil {
                continue;
            }

            // Compare bundle IDs
            let is_equal: bool = msg_send![app_bundle_id, isEqualToString: bundle_id_nsstring];
            if is_equal {
                // Found the app, activate it
                // Use default activation (0) to prevent hiding other apps' windows
                // Previously used NSApplicationActivateIgnoringOtherApps (2) which could hide other windows
                let options: usize = 0;
                let success: bool = msg_send![app, activateWithOptions: options];
                return success;
            }
        }

        false
    }
}

#[cfg(not(target_os = "macos"))]
pub fn activate_app_by_bundle_id(_bundle_id: &str) -> bool {
    false
}

/// Bring a Tauri window to front using native macOS APIs
/// This is more reliable than Tauri's set_focus() for non-always-on-top windows
#[cfg(target_os = "macos")]
pub fn bring_window_to_front(window: &tauri::WebviewWindow) {
    use cocoa::base::nil;
    use objc::runtime::YES;

    unsafe {
        // Get the native NSWindow
        if let Ok(ns_window) = window.ns_window() {
            let ns_window = ns_window as id;

            // Check if window is minimized and deminiaturize if needed
            let is_miniaturized: bool = msg_send![ns_window, isMiniaturized];
            if is_miniaturized {
                let _: () = msg_send![ns_window, deminiaturize: nil];
            }

            // Make sure window is visible
            let _: () = msg_send![ns_window, setIsVisible: YES];

            // Activate the application using NSRunningApplication
            let current_app: id = msg_send![class!(NSRunningApplication), currentApplication];
            let options: usize = 1 << 1; // NSApplicationActivateIgnoringOtherApps
            let _: bool = msg_send![current_app, activateWithOptions: options];

            // Order front regardless of app activation state
            let _: () = msg_send![ns_window, orderFrontRegardless];
            
            // Ensure it's the key window
            let _: () = msg_send![ns_window, makeKeyWindow];
        }
    }
}

#[cfg(not(target_os = "macos"))]
pub fn bring_window_to_front(_window: &tauri::WebviewWindow) {
    // No-op on non-macOS platforms
}

/// Helper to safely switch focus from launcher to target window
/// Handles the timing issues with hiding the launcher window on macOS
pub fn switch_focus_and_hide_launcher(app_handle: &tauri::AppHandle, target_window: &tauri::WebviewWindow) {
    use tauri::Manager;
    let target_clone = target_window.clone();
    let app_handle_clone = app_handle.clone();
    
    std::thread::spawn(move || {
        // Wait for macOS animations/focus switching to settle
        std::thread::sleep(std::time::Duration::from_millis(150));
        
        let app_handle_for_closure = app_handle_clone.clone();
        // Run UI operations on main thread
        let _ = app_handle_clone.run_on_main_thread(move || {
            // Hide launcher
            if let Some(main_window) = app_handle_for_closure.get_webview_window("main") {
                let _ = main_window.hide();
            }
            
            // Force activate target window
            #[cfg(target_os = "macos")]
            bring_window_to_front(&target_clone);
            
            // Ensure internal state is updated
            let _ = target_clone.set_focus();
        });
    });
}


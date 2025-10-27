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
        let mut decl = objc::declare::ClassDecl::new("AppLauncherDelegate", superclass).unwrap();

        // NSTerminateCancel = 0
        extern "C" fn application_should_terminate(
            _self: &Object,
            _cmd: Sel,
            _sender: id,
        ) -> u64 {
            eprintln!("!!! applicationShouldTerminate called - preventing app termination");

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

                    eprintln!("!!! Key window title: '{}'", title_string);

                    if title_string == "App Launcher" {
                        // Hide the launcher window instead of closing it
                        eprintln!("!!! Hiding launcher window");
                        let _: () = msg_send![key_window, orderOut: nil];
                    } else {
                        // Close other windows (webapps, terminals)
                        eprintln!("!!! Closing window: {}", title_string);
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

        eprintln!("!!! Custom macOS delegate installed to prevent Cmd+Q termination");
    }
}

#[cfg(not(target_os = "macos"))]
pub fn prevent_app_termination() {
    // No-op on non-macOS platforms
}


// SPDX-FileCopyrightText: © 2026 Iain Nicol
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use ty::{iface::Procedure, types, vba_defn};

// TODO: use this in sem.
pub fn signatures() -> Vec<Procedure> {
    // TODO: full signature
    let msg_box = vba_defn!(Function MsgBox(Prompt));
    // let msg_box = vba_defn!(Function MsgBox(Prompt, [Buttons As VbMsgBoxStyle = vbOKOnly], [Title], [HelpFile], [Context]) As VbMsgBoxResult);
    vec![msg_box]
}

// TODO: define a trait and impl it.
// That way I can unit test.

// TODO: xplat (or rather, port to win32 and gtk-4)

// TODO: full signature, return type. (Update codegen for both).
#[unsafe(no_mangle)]
#[cfg(target_os = "macos")]
pub extern "C" fn msg_box(prompt: types::String) {
    use objc2_core_foundation::{
        CFOptionFlags, CFString, CFUserNotification, kCFUserNotificationPlainAlertLevel,
    };

    let prompt = CFString::from_str(&prompt.to_rust_string_lossy());
    let title = CFString::from_str("");
    let mut response: CFOptionFlags = 0;

    // Note that CFUserNotificationDisplayAlert always displays an icon.
    // IMHO this looks weird, but it's a macOS style. Switching to NSAlert would not help: it defaults to the app icon, only hiding it if presented as a sheet on an app window.

    // SAFETY: Neither alert_header nor response are null, satisfying <https://developer.apple.com/documentation/corefoundation/cfusernotificationdisplayalert(_:_:_:_:_:_:_:_:_:_:_:)>.
    // A valid pointer is passed for response.
    unsafe {
        CFUserNotification::display_alert(
            0.0,
            kCFUserNotificationPlainAlertLevel,
            None,
            None,
            None,
            Some(&title),
            Some(&prompt),
            Some(&CFString::from_static_str("OK")),
            None,
            None,
            &mut response,
        );
    }
}

#[unsafe(no_mangle)]
#[cfg(target_os = "linux")]
pub extern "C" fn msg_box(prompt: types::String) {
    use gtk::{
        AlertDialog,
        Application, ApplicationWindow,
        gio::{Cancellable, prelude::{ApplicationExt, ApplicationExtManual}},
        glib::object::ObjectExt,
        prelude::GtkWindowExt,
    };

    let app = Application::builder().build();
    app.connect_activate(move |app| {
        let window = ApplicationWindow::builder().application(app).build();
        let window_ref = window.downgrade();

        let alert = AlertDialog::builder()
            .modal(true)
            .detail(prompt.to_rust_string_lossy())
            .message("Project1")
            .build();
        alert.choose(Some(&window), None::<&Cancellable>, move |_r| {
            if let Some(window) = window_ref.upgrade() {
                window.destroy();
            }
        });
    });
    app.run();
}

#[unsafe(no_mangle)]
#[cfg(windows)]
pub extern "C" fn msg_box(prompt: types::String) {
    use std::ptr;

    use windows_sys::{
        Win32::UI::WindowsAndMessaging::{MB_OK, MessageBoxW},
        core::w,
    };

    let mut prompt = prompt.as_slice().to_vec();
    prompt.push(0);
    unsafe { MessageBoxW(ptr::null_mut(), prompt.as_ptr(), w!("Project1"), MB_OK) };
}

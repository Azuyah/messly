#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{Manager, RunEvent, WindowEvent};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{TrayIconBuilder, TrayIconEvent};

fn main() {
  let app = tauri::Builder::default()
    .setup(|app| {
      let window = app
        .get_webview_window("main")
        .expect("main window not found");

      // ✅ Disable minimize (prevents Dock right-side minimized pile)
      let _ = window.set_minimizable(false);

      // ✅ Close (X) -> hide instead of quitting
      let window_for_close = window.clone();
      window.on_window_event(move |event| {
        if let WindowEvent::CloseRequested { api, .. } = event {
          api.prevent_close();
          let _ = window_for_close.hide();
        }
      });

      // Tray menu items
      let show = MenuItem::with_id(app, "show", "Show Messly", true, None::<&str>)?;
      let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
      let menu = Menu::with_items(app, &[&show, &quit])?;

      // Build tray icon (store it so it doesn't get dropped)
      let tray = TrayIconBuilder::new()
        .menu(&menu)
        .on_menu_event(|app, event| match event.id().as_ref() {
          "show" => {
            if let Some(w) = app.get_webview_window("main") {
              let _ = w.show();
              let _ = w.set_focus();
            }
          }
          "quit" => app.exit(0),
          _ => {}
        })
        .on_tray_icon_event(|tray, event| {
          if let TrayIconEvent::Click { .. } = event {
            if let Some(w) = tray.app_handle().get_webview_window("main") {
              let _ = w.show();
              let _ = w.set_focus();
            }
          }
        })
        .build(app)?; // NOTE: pass `app` (manager), not an AppHandle

      // Keep tray alive
      app.manage(tray);

      Ok(())
    })
    .build(tauri::generate_context!())
    .expect("error while building tauri application");

app.run(|app_handle, event| {
  if let RunEvent::Reopen { has_visible_windows, .. } = event {
    if !has_visible_windows {
      if let Some(w) = app_handle.get_webview_window("main") {
        let _ = w.show();
        let _ = w.set_focus();
      }
    }
  }
});
}
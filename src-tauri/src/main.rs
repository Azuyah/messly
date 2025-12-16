#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{Manager, RunEvent, WindowEvent};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{TrayIconBuilder, TrayIconEvent};
use tauri::webview::WebviewWindowBuilder;

fn main() {
  let app = tauri::Builder::default()
    .setup(|app| {
      // -------------------------------
      // Create main webview manually
      // -------------------------------
      let window_cfg = &app.config().app.windows[0];

      let init_js = r#"
        (() => {
          if (!('mediaSession' in navigator)) return;
          const ms = navigator.mediaSession;

          function anyMediaPlaying() {
            const els = Array.from(document.querySelectorAll('audio,video'));
            return els.some(el => {
              try {
                return !el.paused && !el.ended && el.readyState > 1;
              } catch {
                return false;
              }
            });
          }

          function clearNowPlaying() {
            try { ms.metadata = null; } catch {}
            try { ms.playbackState = 'none'; } catch {}
            try { ms.setPositionState?.(null); } catch {}
          }

          function cleanup(delay = 700) {
            setTimeout(() => {
              if (!anyMediaPlaying()) clearNowPlaying();
            }, delay);
          }

          document.addEventListener('ended', () => cleanup(250), true);
          document.addEventListener('pause', () => cleanup(250), true);

          setInterval(() => {
            if (!anyMediaPlaying()) clearNowPlaying();
          }, 1200);
        })();
      "#;

      let window = WebviewWindowBuilder::from_config(app, window_cfg)?
        .initialization_script(init_js)
        .build()?;

      // -------------------------------
      // Window behavior
      // -------------------------------
      let _ = window.set_minimizable(false);

      let window_for_close = window.clone();
      window.on_window_event(move |event| {
        if let WindowEvent::CloseRequested { api, .. } = event {
          api.prevent_close();
          let _ = window_for_close.hide();
        }
      });

      // -------------------------------
      // Tray menu
      // -------------------------------
      let show = MenuItem::with_id(app, "show", "Show Messly", true, None::<&str>)?;
      let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
      let menu = Menu::with_items(app, &[&show, &quit])?;

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
        .build(app)?;

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
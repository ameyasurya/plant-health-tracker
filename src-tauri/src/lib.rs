pub mod catalog;
pub mod commands;
pub mod models;
pub mod reminder;
pub mod schedule;
pub mod store;
pub mod time;
pub mod weather;

use std::sync::Mutex;

use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{Emitter, Manager, WindowEvent};
use tauri_plugin_autostart::MacosLauncher;

use commands::AppState;
use store::Store;

pub fn run() {
    let mut builder = tauri::Builder::default();

    // Single-instance guard must be registered first: if a second copy is
    // launched (e.g. the user double-clicks the shortcut again, or Windows
    // fires autostart twice), this focuses the existing window instead of
    // starting a second reminder engine that would double-fire digests.
    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }));
    }

    builder
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--hidden".into()]),
        ))
        .invoke_handler(tauri::generate_handler![
            commands::list_due_today,
            commands::list_soon,
            commands::list_all_plants,
            commands::mark_done,
            commands::snooze,
            commands::skip_soil_wet,
            commands::update_plant,
            commands::get_plant,
            commands::add_plant,
            commands::delete_plant,
            commands::search_catalog,
            commands::list_spaces,
            commands::add_space,
            commands::rename_space,
            commands::delete_space,
            commands::search_places,
            commands::detect_location,
            commands::refresh_weather,
            commands::get_weather,
            commands::get_settings,
            commands::update_settings,
            commands::is_autostart_enabled,
            commands::set_pinned_on_top,
            commands::set_active_space,
        ])
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()?;
            let store = Store::new(app_data_dir)?;
            store.ensure_initialised()?;
            store.backfill_catalog_knowledge()?;
            app.manage(AppState {
                store: Mutex::new(store),
            });

            setup_tray(app)?;

            // Reconcile autostart with the stored preference. Earlier builds
            // saved launch_at_startup but never told the OS, so an existing
            // install can have the setting on while Windows knows nothing
            // about it.
            //
            // When it should be on we re-register unconditionally rather
            // than skipping if `is_enabled()` is already true: that only
            // reports whether *an* entry exists, not whether it points at
            // this executable. A registration made from a different path
            // (a dev build, or an older install location) would otherwise
            // survive forever and silently stop working once that path went
            // away. Re-registering is idempotent and rewrites the path.
            {
                use tauri_plugin_autostart::ManagerExt;
                let wanted = {
                    let state = app.state::<AppState>();
                    let store = state.store.lock().expect("store lock");
                    store.load_settings().map(|s| s.launch_at_startup).unwrap_or(false)
                };
                let manager = app.autolaunch();
                let _ = if wanted {
                    manager.enable()
                } else if manager.is_enabled().unwrap_or(false) {
                    manager.disable()
                } else {
                    Ok(())
                };
            }

            let launched_hidden = std::env::args().any(|arg| arg == "--hidden");
            if let Some(window) = app.get_webview_window("main") {
                // Apply the saved pin state here rather than from the
                // frontend: the webview loading is not a precondition for
                // the window being where the user left it, and doing it in
                // JS meant a visible period where an unpinned-by-accident
                // widget could already be buried behind another window.
                let pinned = {
                    let state = app.state::<AppState>();
                    let store = state.store.lock().expect("store lock");
                    store.load_settings().map(|s| s.pinned_on_top).unwrap_or(true)
                };
                let _ = window.set_always_on_top(pinned);

                if launched_hidden {
                    window.hide()?;
                }
                // Minimize-to-tray: closing the window hides it instead of
                // quitting the process, so the reminder engine keeps running.
                let window_handle = window.clone();
                window.on_window_event(move |event| {
                    if let WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = window_handle.hide();
                    }
                });
            }

            reminder::spawn(app.handle().clone());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running the plant health tracker");
}

fn setup_tray(app: &tauri::App) -> tauri::Result<()> {
    let show_i = MenuItem::with_id(app, "show", "Show widget", true, None::<&str>)?;
    let hide_i = MenuItem::with_id(app, "hide", "Hide widget", true, None::<&str>)?;
    let mark_all_i = MenuItem::with_id(app, "mark_all_viewed", "Mark all viewed", true, None::<&str>)?;
    let settings_i = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show_i, &hide_i, &mark_all_i, &settings_i, &quit_i])?;

    TrayIconBuilder::new()
        .icon(app.default_window_icon().cloned().expect("tray icon asset missing"))
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| {
            let window = app.get_webview_window("main");
            match event.id().as_ref() {
                "show" => {
                    // This is the recovery path when an unpinned widget has
                    // been buried by another window, so it must raise as
                    // well as unhide -- show() alone leaves it where it is
                    // in the z-order.
                    if let Some(w) = &window {
                        let _ = w.unminimize();
                        let _ = w.show();
                        let _ = w.set_focus();
                    }
                }
                "hide" => {
                    if let Some(w) = &window {
                        let _ = w.hide();
                    }
                }
                "mark_all_viewed" => {
                    // Frontend listens for this event and clears any unread
                    // badge state; the underlying due/soon data is unchanged.
                    let _ = app.emit("mark-all-viewed", ());
                }
                "settings" => {
                    if let Some(w) = &window {
                        let _ = w.show();
                        let _ = w.set_focus();
                        let _ = app.emit("open-settings", ());
                    }
                }
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .build(app)?;
    Ok(())
}

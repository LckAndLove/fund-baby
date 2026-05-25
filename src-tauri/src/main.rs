#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;
use tauri::{
    menu::{CheckMenuItem, MenuBuilder, MenuItem, Submenu},
    PhysicalPosition, PhysicalSize,
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, WindowEvent,
};
use tauri_plugin_autostart::ManagerExt as AutostartManagerExt;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Shortcut, ShortcutState};

const FULL_WINDOW_SIZE: PhysicalSize<u32> = PhysicalSize { width: 720, height: 360 };
const FULL_WINDOW_MIN_SIZE: PhysicalSize<u32> = PhysicalSize { width: 620, height: 220 };
const FULL_WINDOW_MAX_SIZE: PhysicalSize<u32> = PhysicalSize { width: 900, height: 700 };
const COMPACT_WINDOW_SIZE: PhysicalSize<u32> = PhysicalSize { width: 225, height: 38 };
const COMPACT_WINDOW_MARGIN: i32 = 4;

#[derive(Default)]
struct BossKeyState {
    last_position: Mutex<Option<PhysicalPosition<i32>>>,
    is_compact: Mutex<bool>,
}

fn show_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if let Ok(mut is_compact) = app.state::<BossKeyState>().is_compact.lock() {
            *is_compact = false;
        }
        let position = app
            .state::<BossKeyState>()
            .last_position
            .lock()
            .ok()
            .and_then(|mut position| position.take())
            .unwrap_or_else(|| PhysicalPosition::new(120, 120));
        let _ = window.show();
        let _ = window.set_resizable(true);
        let _ = window.set_min_size(Some(FULL_WINDOW_MIN_SIZE));
        let _ = window.set_max_size(Some(FULL_WINDOW_MAX_SIZE));
        let _ = window.set_size(FULL_WINDOW_SIZE);
        let _ = window.set_position(position);
        let _ = window.set_focus();
        let _ = window.emit("desktop-widget-compact", false);
        let _ = window.emit("desktop-widget-refresh", ());
    }
}

fn show_main_window_instant(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let position = app
            .state::<BossKeyState>()
            .last_position
            .lock()
            .ok()
            .and_then(|mut position| position.take())
            .unwrap_or_else(|| PhysicalPosition::new(120, 120));
        let _ = window.show();
        let _ = window.set_resizable(true);
        let _ = window.set_min_size(Some(FULL_WINDOW_MIN_SIZE));
        let _ = window.set_max_size(Some(FULL_WINDOW_MAX_SIZE));
        let _ = window.set_size(FULL_WINDOW_SIZE);
        let _ = window.set_position(position);
        let _ = window.set_focus();
        if let Ok(mut is_compact) = app.state::<BossKeyState>().is_compact.lock() {
            *is_compact = false;
        }
        let _ = window.emit("desktop-widget-compact", false);
        let _ = window.emit("desktop-widget-refresh", ());
    }
}

fn hide_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if let Ok(mut is_compact) = app.state::<BossKeyState>().is_compact.lock() {
            *is_compact = false;
        }
        let _ = window.emit("desktop-widget-compact", false);
        let _ = window.hide();
    }
}

fn show_compact_profit_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let was_compact = app
            .state::<BossKeyState>()
            .is_compact
            .lock()
            .map(|is_compact| *is_compact)
            .unwrap_or(false);

        if !was_compact {
            if let Ok(position) = window.outer_position() {
                if position.x > -30000 && position.y > -30000 {
                    if let Ok(mut last_position) = app.state::<BossKeyState>().last_position.lock() {
                        *last_position = Some(position);
                    }
                }
            }
        }

        let position = window
            .current_monitor()
            .ok()
            .flatten()
            .map(|monitor| {
                let monitor_position = monitor.position();
                let monitor_size = monitor.size();
                PhysicalPosition::new(
                    monitor_position.x + COMPACT_WINDOW_MARGIN,
                    monitor_position.y + monitor_size.height as i32 - COMPACT_WINDOW_SIZE.height as i32 - COMPACT_WINDOW_MARGIN,
                )
            })
            .unwrap_or_else(|| PhysicalPosition::new(COMPACT_WINDOW_MARGIN, 720));

        let _ = window.show();
        let _ = window.set_resizable(false);
        let _ = window.set_min_size(Some(COMPACT_WINDOW_SIZE));
        let _ = window.set_max_size(Some(COMPACT_WINDOW_SIZE));
        let _ = window.set_size(COMPACT_WINDOW_SIZE);
        let _ = window.set_position(position);
        if let Ok(mut is_compact) = app.state::<BossKeyState>().is_compact.lock() {
            *is_compact = true;
        }
        let _ = window.emit("desktop-widget-compact", true);
    }
}

fn toggle_main_window_instant(app: &tauri::AppHandle) {
    let is_hidden = app
        .state::<BossKeyState>()
        .last_position
        .lock()
        .map(|position| position.is_some())
        .unwrap_or(false);

    let is_visible = app
        .get_webview_window("main")
        .and_then(|window| window.is_visible().ok())
        .unwrap_or(false);
    let is_compact = app
        .state::<BossKeyState>()
        .is_compact
        .lock()
        .map(|is_compact| *is_compact)
        .unwrap_or(false);

    if is_hidden || !is_visible || is_compact {
        show_main_window_instant(app);
    } else {
        show_compact_profit_window(app);
    }
}

fn main() {
    tauri::Builder::default()
        .manage(BossKeyState::default())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--hidden"]),
        ))
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    if event.state != ShortcutState::Pressed {
                        return;
                    }
                    if shortcut == &Shortcut::new(None, Code::F2) {
                        toggle_main_window_instant(app);
                    }
                })
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_skip_taskbar(true);
            }

            if let Err(error) = app.global_shortcut().register(Shortcut::new(None, Code::F2)) {
                eprintln!("failed to register F2 global shortcut: {error}");
            }

            let autostart_enabled = app.autolaunch().is_enabled().unwrap_or(false);
            let autostart_item =
                CheckMenuItem::with_id(app, "autostart", "开机自启动", true, autostart_enabled, None::<&str>)?;
            let opacity_menu = Submenu::with_id_and_items(
                app,
                "opacity",
                "透明度",
                true,
                &[
                    &MenuItem::with_id(app, "opacity:0", "0%", true, None::<&str>)?,
                    &MenuItem::with_id(app, "opacity:20", "20%", true, None::<&str>)?,
                    &MenuItem::with_id(app, "opacity:40", "40%", true, None::<&str>)?,
                    &MenuItem::with_id(app, "opacity:60", "60%", true, None::<&str>)?,
                    &MenuItem::with_id(app, "opacity:80", "80%", true, None::<&str>)?,
                    &MenuItem::with_id(app, "opacity:100", "100%", true, None::<&str>)?,
                ],
            )?;

            let menu = MenuBuilder::new(app)
                .text("show", "显示窗口")
                .text("hide", "隐藏到托盘")
                .separator()
                .item(&autostart_item)
                .item(&opacity_menu)
                .separator()
                .text("quit", "退出")
                .build()?;
            let icon = app.default_window_icon().cloned();
            let mut tray = TrayIconBuilder::with_id("main")
                .tooltip("养基小宝")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(move |app, event| match event.id().as_ref() {
                    "show" => show_main_window(app),
                    "hide" => hide_main_window(app),
                    "autostart" => {
                        let enabled = !app.autolaunch().is_enabled().unwrap_or(false);
                        let result = if enabled {
                            app.autolaunch().enable()
                        } else {
                            app.autolaunch().disable()
                        };
                        if result.is_ok() {
                            let _ = autostart_item.set_checked(enabled);
                        }
                    }
                    "quit" => app.exit(0),
                    id if id.starts_with("opacity:") => {
                        if let Ok(percent) = id.trim_start_matches("opacity:").parse::<u8>() {
                            let opacity = f64::from(percent) / 100.0;
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.emit("desktop-widget-opacity", opacity);
                            }
                        }
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::DoubleClick {
                        button: MouseButton::Left,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let visible = window.is_visible().unwrap_or(false);
                            if visible {
                                hide_main_window(app);
                            } else {
                                show_main_window(app);
                            }
                        }
                    }
                });
            if let Some(icon) = icon {
                tray = tray.icon(icon);
            }
            tray.build(app)?;
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;
use tauri::{
    menu::MenuBuilder,
    PhysicalPosition,
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    Manager, WindowEvent,
};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Shortcut, ShortcutState};

const BOSS_KEY_HIDE_POSITION: PhysicalPosition<i32> = PhysicalPosition { x: -32000, y: -32000 };

#[derive(Default)]
struct BossKeyState {
    last_position: Mutex<Option<PhysicalPosition<i32>>>,
}

fn show_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
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
        let _ = window.set_position(position);
        let _ = window.set_focus();
    }
}

fn hide_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}

fn hide_main_window_instant(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if let Ok(position) = window.outer_position() {
            if position.x > -30000 && position.y > -30000 {
                if let Ok(mut last_position) = app.state::<BossKeyState>().last_position.lock() {
                    *last_position = Some(position);
                }
            }
        }
        let _ = window.set_position(BOSS_KEY_HIDE_POSITION);
    }
}

fn main() {
    tauri::Builder::default()
        .manage(BossKeyState::default())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    if event.state != ShortcutState::Pressed {
                        return;
                    }
                    if shortcut == &Shortcut::new(None, Code::F2) {
                        show_main_window_instant(app);
                    }
                    if shortcut == &Shortcut::new(None, Code::F4) {
                        hide_main_window_instant(app);
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
            if let Err(error) = app.global_shortcut().register(Shortcut::new(None, Code::F4)) {
                eprintln!("failed to register F4 global shortcut: {error}");
            }

            let menu = MenuBuilder::new(app)
                .text("show", "显示窗口")
                .text("hide", "隐藏到托盘")
                .separator()
                .text("quit", "退出")
                .build()?;
            let icon = app.default_window_icon().cloned();
            let mut tray = TrayIconBuilder::with_id("main")
                .tooltip("养基小宝")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "show" => show_main_window(app),
                    "hide" => hide_main_window(app),
                    "quit" => app.exit(0),
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

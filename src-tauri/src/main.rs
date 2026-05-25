#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::{fs, sync::Mutex};
use tauri::{
    menu::{CheckMenuItem, MenuBuilder, MenuItem, Submenu},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, PhysicalPosition, PhysicalSize, WindowEvent,
};
use tauri_plugin_autostart::ManagerExt as AutostartManagerExt;
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Shortcut, ShortcutState};

const FULL_WINDOW_SIZE: PhysicalSize<u32> = PhysicalSize {
    width: 720,
    height: 360,
};
const FULL_WINDOW_MIN_SIZE: PhysicalSize<u32> = PhysicalSize {
    width: 620,
    height: 220,
};
const FULL_WINDOW_MAX_SIZE: PhysicalSize<u32> = PhysicalSize {
    width: 900,
    height: 700,
};
const COMPACT_WINDOW_SIZE: PhysicalSize<u32> = PhysicalSize {
    width: 225,
    height: 38,
};
const COMPACT_WINDOW_MARGIN: i32 = 4;
const DESKTOP_STATE_FILE: &str = "desktop-state.json";
const PROGRAMMATIC_MOVE_TOLERANCE: i32 = 4;

#[derive(Clone, Copy, Deserialize, Serialize)]
struct SavedPosition {
    x: i32,
    y: i32,
}

#[derive(Default, Deserialize, Serialize)]
struct DesktopState {
    compact_position: Option<SavedPosition>,
}

#[derive(Default)]
struct BossKeyState {
    last_full_position: Mutex<Option<PhysicalPosition<i32>>>,
    last_compact_position: Mutex<Option<PhysicalPosition<i32>>>,
    is_compact: Mutex<bool>,
    suppressed_move_position: Mutex<Option<PhysicalPosition<i32>>>,
}

impl From<PhysicalPosition<i32>> for SavedPosition {
    fn from(position: PhysicalPosition<i32>) -> Self {
        Self {
            x: position.x,
            y: position.y,
        }
    }
}

impl From<SavedPosition> for PhysicalPosition<i32> {
    fn from(position: SavedPosition) -> Self {
        PhysicalPosition::new(position.x, position.y)
    }
}

fn desktop_state_path(app: &tauri::AppHandle) -> Option<std::path::PathBuf> {
    app.path()
        .app_config_dir()
        .ok()
        .map(|dir| dir.join(DESKTOP_STATE_FILE))
}

fn read_desktop_state(app: &tauri::AppHandle) -> DesktopState {
    desktop_state_path(app)
        .and_then(|path| fs::read_to_string(path).ok())
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

fn write_desktop_state(app: &tauri::AppHandle, state: &DesktopState) {
    if let Some(path) = desktop_state_path(app) {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(text) = serde_json::to_string_pretty(state) {
            let _ = fs::write(path, text);
        }
    }
}

fn save_compact_position(app: &tauri::AppHandle, position: PhysicalPosition<i32>) {
    let mut state = read_desktop_state(app);
    state.compact_position = Some(position.into());
    write_desktop_state(app, &state);
}

fn suppress_programmatic_move(app: &tauri::AppHandle, position: PhysicalPosition<i32>) {
    if let Ok(mut suppressed_position) = app.state::<BossKeyState>().suppressed_move_position.lock()
    {
        *suppressed_position = Some(position);
    }
}

fn is_near_position(a: PhysicalPosition<i32>, b: PhysicalPosition<i32>) -> bool {
    (a.x - b.x).abs() <= PROGRAMMATIC_MOVE_TOLERANCE
        && (a.y - b.y).abs() <= PROGRAMMATIC_MOVE_TOLERANCE
}

fn show_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if let Ok(mut is_compact) = app.state::<BossKeyState>().is_compact.lock() {
            *is_compact = false;
        }
        let position = app
            .state::<BossKeyState>()
            .last_full_position
            .lock()
            .ok()
            .and_then(|mut position| position.take())
            .unwrap_or_else(|| PhysicalPosition::new(120, 120));
        let _ = window.show();
        let _ = window.set_resizable(true);
        let _ = window.set_min_size(Some(FULL_WINDOW_MIN_SIZE));
        let _ = window.set_max_size(Some(FULL_WINDOW_MAX_SIZE));
        let _ = window.set_size(FULL_WINDOW_SIZE);
        suppress_programmatic_move(app, position);
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
            .last_full_position
            .lock()
            .ok()
            .and_then(|mut position| position.take())
            .unwrap_or_else(|| PhysicalPosition::new(120, 120));
        let _ = window.show();
        let _ = window.set_resizable(true);
        let _ = window.set_min_size(Some(FULL_WINDOW_MIN_SIZE));
        let _ = window.set_max_size(Some(FULL_WINDOW_MAX_SIZE));
        let _ = window.set_size(FULL_WINDOW_SIZE);
        if let Ok(mut is_compact) = app.state::<BossKeyState>().is_compact.lock() {
            *is_compact = false;
        }
        suppress_programmatic_move(app, position);
        let _ = window.set_position(position);
        let _ = window.set_focus();
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
                    if let Ok(mut last_position) =
                        app.state::<BossKeyState>().last_full_position.lock()
                    {
                        *last_position = Some(position);
                    }
                }
            }
        }

        let position = app
            .state::<BossKeyState>()
            .last_compact_position
            .lock()
            .ok()
            .and_then(|position| *position)
            .unwrap_or_else(|| {
                window
                    .current_monitor()
                    .ok()
                    .flatten()
                    .map(|monitor| {
                        let work_area = monitor.work_area();
                        let monitor_position = work_area.position;
                        let monitor_size = work_area.size;
                        PhysicalPosition::new(
                            monitor_position.x + COMPACT_WINDOW_MARGIN,
                            monitor_position.y + monitor_size.height as i32
                                - COMPACT_WINDOW_SIZE.height as i32
                                - COMPACT_WINDOW_MARGIN,
                        )
                    })
                    .unwrap_or_else(|| PhysicalPosition::new(COMPACT_WINDOW_MARGIN, 720))
            });

        let _ = window.show();
        let _ = window.set_resizable(false);
        let _ = window.set_min_size(Some(COMPACT_WINDOW_SIZE));
        let _ = window.set_max_size(Some(COMPACT_WINDOW_SIZE));
        let _ = window.set_size(COMPACT_WINDOW_SIZE);
        suppress_programmatic_move(app, position);
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
        .last_full_position
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

#[tauri::command]
fn save_export_config_file(app: tauri::AppHandle, content: String) -> Result<bool, String> {
    let file_path = app
        .dialog()
        .file()
        .add_filter("JSON", &["json"])
        .set_file_name(format!(
            "realtime-fund-config-{}.json",
            timestamp_millis()
        ))
        .blocking_save_file()
        .ok_or_else(|| "cancelled".to_string())?
        .into_path()
        .map_err(|error| error.to_string())?;

    fs::write(file_path, content).map_err(|error| error.to_string())?;
    Ok(true)
}

#[tauri::command]
fn open_import_config_file(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let file_path = app
        .dialog()
        .file()
        .add_filter("JSON", &["json"])
        .blocking_pick_file()
        .ok_or_else(|| "cancelled".to_string())?
        .into_path()
        .map_err(|error| error.to_string())?;

    fs::read_to_string(file_path)
        .map(Some)
        .map_err(|error| error.to_string())
}

fn timestamp_millis() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default()
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
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            save_export_config_file,
            open_import_config_file
        ])
        .setup(|app| {
            let saved_state = read_desktop_state(app.handle());
            if let Some(position) = saved_state.compact_position {
                if let Ok(mut last_compact_position) =
                    app.state::<BossKeyState>().last_compact_position.lock()
                {
                    *last_compact_position = Some(position.into());
                }
            }

            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_skip_taskbar(true);
            }

            if let Err(error) = app
                .global_shortcut()
                .register(Shortcut::new(None, Code::F2))
            {
                eprintln!("failed to register F2 global shortcut: {error}");
            }

            let autostart_enabled = app.autolaunch().is_enabled().unwrap_or(false);
            let autostart_item = CheckMenuItem::with_id(
                app,
                "autostart",
                "开机自启动",
                true,
                autostart_enabled,
                None::<&str>,
            )?;
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
                .text("import-data", "导入配置")
                .text("export-data", "导出配置")
                .separator()
                .text("quit", "退出")
                .build()?;
            let icon = app.default_window_icon().cloned();
            let mut tray = TrayIconBuilder::with_id("main")
                .tooltip("养基场")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(move |app, event| match event.id().as_ref() {
                    "show" => show_main_window(app),
                    "hide" => hide_main_window(app),
                    "import-data" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                            let _ = window.emit("desktop-data-import", ());
                        }
                    }
                    "export-data" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                            let _ = window.emit("desktop-data-export", ());
                        }
                    }
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
        .on_window_event(|window, event| match event {
            WindowEvent::Moved(position) => {
                let app = window.app_handle();
                let suppressed = app
                    .state::<BossKeyState>()
                    .suppressed_move_position
                    .lock()
                    .ok()
                    .and_then(|mut suppressed_position| {
                        let should_suppress = suppressed_position
                            .map(|target| is_near_position(target, *position))
                            .unwrap_or(false);
                        if should_suppress {
                            *suppressed_position = None;
                        }
                        Some(should_suppress)
                    })
                    .unwrap_or(false);
                if suppressed {
                    return;
                }
                let is_compact = app
                    .state::<BossKeyState>()
                    .is_compact
                    .lock()
                    .map(|is_compact| *is_compact)
                    .unwrap_or(false);
                if is_compact {
                    let is_compact_size = window
                        .outer_size()
                        .map(|size| {
                            size.width <= COMPACT_WINDOW_SIZE.width + 8
                                && size.height <= COMPACT_WINDOW_SIZE.height + 8
                        })
                        .unwrap_or(false);
                    if !is_compact_size {
                        return;
                    }
                    if let Ok(mut last_compact_position) =
                        app.state::<BossKeyState>().last_compact_position.lock()
                    {
                        *last_compact_position = Some(*position);
                    }
                    save_compact_position(app, *position);
                }
            }
            WindowEvent::CloseRequested { api, .. } => {
                api.prevent_close();
                let _ = window.hide();
            }
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

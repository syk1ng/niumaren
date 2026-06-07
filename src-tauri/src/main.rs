#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod models;
mod db;
mod commands;
mod email;
mod holiday;
mod scheduler;

use db::Database;
use scheduler::Scheduler;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_dir: PathBuf = app.path().app_data_dir()?;
            let database = Database::new(app_dir)
                .expect("Failed to initialize database");
            let db = Arc::new(database);
            app.manage(db.clone());

            // Build tray menu
            let show = MenuItem::with_id(app, "show", "打开主面板", true, None::<&str>)?;
            let _schedule_menu = MenuItem::with_id(app, "schedule", "查看排班表", true, None::<&str>)?;
            let _manual = MenuItem::with_id(app, "manual", "手动发送通知", true, None::<&str>)?;
            let separator = MenuItem::with_id(app, "sep", "──────────", false, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &_schedule_menu, &_manual, &separator, &quit])?;

            // Build a simple 16x16 orange circle icon in RGBA
            let size = 16u32;
            let mut rgba = Vec::with_capacity((size * size * 4) as usize);
            let cx = (size / 2) as f64;
            let cy = (size / 2) as f64;
            let r = (size / 2 - 1) as f64;
            for y in 0..size {
                for x in 0..size {
                    let dx = x as f64 - cx;
                    let dy = y as f64 - cy;
                    let dist = (dx * dx + dy * dy).sqrt();
                    if dist <= r {
                        rgba.extend_from_slice(&[245, 158, 11, 255]); // #f59e0b
                    } else {
                        rgba.extend_from_slice(&[0, 0, 0, 0]);
                    }
                }
            }
            let tray_icon = Image::new_owned(rgba, size, size);
            let _tray = TrayIconBuilder::new()
                .icon(tray_icon)
                .menu(&menu)
                .tooltip("牛马人 · 值班助手")
                .on_menu_event(|app, event| {
                    match event.id.as_ref() {
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                window.show().ok();
                                window.set_focus().ok();
                            }
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up, ..
                    } = event {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            window.show().ok();
                            window.set_focus().ok();
                        }
                    }
                })
                .build(app)?;
            // Keep tray icon alive after setup returns
            Box::leak(Box::new(_tray));

            // Hide window on close instead of quitting
            if let Some(window) = app.get_webview_window("main") {
                let w = window.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        w.hide().ok();
                    }
                });
            }

            // Start scheduler in background
            let scheduler_db = db.clone();
            tauri::async_runtime::spawn(async move {
                let scheduler = Scheduler::new(scheduler_db);
                scheduler.start().await;
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_personnel,
            commands::add_personnel,
            commands::update_personnel,
            commands::delete_personnel,
            commands::reorder_personnel,
            commands::get_settings,
            commands::save_settings,
            commands::save_setting,
            commands::test_send_email,
            commands::check_holiday,
            commands::get_schedules,
            commands::get_email_logs,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

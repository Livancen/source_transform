mod commands;
mod hw;
mod logger;
mod naming;
mod process;
mod types;
mod upload_server;
mod watermark;

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};

fn show_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            show_main_window(app);
        }))
        .setup(|app| {
            if let Ok(app_dir) = app.path().app_data_dir() {
                match logger::init(&app_dir) {
                    Ok(path) => {
                        logger::info(format!("应用启动，日志文件: {}", path.display()));
                    }
                    Err(e) => {
                        eprintln!("初始化日志失败: {}", e);
                    }
                }
            }

            #[cfg(desktop)]
            {
                app.handle()
                    .plugin(tauri_plugin_updater::Builder::new().build())?;

                let show_i =
                    MenuItem::with_id(app, "show", "打开主界面", true, None::<&str>)?;
                let quit_i = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
                let menu = Menu::with_items(app, &[&show_i, &quit_i])?;

                let icon = app
                    .default_window_icon()
                    .ok_or("缺少默认窗口图标")?
                    .clone();

                let _tray = TrayIconBuilder::with_id("main-tray")
                    .icon(icon)
                    .tooltip("素材转换工具")
                    .menu(&menu)
                    .show_menu_on_left_click(false)
                    .on_menu_event(|app, event| match event.id.as_ref() {
                        "show" => show_main_window(app),
                        "quit" => {
                            logger::info("用户从托盘退出应用");
                            app.exit(0);
                        }
                        _ => {}
                    })
                    .on_tray_icon_event(|tray, event| {
                        if let TrayIconEvent::Click {
                            button: MouseButton::Left,
                            button_state: MouseButtonState::Up,
                            ..
                        } = event
                        {
                            show_main_window(tray.app_handle());
                        }
                    })
                    .build(app)?;
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_input_dir,
            commands::get_output_dir,
            commands::set_custom_dirs,
            commands::get_custom_dirs,
            commands::set_naming_options,
            commands::get_naming_options,
            commands::get_hw_accel_options,
            commands::set_hw_accel_mode,
            commands::detect_hw_encoders,
            commands::scan_input_files,
            commands::get_video_dimensions,
            commands::get_image_dimensions,
            commands::extract_video_frame,
            commands::load_image_preview,
            commands::get_file_thumbnail,
            commands::process_files,
            commands::crop_by_ratios,
            commands::crop_videos_by_ratios,
            commands::custom_crop,
            commands::merge_videos,
            commands::join_media,
            commands::open_folder,
            commands::start_upload_server,
            commands::get_log_path,
            commands::read_logs,
            commands::clear_logs,
            commands::export_logs,
            commands::open_logs_dir,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

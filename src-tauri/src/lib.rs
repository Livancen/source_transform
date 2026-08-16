mod commands;
mod naming;
mod process;
mod types;
mod upload_server;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            #[cfg(desktop)]
            {
                app.handle()
                    .plugin(tauri_plugin_updater::Builder::new().build())?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_input_dir,
            commands::get_output_dir,
            commands::set_custom_dirs,
            commands::get_custom_dirs,
            commands::set_naming_options,
            commands::get_naming_options,
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
            commands::open_folder,
            commands::start_upload_server,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

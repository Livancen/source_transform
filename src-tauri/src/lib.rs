mod types;
mod process;
mod commands;
mod upload_server;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            commands::get_input_dir,
            commands::get_output_dir,
            commands::set_custom_dirs,
            commands::get_custom_dirs,
            commands::scan_input_files,
            commands::get_video_dimensions,
            commands::extract_video_frame,
            commands::process_files,
            commands::crop_videos_by_ratios,
            commands::merge_videos,
            commands::open_folder,
            commands::start_upload_server,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

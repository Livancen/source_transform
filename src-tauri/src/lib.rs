use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_shell::ShellExt;
use walkdir::WalkDir;

// 支持的文件扩展名
const IMAGE_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "gif", "bmp", "webp", "tiff"];
const VIDEO_EXTENSIONS: &[&str] = &["mp4", "avi", "mov", "mkv", "wmv", "flv", "webm", "m4v"];

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProcessOptions {
    pub compress: bool,
    pub compress_quality: u32, // 1-100
    pub compress_resize: bool, // 压缩时是否同时降低分辨率
    pub compress_width: u32,   // 压缩时的目标宽度
    pub compress_height: u32,  // 压缩时的目标高度
    pub reduce_resolution: bool,
    pub target_width: u32,
    pub target_height: u32,
    pub reduce_bitrate: bool,
    pub target_bitrate: String, // e.g., "1M", "500k"
    pub reduce_profile: bool,
    pub target_profile: String, // e.g., "baseline", "main", "high"
    pub rotate: bool,
    pub rotation_degrees: i32, // 90, 180, 270, -90
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProcessProgress {
    pub current: usize,
    pub total: usize,
    pub current_file: String,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileInfo {
    pub path: String,
    pub name: String,
    pub file_type: String, // "image" or "video"
}

fn get_file_type(path: &PathBuf) -> Option<String> {
    let ext = path.extension()?.to_str()?.to_lowercase();
    if IMAGE_EXTENSIONS.contains(&ext.as_str()) {
        Some("image".to_string())
    } else if VIDEO_EXTENSIONS.contains(&ext.as_str()) {
        Some("video".to_string())
    } else {
        None
    }
}

fn generate_timestamp() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", now.as_millis())
}

fn add_timestamp_to_filename(path: &PathBuf) -> PathBuf {
    let stem = path.file_stem().unwrap_or_default().to_string_lossy();
    let ext = path.extension().unwrap_or_default().to_string_lossy();
    let timestamp = generate_timestamp();
    let new_name = if ext.is_empty() {
        format!("{}_{}", stem, timestamp)
    } else {
        format!("{}_{}.{}", stem, timestamp, ext)
    };
    path.with_file_name(new_name)
}

#[tauri::command]
fn get_input_dir(app: AppHandle) -> Result<String, String> {
    let app_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    let input_dir = app_dir.join("input");
    std::fs::create_dir_all(&input_dir).map_err(|e| e.to_string())?;
    Ok(input_dir.to_string_lossy().to_string())
}

#[tauri::command]
fn get_output_dir(app: AppHandle) -> Result<String, String> {
    let app_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    let output_dir = app_dir.join("output");
    std::fs::create_dir_all(&output_dir).map_err(|e| e.to_string())?;
    Ok(output_dir.to_string_lossy().to_string())
}

#[tauri::command]
fn set_custom_dirs(app: AppHandle, input_path: String, output_path: String) -> Result<(), String> {
    let app_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    let config_path = app_dir.join("config.json");

    let config = serde_json::json!({
        "input_dir": input_path,
        "output_dir": output_path
    });

    std::fs::write(&config_path, serde_json::to_string_pretty(&config).unwrap())
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
fn get_custom_dirs(app: AppHandle) -> Result<(String, String), String> {
    let app_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    let config_path = app_dir.join("config.json");

    if config_path.exists() {
        let content = std::fs::read_to_string(&config_path).map_err(|e| e.to_string())?;
        let config: serde_json::Value = serde_json::from_str(&content).map_err(|e| e.to_string())?;
        let input = config["input_dir"].as_str().unwrap_or("").to_string();
        let output = config["output_dir"].as_str().unwrap_or("").to_string();
        Ok((input, output))
    } else {
        // 返回默认目录
        let input_dir = app_dir.join("input");
        let output_dir = app_dir.join("output");
        std::fs::create_dir_all(&input_dir).map_err(|e| e.to_string())?;
        std::fs::create_dir_all(&output_dir).map_err(|e| e.to_string())?;
        Ok((input_dir.to_string_lossy().to_string(), output_dir.to_string_lossy().to_string()))
    }
}

#[tauri::command]
fn scan_input_files(input_dir: String) -> Result<Vec<FileInfo>, String> {
    let mut files = Vec::new();
    let path = PathBuf::from(&input_dir);

    if !path.exists() {
        return Err("输入目录不存在".to_string());
    }

    for entry in WalkDir::new(&path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path().to_path_buf();
        if path.is_file() {
            if let Some(file_type) = get_file_type(&path) {
                files.push(FileInfo {
                    path: path.to_string_lossy().to_string(),
                    name: path.file_name().unwrap_or_default().to_string_lossy().to_string(),
                    file_type,
                });
            }
        }
    }

    Ok(files)
}

fn process_image(
    input_path: &str,
    output_path: &str,
    options: &ProcessOptions,
) -> Result<(), String> {
    let mut img = image::open(input_path).map_err(|e| format!("无法打开图片: {}", e))?;

    // 旋转
    if options.rotate {
        img = match options.rotation_degrees {
            90 | -270 => image::DynamicImage::ImageRgba8(image::imageops::rotate90(&img.to_rgba8())),
            180 | -180 => image::DynamicImage::ImageRgba8(image::imageops::rotate180(&img.to_rgba8())),
            270 | -90 => image::DynamicImage::ImageRgba8(image::imageops::rotate270(&img.to_rgba8())),
            _ => img,
        };
    }

    // 调整分辨率（独立选项）
    if options.reduce_resolution && options.target_width > 0 && options.target_height > 0 {
        img = img.resize(
            options.target_width,
            options.target_height,
            image::imageops::FilterType::Lanczos3,
        );
    }

    // 压缩时降低分辨率
    if options.compress && options.compress_resize && options.compress_width > 0 && options.compress_height > 0 {
        img = img.resize(
            options.compress_width,
            options.compress_height,
            image::imageops::FilterType::Lanczos3,
        );
    }

    // 保存（压缩质量通过格式处理）
    let output_path = PathBuf::from(output_path);
    let ext = output_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("jpg")
        .to_lowercase();

    match ext.as_str() {
        "jpg" | "jpeg" => {
            let quality = if options.compress {
                options.compress_quality.min(100).max(1) as u8
            } else {
                90
            };
            let mut file = std::fs::File::create(&output_path)
                .map_err(|e| format!("无法创建输出文件: {}", e))?;
            let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut file, quality);
            img.write_with_encoder(encoder)
                .map_err(|e| format!("无法编码图片: {}", e))?;
        }
        "png" => {
            img.save(&output_path)
                .map_err(|e| format!("无法保存图片: {}", e))?;
        }
        _ => {
            img.save(&output_path)
                .map_err(|e| format!("无法保存图片: {}", e))?;
        }
    }

    Ok(())
}

async fn process_video(
    app: &AppHandle,
    input_path: &str,
    output_path: &str,
    options: &ProcessOptions,
) -> Result<(), String> {
    let mut args: Vec<&str> = vec!["-i", input_path, "-y"];

    // 视频滤镜
    let mut video_filters: Vec<String> = Vec::new();

    // 调整分辨率（独立选项）
    if options.reduce_resolution && options.target_width > 0 && options.target_height > 0 {
        video_filters.push(format!("scale={}:{}", options.target_width, options.target_height));
    }

    // 压缩时降低分辨率
    if options.compress && options.compress_resize && options.compress_width > 0 && options.compress_height > 0 {
        video_filters.push(format!("scale={}:{}", options.compress_width, options.compress_height));
    }

    // 旋转
    if options.rotate {
        match options.rotation_degrees {
            90 | -270 => video_filters.push("transpose=1".to_string()),
            180 | -180 => video_filters.push("transpose=1,transpose=1".to_string()),
            270 | -90 => video_filters.push("transpose=2".to_string()),
            _ => {}
        }
    }

    let vf_arg: String;
    if !video_filters.is_empty() {
        vf_arg = video_filters.join(",");
        args.push("-vf");
        args.push(&vf_arg);
    }

    // 码率
    let bitrate: String;
    if options.reduce_bitrate && !options.target_bitrate.is_empty() {
        bitrate = options.target_bitrate.clone();
        args.push("-b:v");
        args.push(&bitrate);
    }

    // Profile 等级
    let profile: String;
    if options.reduce_profile && !options.target_profile.is_empty() {
        profile = options.target_profile.clone();
        args.push("-profile:v");
        args.push(&profile);
    }

    // 压缩（使用CRF）
    let crf_str: String;
    if options.compress {
        let crf = ((100 - options.compress_quality) as f32 * 0.51) as u32;
        crf_str = crf.to_string();
        args.push("-crf");
        args.push(&crf_str);
    }

    args.push(output_path);

    // 使用sidecar调用ffmpeg
    let output = app
        .shell()
        .sidecar("ffmpeg")
        .map_err(|e| format!("无法找到FFmpeg: {}", e))?
        .args(&args)
        .output()
        .await
        .map_err(|e| format!("无法执行FFmpeg: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("FFmpeg处理失败: {}", stderr));
    }

    Ok(())
}

#[tauri::command]
async fn process_files(
    app: AppHandle,
    input_dir: String,
    output_dir: String,
    options: ProcessOptions,
) -> Result<String, String> {
    // 确保输出目录存在
    std::fs::create_dir_all(&output_dir).map_err(|e| e.to_string())?;

    // 扫描文件
    let files = scan_input_files(input_dir.clone())?;
    let total = files.len();

    if total == 0 {
        return Err("输入目录中没有找到支持的媒体文件".to_string());
    }

    let mut success_count = 0;
    let mut error_count = 0;
    let mut errors: Vec<String> = Vec::new();

    for (index, file) in files.iter().enumerate() {
        // 发送进度事件
        let progress = ProcessProgress {
            current: index + 1,
            total,
            current_file: file.name.clone(),
            status: "processing".to_string(),
        };
        let _ = app.emit("process-progress", &progress);

        // 构建输出路径
        let input_path = PathBuf::from(&file.path);
        let relative_path = input_path
            .strip_prefix(&input_dir)
            .unwrap_or(&input_path);
        let output_path = PathBuf::from(&output_dir).join(relative_path);

        // 添加时间戳到文件名，防止重名覆盖
        let output_path = add_timestamp_to_filename(&output_path);

        // 确保输出文件的父目录存在
        if let Some(parent) = output_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let result = match file.file_type.as_str() {
            "image" => process_image(&file.path, &output_path.to_string_lossy(), &options),
            "video" => process_video(&app, &file.path, &output_path.to_string_lossy(), &options).await,
            _ => Err("不支持的文件类型".to_string()),
        };

        match result {
            Ok(_) => success_count += 1,
            Err(e) => {
                error_count += 1;
                errors.push(format!("{}: {}", file.name, e));
            }
        }
    }

    // 发送完成事件
    let progress = ProcessProgress {
        current: total,
        total,
        current_file: "".to_string(),
        status: "completed".to_string(),
    };
    let _ = app.emit("process-progress", &progress);

    if error_count > 0 {
        Ok(format!(
            "处理完成: {} 成功, {} 失败\n失败详情:\n{}",
            success_count,
            error_count,
            errors.join("\n")
        ))
    } else {
        Ok(format!("全部处理完成: {} 个文件", success_count))
    }
}

#[tauri::command]
fn open_folder(path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        Command::new("explorer")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            get_input_dir,
            get_output_dir,
            set_custom_dirs,
            get_custom_dirs,
            scan_input_files,
            process_files,
            open_folder,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

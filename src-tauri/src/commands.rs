use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_shell::ShellExt;
use walkdir::WalkDir;

use crate::naming::{build_output_name, join_output_path, ratio_output_name};
use crate::process::{
    crop_image_by_ratio, crop_image_region, crop_video_region, process_image, process_video,
};
use crate::types::{
    CustomCropOptions, FileInfo, JoinOptions, NamingOptions, ProcessOptions, ProcessProgress,
    VideoMergeOptions, IMAGE_EXTENSIONS, VIDEO_EXTENSIONS,
};

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

fn default_naming() -> NamingOptions {
    NamingOptions::default()
}

fn even_dim(v: u32) -> u32 {
    (v & !1u32).max(2)
}

fn compute_cover_crop(width: u32, height: u32, target_ratio: f64) -> (u32, u32, u32, u32) {
    let video_ratio = width as f64 / height as f64;
    let (mut crop_w, mut crop_h, mut crop_x, mut crop_y) = if target_ratio < video_ratio {
        let ch = height;
        let cw = (height as f64 * target_ratio) as u32;
        let cx = (width.saturating_sub(cw)) / 2;
        (cw, ch, cx, 0u32)
    } else {
        let cw = width;
        let ch = (width as f64 / target_ratio) as u32;
        let cy = (height.saturating_sub(ch)) / 2;
        (cw, ch, 0u32, cy)
    };
    crop_w = even_dim(crop_w);
    crop_h = even_dim(crop_h);
    crop_x = crop_x & !1u32;
    crop_y = crop_y & !1u32;
    if crop_x + crop_w > width {
        crop_x = even_dim(width.saturating_sub(crop_w)) & !1u32;
    }
    if crop_y + crop_h > height {
        crop_y = even_dim(height.saturating_sub(crop_h)) & !1u32;
    }
    (crop_w, crop_h, crop_x, crop_y)
}

#[tauri::command]
pub fn get_input_dir(app: AppHandle) -> Result<String, String> {
    let app_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let input_dir = app_dir.join("input");
    std::fs::create_dir_all(&input_dir).map_err(|e| e.to_string())?;
    Ok(input_dir.to_string_lossy().to_string())
}

#[tauri::command]
pub fn get_output_dir(app: AppHandle) -> Result<String, String> {
    let app_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let output_dir = app_dir.join("output");
    std::fs::create_dir_all(&output_dir).map_err(|e| e.to_string())?;
    Ok(output_dir.to_string_lossy().to_string())
}

#[tauri::command]
pub fn set_custom_dirs(app: AppHandle, input_path: String, output_path: String) -> Result<(), String> {
    let app_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let config_path = app_dir.join("config.json");

    let mut config = if config_path.exists() {
        let content = std::fs::read_to_string(&config_path).unwrap_or_default();
        serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    config["input_dir"] = serde_json::json!(input_path);
    config["output_dir"] = serde_json::json!(output_path);

    std::fs::write(&config_path, serde_json::to_string_pretty(&config).unwrap())
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn get_custom_dirs(app: AppHandle) -> Result<(String, String), String> {
    let app_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let config_path = app_dir.join("config.json");

    if config_path.exists() {
        let content = std::fs::read_to_string(&config_path).map_err(|e| e.to_string())?;
        let config: serde_json::Value = serde_json::from_str(&content).map_err(|e| e.to_string())?;
        let input = config["input_dir"].as_str().unwrap_or("").to_string();
        let output = config["output_dir"].as_str().unwrap_or("").to_string();
        if !input.is_empty() && !output.is_empty() {
            return Ok((input, output));
        }
    }

    let input_dir = app_dir.join("input");
    let output_dir = app_dir.join("output");
    std::fs::create_dir_all(&input_dir).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&output_dir).map_err(|e| e.to_string())?;
    Ok((
        input_dir.to_string_lossy().to_string(),
        output_dir.to_string_lossy().to_string(),
    ))
}

#[tauri::command]
pub fn set_naming_options(app: AppHandle, naming: NamingOptions) -> Result<(), String> {
    let app_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let config_path = app_dir.join("config.json");

    let mut config = if config_path.exists() {
        let content = std::fs::read_to_string(&config_path).unwrap_or_default();
        serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    config["naming"] = serde_json::to_value(&naming).map_err(|e| e.to_string())?;
    std::fs::write(&config_path, serde_json::to_string_pretty(&config).unwrap())
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_naming_options(app: AppHandle) -> Result<NamingOptions, String> {
    let app_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let config_path = app_dir.join("config.json");

    if config_path.exists() {
        let content = std::fs::read_to_string(&config_path).map_err(|e| e.to_string())?;
        let config: serde_json::Value = serde_json::from_str(&content).map_err(|e| e.to_string())?;
        if let Some(naming) = config.get("naming") {
            if let Ok(opts) = serde_json::from_value::<NamingOptions>(naming.clone()) {
                return Ok(opts);
            }
        }
    }
    Ok(default_naming())
}

#[tauri::command]
pub fn scan_input_files(input_dir: String) -> Result<Vec<FileInfo>, String> {
    let mut files = Vec::new();
    let path = PathBuf::from(&input_dir);

    if !path.exists() {
        return Err("输入目录不存在".to_string());
    }

    for entry in WalkDir::new(&path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path().to_path_buf();
        if path.is_file() {
            if let Some(file_type) = get_file_type(&path) {
                let size_bytes = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                files.push(FileInfo {
                    path: path.to_string_lossy().to_string(),
                    name: path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string(),
                    file_type,
                    size_bytes,
                });
            }
        }
    }

    Ok(files)
}

#[tauri::command]
pub async fn get_video_dimensions(app: AppHandle, video_path: String) -> Result<(u32, u32), String> {
    let output = app
        .shell()
        .sidecar("ffprobe")
        .map_err(|e| format!("无法找到FFprobe: {}", e))?
        .args(&[
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-show_entries",
            "stream=width,height",
            "-of",
            "csv=p=0",
            &video_path,
        ])
        .output()
        .await
        .map_err(|e| format!("无法执行FFprobe: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("FFprobe执行失败: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parts: Vec<&str> = stdout.trim().split(',').collect();

    if parts.len() >= 2 {
        let width = parts[0].parse::<u32>().map_err(|_| "无法解析视频宽度")?;
        let height = parts[1].parse::<u32>().map_err(|_| "无法解析视频高度")?;
        Ok((width, height))
    } else {
        Err("无法获取视频尺寸".to_string())
    }
}

#[tauri::command]
pub async fn get_image_dimensions(app: AppHandle, image_path: String) -> Result<(u32, u32), String> {
    let output = app
        .shell()
        .sidecar("magick")
        .map_err(|e| format!("无法找到ImageMagick: {}", e))?
        .args(&["identify", "-format", "%w %h", &image_path])
        .output()
        .await
        .map_err(|e| format!("无法执行ImageMagick: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("读取图片尺寸失败: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parts: Vec<&str> = stdout.trim().split_whitespace().collect();
    if parts.len() >= 2 {
        let width = parts[0].parse::<u32>().map_err(|_| "无法解析图片宽度")?;
        let height = parts[1].parse::<u32>().map_err(|_| "无法解析图片高度")?;
        Ok((width, height))
    } else {
        Err("无法获取图片尺寸".to_string())
    }
}

#[tauri::command]
pub async fn extract_video_frame(app: AppHandle, video_path: String) -> Result<String, String> {
    let temp_dir = std::env::temp_dir();
    let frame_path = temp_dir.join(format!("crop_preview_{}.jpg", generate_timestamp()));
    let frame_path_str = frame_path.to_string_lossy().to_string();

    let output = app
        .shell()
        .sidecar("ffmpeg")
        .map_err(|e| format!("无法找到FFmpeg: {}", e))?
        .args(&["-i", &video_path, "-vframes", "1", "-q:v", "2", "-y", &frame_path_str])
        .output()
        .await
        .map_err(|e| format!("无法执行FFmpeg: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("提取视频帧失败: {}", stderr));
    }

    let image_data = std::fs::read(&frame_path).map_err(|e| format!("读取帧图片失败: {}", e))?;
    let _ = std::fs::remove_file(&frame_path);

    use base64::{engine::general_purpose::STANDARD, Engine as _};
    let base64_data = STANDARD.encode(&image_data);
    Ok(format!("data:image/jpeg;base64,{}", base64_data))
}

#[tauri::command]
pub async fn load_image_preview(image_path: String) -> Result<String, String> {
    let image_data = std::fs::read(&image_path).map_err(|e| format!("读取图片失败: {}", e))?;
    let ext = PathBuf::from(&image_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("jpg")
        .to_lowercase();
    let mime = match ext.as_str() {
        "png" => "image/png",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "bmp" => "image/bmp",
        _ => "image/jpeg",
    };
    use base64::{engine::general_purpose::STANDARD, Engine as _};
    let base64_data = STANDARD.encode(&image_data);
    Ok(format!("data:{};base64,{}", mime, base64_data))
}

#[tauri::command]
pub async fn process_files(
    app: AppHandle,
    input_dir: String,
    output_dir: String,
    options: ProcessOptions,
    file_type_filter: Option<String>,
    naming: Option<NamingOptions>,
    file_paths: Option<Vec<String>>,
) -> Result<String, String> {
    std::fs::create_dir_all(&output_dir).map_err(|e| e.to_string())?;

    let naming = naming.unwrap_or_else(default_naming);
    let mut files = scan_input_files(input_dir.clone())?;

    if let Some(ref filter) = file_type_filter {
        files.retain(|f| f.file_type == *filter);
    }
    if let Some(ref paths) = file_paths {
        let set: std::collections::HashSet<&str> = paths.iter().map(|s| s.as_str()).collect();
        files.retain(|f| set.contains(f.path.as_str()));
    }

    let total = files.len();
    if total == 0 {
        return Err("没有选中可处理的媒体文件".to_string());
    }

    let mut success_count = 0;
    let mut error_count = 0;
    let mut errors: Vec<String> = Vec::new();

    for (index, file) in files.iter().enumerate() {
        let progress = ProcessProgress {
            current: index + 1,
            total,
            current_file: file.name.clone(),
            status: "processing".to_string(),
        };
        let _ = app.emit("process-progress", &progress);

        let force_ext = if file.file_type == "video"
            && options.convert_format
            && !options.target_format.is_empty()
        {
            Some(options.target_format.as_str())
        } else if file.file_type == "image"
            && options.convert_format
            && !options.target_format.is_empty()
        {
            Some(options.target_format.as_str())
        } else {
            None
        };

        let out_name = build_output_name(&file.name, &naming, force_ext);
        let output_path = join_output_path(&output_dir, &out_name);

        if let Some(parent) = output_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let result = match file.file_type.as_str() {
            "image" => {
                process_image(&app, &file.path, &output_path.to_string_lossy(), &options).await
            }
            "video" => {
                process_video(&app, &file.path, &output_path.to_string_lossy(), &options).await
            }
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
pub async fn crop_by_ratios(
    app: AppHandle,
    input_dir: String,
    output_dir: String,
    ratios: Vec<String>,
    file_type_filter: Option<String>,
    file_paths: Option<Vec<String>>,
) -> Result<String, String> {
    if ratios.is_empty() {
        return Err("请至少添加一个比例".to_string());
    }

    std::fs::create_dir_all(&output_dir).map_err(|e| e.to_string())?;

    let mut files = scan_input_files(input_dir.clone())?;
    if let Some(ref filter) = file_type_filter {
        files.retain(|f| f.file_type == *filter);
    }
    if let Some(ref paths) = file_paths {
        let set: std::collections::HashSet<&str> = paths.iter().map(|s| s.as_str()).collect();
        files.retain(|f| set.contains(f.path.as_str()));
    }

    if files.is_empty() {
        return Err("没有选中可裁剪的媒体文件".to_string());
    }

    let total = files.len() * ratios.len();
    let mut success_count = 0;
    let mut error_count = 0;
    let mut errors: Vec<String> = Vec::new();
    let mut current = 0;

    for file in &files {
        let (width, height) = match file.file_type.as_str() {
            "video" => match get_video_dimensions(app.clone(), file.path.clone()).await {
                Ok(d) => d,
                Err(e) => {
                    for ratio in &ratios {
                        current += 1;
                        errors.push(format!("{} ({}): {}", file.name, ratio, e));
                        error_count += 1;
                    }
                    continue;
                }
            },
            "image" => match get_image_dimensions(app.clone(), file.path.clone()).await {
                Ok(d) => d,
                Err(e) => {
                    for ratio in &ratios {
                        current += 1;
                        errors.push(format!("{} ({}): {}", file.name, ratio, e));
                        error_count += 1;
                    }
                    continue;
                }
            },
            _ => continue,
        };

        if width == 0 || height == 0 {
            for ratio in &ratios {
                current += 1;
                errors.push(format!("{} ({}): 尺寸无效", file.name, ratio));
                error_count += 1;
            }
            continue;
        }

        let file_stem = PathBuf::from(&file.name)
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let orig_ext = PathBuf::from(&file.name)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or(if file.file_type == "video" { "mp4" } else { "jpg" })
            .to_string();

        for ratio in &ratios {
            current += 1;
            let progress = ProcessProgress {
                current,
                total,
                current_file: format!("{} ({})", file.name, ratio),
                status: "processing".to_string(),
            };
            let _ = app.emit("crop-progress", &progress);

            let ratio_parts: Vec<&str> = ratio.split(':').collect();
            if ratio_parts.len() != 2 {
                errors.push(format!("{} ({}): 比例格式无效", file.name, ratio));
                error_count += 1;
                continue;
            }

            let rw: f64 = ratio_parts[0].parse().unwrap_or(0.0);
            let rh: f64 = ratio_parts[1].parse().unwrap_or(0.0);
            if rw <= 0.0 || rh <= 0.0 {
                errors.push(format!("{} ({}): 比例值无效", file.name, ratio));
                error_count += 1;
                continue;
            }

            let target_ratio = rw / rh;
            let (crop_w, crop_h, crop_x, crop_y) = compute_cover_crop(width, height, target_ratio);

            let out_ext = if file.file_type == "video" {
                "mp4"
            } else {
                orig_ext.as_str()
            };
            let output_filename = ratio_output_name(&file_stem, ratio, target_ratio, out_ext);
            let output_path = PathBuf::from(&output_dir).join(&output_filename);

            let result = match file.file_type.as_str() {
                "video" => {
                    crop_video_region(
                        &app,
                        &file.path,
                        &output_path.to_string_lossy(),
                        crop_x,
                        crop_y,
                        crop_w,
                        crop_h,
                    )
                    .await
                }
                "image" => {
                    crop_image_by_ratio(
                        &app,
                        &file.path,
                        &output_path.to_string_lossy(),
                        crop_w,
                        crop_h,
                        crop_x,
                        crop_y,
                    )
                    .await
                }
                _ => Err("不支持的文件类型".to_string()),
            };

            match result {
                Ok(_) => success_count += 1,
                Err(e) => {
                    errors.push(format!("{} ({}): {}", file.name, ratio, e));
                    error_count += 1;
                }
            }
        }
    }

    let progress = ProcessProgress {
        current: total,
        total,
        current_file: "".to_string(),
        status: "completed".to_string(),
    };
    let _ = app.emit("crop-progress", &progress);

    if error_count > 0 {
        Ok(format!(
            "裁剪完成: {} 成功, {} 失败\n失败详情:\n{}",
            success_count,
            error_count,
            errors.join("\n")
        ))
    } else {
        Ok(format!("全部裁剪完成: {} 个文件", success_count))
    }
}

// 兼容旧命令名
#[tauri::command]
pub async fn crop_videos_by_ratios(
    app: AppHandle,
    input_dir: String,
    output_dir: String,
    ratios: Vec<String>,
) -> Result<String, String> {
    crop_by_ratios(
        app,
        input_dir,
        output_dir,
        ratios,
        Some("video".to_string()),
        None,
    )
    .await
}

#[tauri::command]
pub async fn get_file_thumbnail(app: AppHandle, path: String, file_type: String) -> Result<String, String> {
    if file_type == "video" {
        extract_video_frame(app, path).await
    } else {
        // 用 ImageMagick 生成小缩略图，避免大图 base64 过重
        let temp_dir = std::env::temp_dir();
        let out = temp_dir.join(format!("thumb_{}.jpg", generate_timestamp()));
        let out_str = out.to_string_lossy().to_string();
        let output = app
            .shell()
            .sidecar("magick")
            .map_err(|e| format!("无法找到ImageMagick: {}", e))?
            .args(&[
                &path,
                "-thumbnail",
                "96x96^",
                "-gravity",
                "center",
                "-extent",
                "96x96",
                "-quality",
                "70",
                &out_str,
            ])
            .output()
            .await
            .map_err(|e| format!("无法执行ImageMagick: {}", e))?;

        if !output.status.success() {
            // 回退为原图预览
            return load_image_preview(path).await;
        }

        let image_data = std::fs::read(&out).map_err(|e| format!("读取缩略图失败: {}", e))?;
        let _ = std::fs::remove_file(&out);
        use base64::{engine::general_purpose::STANDARD, Engine as _};
        Ok(format!("data:image/jpeg;base64,{}", STANDARD.encode(&image_data)))
    }
}

#[tauri::command]
pub async fn custom_crop(app: AppHandle, options: CustomCropOptions) -> Result<String, String> {
    if options.crop_width == 0 || options.crop_height == 0 {
        return Err("裁剪尺寸无效".to_string());
    }

    let input = PathBuf::from(&options.input_path);
    if !input.is_file() {
        return Err("输入文件不存在".to_string());
    }

    std::fs::create_dir_all(&options.output_dir).map_err(|e| e.to_string())?;

    let file_name = input
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "crop_output".to_string());
    let file_type = get_file_type(&input).unwrap_or_else(|| "image".to_string());

    let force_ext = if file_type == "video" {
        Some("mp4")
    } else {
        None
    };
    let out_name = build_output_name(&file_name, &options.naming, force_ext);
    let output_path = join_output_path(&options.output_dir, &out_name);

    let result = match file_type.as_str() {
        "video" => {
            crop_video_region(
                &app,
                &options.input_path,
                &output_path.to_string_lossy(),
                options.crop_x,
                options.crop_y,
                options.crop_width,
                options.crop_height,
            )
            .await
        }
        "image" => {
            crop_image_region(
                &app,
                &options.input_path,
                &output_path.to_string_lossy(),
                options.crop_x,
                options.crop_y,
                options.crop_width,
                options.crop_height,
            )
            .await
        }
        _ => Err("不支持的文件类型".to_string()),
    };

    result?;
    Ok(format!("裁剪完成: {}", output_path.to_string_lossy()))
}

/// 规范化输出帧率：仅允许 30 / 60
fn normalize_output_fps(fps: Option<u32>) -> Option<u32> {
    match fps {
        Some(30) => Some(30),
        Some(60) => Some(60),
        _ => None,
    }
}

/// fps 滤镜：升帧会补帧（重复帧），降帧会丢帧
fn fps_filter_segment(fps: u32) -> String {
    format!("fps={}", fps)
}

fn validate_video_level(level: &str) -> bool {
    matches!(
        level,
        "3.0" | "3.1" | "3.2" | "4.0" | "4.1" | "4.2" | "5.0" | "5.1" | "5.2"
    )
}

fn validate_video_profile(profile: &str) -> bool {
    matches!(profile, "baseline" | "main" | "high")
}

#[tauri::command]
pub async fn merge_videos(app: AppHandle, options: VideoMergeOptions) -> Result<String, String> {
    validate_merge_options(&options)?;

    if let Some(parent) = PathBuf::from(&options.output_path).parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let first = &options.slots[0];
    let second = &options.slots[1];
    let is_image = options.media_kind == "image";
    let stack_filter = if options.layout == "vertical" {
        "vstack"
    } else {
        "hstack"
    };
    let out_fps = if is_image {
        None
    } else {
        normalize_output_fps(options.output_fps)
    };

    // 视频：各路先 scale，再 fps（不足 60 时补帧），再 stack
    let (s0, s1) = if let Some(fps) = out_fps {
        let f = fps_filter_segment(fps);
        (
            format!("scale={}:{},{},setsar=1", first.width, first.height, f),
            format!("scale={}:{},{},setsar=1", second.width, second.height, f),
        )
    } else {
        (
            format!("scale={}:{},setsar=1", first.width, first.height),
            format!("scale={}:{},setsar=1", second.width, second.height),
        )
    };

    let mut filter = format!(
        "[0:v]{}[v0];[1:v]{}[v1];[v0][v1]{}=inputs=2{}[v]",
        s0,
        s1,
        stack_filter,
        if is_image { "" } else { ":shortest=1" },
    );

    if let (Some(width), Some(height)) = (options.output_width, options.output_height) {
        filter.push_str(&format!(";[v]scale={}:{},setsar=1[outv]", width, height));
    } else {
        filter.push_str(";[v]setsar=1[outv]");
    }

    let mut args: Vec<String> = vec![
        "-i".to_string(),
        first.path.clone(),
        "-i".to_string(),
        second.path.clone(),
        "-filter_complex".to_string(),
        filter,
        "-map".to_string(),
        "[outv]".to_string(),
    ];

    if is_image {
        args.extend(["-frames:v".to_string(), "1".to_string()]);
    } else {
        args.extend([
            "-an".to_string(),
            "-c:v".to_string(),
            "libx264".to_string(),
            "-pix_fmt".to_string(),
            "yuv420p".to_string(),
        ]);
        if let Some(fps) = out_fps {
            args.push("-r".to_string());
            args.push(fps.to_string());
        }
        if options.set_level {
            let profile = options
                .video_profile
                .as_deref()
                .unwrap_or("high")
                .to_lowercase();
            let level = options.video_level.as_deref().unwrap_or("4.0");
            if !validate_video_profile(&profile) {
                return Err("Profile 无效".to_string());
            }
            if !validate_video_level(level) {
                return Err("Level 无效".to_string());
            }
            args.extend([
                "-profile:v".to_string(),
                profile,
                "-level:v".to_string(),
                level.to_string(),
            ]);
        }
    }

    args.push("-y".to_string());
    args.push(options.output_path.clone());

    let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

    let output = app
        .shell()
        .sidecar("ffmpeg")
        .map_err(|e| format!("无法找到FFmpeg: {}", e))?
        .args(&arg_refs)
        .output()
        .await
        .map_err(|e| format!("无法执行FFmpeg: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("拼接失败: {}", stderr));
    }

    Ok(format!("拼接完成: {}", options.output_path))
}

fn join_bg_color(background: &str, is_image: bool) -> Result<String, String> {
    let bg = background.trim().to_lowercase();
    match bg.as_str() {
        "transparent" if is_image => Ok("black@0".to_string()),
        "transparent" => Ok("black".to_string()),
        "#ffffff" | "white" => Ok("white".to_string()),
        "#000000" | "black" | "" => Ok("black".to_string()),
        s if s.starts_with('#') && s.len() == 7 => {
            let hex = &s[1..];
            if hex.chars().all(|c| c.is_ascii_hexdigit()) {
                Ok(format!("0x{}", hex))
            } else {
                Err("背景色无效".to_string())
            }
        }
        _ => Err("背景色无效".to_string()),
    }
}

fn join_item_scale_filter(fit: &str, w: u32, h: u32, pad_color: &str) -> Result<String, String> {
    let w = even_dim(w);
    let h = even_dim(h);
    match fit {
        "fill" => Ok(format!("scale={}:{}:flags=bicubic,setsar=1", w, h)),
        "cover" => Ok(format!(
            "scale={}:{}:force_original_aspect_ratio=increase:flags=bicubic,crop={}:{},setsar=1",
            w, h, w, h
        )),
        "contain" => Ok(format!(
            "scale={}:{}:force_original_aspect_ratio=decrease:flags=bicubic,pad={}:{}:(ow-iw)/2:(oh-ih)/2:color={},setsar=1",
            w, h, w, h, pad_color
        )),
        _ => Err(format!("不支持的填充模式: {}", fit)),
    }
}

fn resolve_join_output_kind(items: &[crate::types::JoinItem]) -> String {
    if items.iter().any(|i| i.media_kind == "video") {
        "video".to_string()
    } else {
        "image".to_string()
    }
}

fn validate_join_options(options: &JoinOptions) -> Result<String, String> {
    if options.output_path.trim().is_empty() {
        return Err("输出路径不能为空".to_string());
    }
    if options.canvas_width < 2 || options.canvas_height < 2 {
        return Err("画布尺寸无效".to_string());
    }
    if options.canvas_width > 7680 || options.canvas_height > 7680 {
        return Err("画布尺寸过大".to_string());
    }
    if options.items.is_empty() {
        return Err("请至少添加一个素材".to_string());
    }
    if options.items.len() > 6 {
        return Err("自定义拼接最多支持 6 个素材".to_string());
    }

    for (index, item) in options.items.iter().enumerate() {
        if item.path.trim().is_empty() {
            return Err(format!("第 {} 个素材路径为空", index + 1));
        }
        if item.width < 2 || item.height < 2 {
            return Err(format!("第 {} 个素材尺寸无效", index + 1));
        }
        if !PathBuf::from(&item.path).is_file() {
            return Err(format!("第 {} 个文件不存在: {}", index + 1, item.name));
        }
        if !matches!(item.fit.as_str(), "cover" | "contain" | "fill") {
            return Err(format!("第 {} 个素材填充模式无效", index + 1));
        }
        if item.media_kind != "image" && item.media_kind != "video" {
            return Err(format!("第 {} 个素材类型无效", index + 1));
        }
    }

    let output_kind = resolve_join_output_kind(&options.items);
    Ok(output_kind)
}

#[tauri::command]
pub async fn join_media(app: AppHandle, options: JoinOptions) -> Result<String, String> {
    let output_kind = validate_join_options(&options)?;
    let is_image = output_kind == "image";

    if let Some(parent) = PathBuf::from(&options.output_path).parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let canvas_w = even_dim(options.canvas_width);
    let canvas_h = even_dim(options.canvas_height);
    let bg = join_bg_color(&options.background, is_image)?;
    let pad_color = bg.clone();

    let mut items = options.items.clone();
    items.sort_by_key(|i| i.z);

    let out_fps = if is_image {
        None
    } else {
        normalize_output_fps(options.output_fps)
    };

    let mut filter_parts: Vec<String> = Vec::new();
    let base_dur = if is_image { "1" } else { "999999" };
    let base_fmt = if is_image && options.background.trim().eq_ignore_ascii_case("transparent") {
        "format=rgba"
    } else {
        "format=yuv420p"
    };
    // 底图与输出帧率一致，避免 overlay 时间基错乱
    let base_rate = match out_fps {
        Some(fps) => format!(":r={}", fps),
        None => String::new(),
    };
    filter_parts.push(format!(
        "color=c={}:s={}x{}:d={}{},{}[base]",
        bg, canvas_w, canvas_h, base_dur, base_rate, base_fmt
    ));

    for (i, item) in items.iter().enumerate() {
        let scale = join_item_scale_filter(&item.fit, item.width, item.height, &pad_color)?;
        // 视频输出：各图层统一 fps（不足目标帧率时补帧），再统一像素格式
        let chain = if let Some(fps) = out_fps {
            format!("{},{},{}", scale, fps_filter_segment(fps), base_fmt)
        } else {
            format!("{},{}", scale, base_fmt)
        };
        filter_parts.push(format!("[{}:v]{}[v{}]", i, chain, i));
    }

    let mut prev = "base".to_string();
    for (i, item) in items.iter().enumerate() {
        let out = if i + 1 == items.len() {
            "outv".to_string()
        } else {
            format!("b{}", i)
        };
        // 视频输出：shortest=1，时长由最短视频轨决定（静图 loop 不会先结束）
        let shortest = if is_image { "" } else { ":shortest=1" };
        filter_parts.push(format!(
            "[{}][v{}]overlay={}:{}{}[{}]",
            prev, i, item.x, item.y, shortest, out
        ));
        prev = out;
    }

    let filter = filter_parts.join(";");

    let mut args: Vec<String> = Vec::new();
    for item in &items {
        // 混合/视频输出时，图片需 loop，否则 overlay 只有一帧
        if !is_image && item.media_kind == "image" {
            args.push("-loop".to_string());
            args.push("1".to_string());
        }
        args.push("-i".to_string());
        args.push(item.path.clone());
    }
    args.push("-filter_complex".to_string());
    args.push(filter);
    args.push("-map".to_string());
    args.push("[outv]".to_string());

    if is_image {
        args.extend([
            "-frames:v".to_string(),
            "1".to_string(),
            "-pix_fmt".to_string(),
            if options.background.trim().eq_ignore_ascii_case("transparent") {
                "rgba".to_string()
            } else {
                "rgb24".to_string()
            },
        ]);
    } else {
        args.extend([
            "-an".to_string(),
            "-c:v".to_string(),
            "libx264".to_string(),
            "-pix_fmt".to_string(),
            "yuv420p".to_string(),
            "-movflags".to_string(),
            "+faststart".to_string(),
        ]);
        if let Some(fps) = out_fps {
            args.push("-r".to_string());
            args.push(fps.to_string());
        }
        if options.set_level {
            let profile = options
                .video_profile
                .as_deref()
                .unwrap_or("high")
                .to_lowercase();
            let level = options.video_level.as_deref().unwrap_or("4.0");
            if !validate_video_profile(&profile) {
                return Err("Profile 无效".to_string());
            }
            if !validate_video_level(level) {
                return Err("Level 无效".to_string());
            }
            args.extend([
                "-profile:v".to_string(),
                profile,
                "-level:v".to_string(),
                level.to_string(),
            ]);
        }
    }

    args.push("-y".to_string());
    args.push(options.output_path.clone());

    let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

    let output = app
        .shell()
        .sidecar("ffmpeg")
        .map_err(|e| format!("无法找到FFmpeg: {}", e))?
        .args(&arg_refs)
        .output()
        .await
        .map_err(|e| format!("无法执行FFmpeg: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("自定义拼接失败: {}", stderr));
    }

    Ok(format!("自定义拼接完成: {}", options.output_path))
}

fn validate_merge_options(options: &VideoMergeOptions) -> Result<(), String> {
    if options.output_path.trim().is_empty() {
        return Err("输出路径不能为空".to_string());
    }

    for (index, slot) in options.slots.iter().enumerate() {
        if slot.path.trim().is_empty() {
            return Err(format!("请为第 {} 个坑位选择文件", index + 1));
        }
        if slot.width == 0 || slot.height == 0 {
            return Err(format!("第 {} 个坑位尺寸无效", index + 1));
        }
        if !PathBuf::from(&slot.path).is_file() {
            return Err(format!("第 {} 个文件不存在", index + 1));
        }
    }

    match options.layout.as_str() {
        "vertical" if options.slots[0].width != options.slots[1].width => {
            Err("上下拼接时两个坑位宽度必须一致".to_string())
        }
        "horizontal" if options.slots[0].height != options.slots[1].height => {
            Err("左右拼接时两个坑位高度必须一致".to_string())
        }
        "vertical" | "horizontal" => {
            if options.output_width.is_some() != options.output_height.is_some() {
                return Err("输出宽高必须同时填写".to_string());
            }
            if matches!(options.output_width, Some(0)) || matches!(options.output_height, Some(0)) {
                return Err("输出分辨率无效".to_string());
            }
            Ok(())
        }
        _ => Err("拼接布局无效".to_string()),
    }
}

#[tauri::command]
pub fn open_folder(path: String) -> Result<(), String> {
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

#[tauri::command]
pub fn start_upload_server(input_dir: String) -> Result<String, String> {
    crate::upload_server::start_server(input_dir)
}

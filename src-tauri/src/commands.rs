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
    CustomCropOptions, FileInfo, NamingOptions, ProcessOptions, ProcessProgress, VideoMergeOptions,
    IMAGE_EXTENSIONS, VIDEO_EXTENSIONS,
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

    let mut filter = format!(
        "[0:v]scale={}:{},setsar=1[v0];[1:v]scale={}:{},setsar=1[v1];[v0][v1]{}=inputs=2{}[v]",
        first.width,
        first.height,
        second.width,
        second.height,
        stack_filter,
        if is_image { "" } else { ":shortest=1" },
    );

    let output_label =
        if let (Some(width), Some(height)) = (options.output_width, options.output_height) {
            filter.push_str(&format!(";[v]scale={}:{},setsar=1[outv]", width, height));
            "[outv]"
        } else {
            filter.push_str(";[v]setsar=1[outv]");
            "[outv]"
        };

    let mut args: Vec<&str> = vec![
        "-i",
        &first.path,
        "-i",
        &second.path,
        "-filter_complex",
        &filter,
        "-map",
        output_label,
    ];

    if is_image {
        args.extend_from_slice(&["-frames:v", "1"]);
    } else {
        args.extend_from_slice(&["-an", "-c:v", "libx264", "-pix_fmt", "yuv420p"]);
    }

    args.push("-y");
    args.push(&options.output_path);

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
        return Err(format!("拼接失败: {}", stderr));
    }

    Ok(format!("拼接完成: {}", options.output_path))
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

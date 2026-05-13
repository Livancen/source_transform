use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_shell::ShellExt;
use walkdir::WalkDir;

use crate::process::{process_image, process_video};
use crate::types::{FileInfo, ProcessOptions, ProcessProgress, IMAGE_EXTENSIONS, VIDEO_EXTENSIONS};

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

fn change_extension(path: &PathBuf, new_ext: &str) -> PathBuf {
    path.with_extension(new_ext)
}

#[tauri::command]
pub fn get_input_dir(app: AppHandle) -> Result<String, String> {
    let app_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    let input_dir = app_dir.join("input");
    std::fs::create_dir_all(&input_dir).map_err(|e| e.to_string())?;
    Ok(input_dir.to_string_lossy().to_string())
}

#[tauri::command]
pub fn get_output_dir(app: AppHandle) -> Result<String, String> {
    let app_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    let output_dir = app_dir.join("output");
    std::fs::create_dir_all(&output_dir).map_err(|e| e.to_string())?;
    Ok(output_dir.to_string_lossy().to_string())
}

#[tauri::command]
pub fn set_custom_dirs(app: AppHandle, input_path: String, output_path: String) -> Result<(), String> {
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
pub fn get_custom_dirs(app: AppHandle) -> Result<(String, String), String> {
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

#[tauri::command]
pub async fn get_video_dimensions(app: AppHandle, video_path: String) -> Result<(u32, u32), String> {
    // 使用ffprobe获取视频尺寸
    let output = app
        .shell()
        .sidecar("ffprobe")
        .map_err(|e| format!("无法找到FFprobe: {}", e))?
        .args(&[
            "-v", "error",
            "-select_streams", "v:0",
            "-show_entries", "stream=width,height",
            "-of", "csv=p=0",
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
pub async fn extract_video_frame(app: AppHandle, video_path: String) -> Result<String, String> {
    // 获取临时目录
    let temp_dir = std::env::temp_dir();
    let frame_path = temp_dir.join(format!("crop_preview_{}.jpg", generate_timestamp()));
    let frame_path_str = frame_path.to_string_lossy().to_string();

    // 使用ffmpeg提取第一帧
    let output = app
        .shell()
        .sidecar("ffmpeg")
        .map_err(|e| format!("无法找到FFmpeg: {}", e))?
        .args(&[
            "-i", &video_path,
            "-vframes", "1",
            "-q:v", "2",
            "-y",
            &frame_path_str,
        ])
        .output()
        .await
        .map_err(|e| format!("无法执行FFmpeg: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("提取视频帧失败: {}", stderr));
    }

    // 读取图片文件并转换为Base64
    let image_data = std::fs::read(&frame_path)
        .map_err(|e| format!("读取帧图片失败: {}", e))?;

    // 删除临时文件
    let _ = std::fs::remove_file(&frame_path);

    // 返回Base64数据URL
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    let base64_data = STANDARD.encode(&image_data);
    Ok(format!("data:image/jpeg;base64,{}", base64_data))
}

#[tauri::command]
pub async fn process_files(
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

        // 构建输出路径（直接使用原文件名，已存在则覆盖）
        let input_path = PathBuf::from(&file.path);
        let relative_path = input_path
            .strip_prefix(&input_dir)
            .unwrap_or(&input_path);
        let output_path = PathBuf::from(&output_dir).join(relative_path);

        // 确保输出文件的父目录存在
        if let Some(parent) = output_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let result = match file.file_type.as_str() {
            "image" => process_image(&app, &file.path, &output_path.to_string_lossy(), &options).await,
            "video" => {
                // 如果启用格式转换，更改输出文件扩展名
                let final_output_path = if options.convert_format && !options.target_format.is_empty() {
                    change_extension(&output_path, &options.target_format)
                } else {
                    output_path.clone()
                };
                process_video(&app, &file.path, &final_output_path.to_string_lossy(), &options).await
            },
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
pub async fn crop_videos_by_ratios(
    app: AppHandle,
    input_dir: String,
    output_dir: String,
    ratios: Vec<String>,
) -> Result<String, String> {
    if ratios.is_empty() {
        return Err("请至少添加一个比例".to_string());
    }

    std::fs::create_dir_all(&output_dir).map_err(|e| e.to_string())?;

    let files = scan_input_files(input_dir.clone())?;
    let video_files: Vec<_> = files.into_iter().filter(|f| f.file_type == "video").collect();

    if video_files.is_empty() {
        return Err("输入目录中没有找到视频文件".to_string());
    }

    let total = video_files.len() * ratios.len();
    let mut success_count = 0;
    let mut error_count = 0;
    let mut errors: Vec<String> = Vec::new();
    let mut current = 0;

    for file in &video_files {
        // 获取视频尺寸
        let dim_output = app
            .shell()
            .sidecar("ffprobe")
            .map_err(|e| format!("无法找到FFprobe: {}", e))?
            .args(&[
                "-v", "error",
                "-select_streams", "v:0",
                "-show_entries", "stream=width,height",
                "-of", "csv=p=0",
                &file.path,
            ])
            .output()
            .await
            .map_err(|e| format!("无法执行FFprobe: {}", e))?;

        if !dim_output.status.success() {
            for ratio in &ratios {
                current += 1;
                errors.push(format!("{} ({}): 无法获取视频尺寸", file.name, ratio));
                error_count += 1;
            }
            continue;
        }

        let stdout = String::from_utf8_lossy(&dim_output.stdout);
        let parts: Vec<&str> = stdout.trim().split(',').collect();
        if parts.len() < 2 {
            for ratio in &ratios {
                current += 1;
                errors.push(format!("{} ({}): 无法解析视频尺寸", file.name, ratio));
                error_count += 1;
            }
            continue;
        }

        let width: u32 = parts[0].parse().unwrap_or(0);
        let height: u32 = parts[1].parse().unwrap_or(0);
        if width == 0 || height == 0 {
            for ratio in &ratios {
                current += 1;
                errors.push(format!("{} ({}): 视频尺寸无效", file.name, ratio));
                error_count += 1;
            }
            continue;
        }

        for ratio in &ratios {
            current += 1;

            // 发送进度事件
            let progress = ProcessProgress {
                current,
                total,
                current_file: format!("{} ({})", file.name, ratio),
                status: "processing".to_string(),
            };
            let _ = app.emit("crop-progress", &progress);

            // 解析比例 "W:H"
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
            let video_ratio = width as f64 / height as f64;

            // object-fit: cover 裁剪算法
            let (crop_w, crop_h, crop_x, crop_y) = if target_ratio < video_ratio {
                // 目标更窄，以高度为基准裁剪宽度
                let ch = height;
                let cw = (height as f64 * target_ratio) as u32;
                let cx = (width - cw) / 2;
                (cw, ch, cx, 0u32)
            } else {
                // 目标更宽，以宽度为基准裁剪高度
                let cw = width;
                let ch = (width as f64 / target_ratio) as u32;
                let cy = (height - ch) / 2;
                (cw, ch, 0u32, cy)
            };

            // 构建输出文件名: name_ratio.mp4
            let file_stem = PathBuf::from(&file.name)
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let ratio_filename = ratio.replace(':', "-");
            let output_filename = format!("{}_{}.mp4", file_stem, ratio_filename);
            let output_path = PathBuf::from(&output_dir).join(&output_filename);

            let crop_filter = format!("crop={}:{}:{}:{}", crop_w, crop_h, crop_x, crop_y);

            let result = app
                .shell()
                .sidecar("ffmpeg")
                .map_err(|e| format!("无法找到FFmpeg: {}", e))?
                .args(&[
                    "-i", &file.path,
                    "-vf", &crop_filter,
                    "-y",
                    &output_path.to_string_lossy(),
                ])
                .output()
                .await
                .map_err(|e| format!("无法执行FFmpeg: {}", e));

            match result {
                Ok(output) if output.status.success() => success_count += 1,
                Ok(output) => {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    errors.push(format!("{} ({}): {}", file.name, ratio, stderr));
                    error_count += 1;
                }
                Err(e) => {
                    errors.push(format!("{} ({}): {}", file.name, ratio, e));
                    error_count += 1;
                }
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

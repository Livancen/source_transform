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
    pub reduce_level: bool,
    pub target_level: String, // e.g., "3.0", "4.0", "5.1"
    pub convert_h265_to_h264: bool,
    pub convert_format: bool,
    pub target_format: String, // e.g., "mp4", "avi", "mkv"
    pub crop: bool,
    pub crop_width: u32,
    pub crop_height: u32,
    pub crop_x: u32,
    pub crop_y: u32,
    pub rotate: bool,
    pub rotation_degrees: i32, // 90, 180, 270, -90
    pub mute: bool,            // 视频静音（去除音频）
    pub change_framerate: bool, // 调整帧率
    pub target_framerate: f32,  // 目标帧率
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VideoInfo {
    pub path: String,
    pub name: String,
    pub codec: String,
    pub profile: String,
    pub level: String,
    pub width: u32,
    pub height: u32,
    pub framerate: f64,
    pub bitrate: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeviceCompatibility {
    pub device: String,
    pub compatible: bool,
    pub reason: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VideoCompatibilityResult {
    pub path: String,
    pub name: String,
    pub video_info: VideoInfo,
    pub devices: Vec<DeviceCompatibility>,
    pub thumbnail: String, // Base64 encoded thumbnail
}

// 图片信息结构体
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImageInfo {
    pub path: String,
    pub name: String,
    pub codec: String,         // 编码格式，如 mjpeg, png
    pub pix_fmt: String,       // 像素格式，如 yuvj420p, gbrap, rgb24
    pub color_space: String,   // 色彩空间，如 bt709, bt470bg
    pub color_range: String,   // 色彩范围，如 pc (full), tv (limited)
    pub width: u32,
    pub height: u32,
}

// 图片兼容性问题
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImageCompatibilityIssue {
    pub issue_type: String,    // 问题类型：pix_fmt, color_space, resolution
    pub severity: String,      // 严重程度：error, warning, info
    pub description: String,   // 问题描述
}

// 图片兼容性检测结果
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImageCompatibilityResult {
    pub path: String,
    pub name: String,
    pub image_info: ImageInfo,
    pub compatible: bool,          // 是否完全兼容
    pub issues: Vec<ImageCompatibilityIssue>,  // 兼容性问题列表
    pub thumbnail: String,         // Base64 encoded thumbnail
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

fn change_extension(path: &PathBuf, new_ext: &str) -> PathBuf {
    path.with_extension(new_ext)
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

#[tauri::command]
async fn get_video_dimensions(app: AppHandle, video_path: String) -> Result<(u32, u32), String> {
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
async fn extract_video_frame(app: AppHandle, video_path: String) -> Result<String, String> {
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

// 检查设备兼容性
fn check_device_compatibility(info: &VideoInfo) -> Vec<DeviceCompatibility> {
    let mut results = Vec::new();
    let codec = info.codec.to_lowercase();

    // 解析level为数字
    let level_num: f64 = info.level.parse().unwrap_or(0.0);

    // RK3399 兼容性检查
    // H.264: 4K@30fps, Level 5.1
    // H.265: 4K@60fps, Level 5.1
    // VP9: 4K@30fps
    let rk3399 = {
        let mut compatible = true;
        let mut reason = String::from("兼容");

        if codec.contains("h264") || codec.contains("avc") {
            if info.width > 4096 || info.height > 2160 {
                compatible = false;
                reason = format!("分辨率 {}x{} 超过 4K 限制", info.width, info.height);
            } else if level_num > 5.1 {
                compatible = false;
                reason = format!("Level {} 超过 5.1 限制", info.level);
            } else if info.width > 1920 && info.framerate > 30.0 {
                compatible = false;
                reason = format!("4K@{}fps 超过 4K@30fps 限制", info.framerate as u32);
            }
        } else if codec.contains("h265") || codec.contains("hevc") {
            if info.width > 4096 || info.height > 2160 {
                compatible = false;
                reason = format!("分辨率 {}x{} 超过 4K 限制", info.width, info.height);
            } else if level_num > 5.1 {
                compatible = false;
                reason = format!("Level {} 超过 5.1 限制", info.level);
            }
        } else if codec.contains("vp9") {
            if info.width > 4096 || info.height > 2160 {
                compatible = false;
                reason = format!("分辨率 {}x{} 超过 4K 限制", info.width, info.height);
            } else if info.framerate > 30.0 {
                compatible = false;
                reason = format!("VP9 {}fps 超过 30fps 限制", info.framerate as u32);
            }
        } else if codec.contains("av1") {
            compatible = false;
            reason = String::from("不支持 AV1 编码");
        } else {
            compatible = false;
            reason = format!("不支持 {} 编码", info.codec);
        }

        DeviceCompatibility {
            device: "RK3399".to_string(),
            compatible,
            reason,
        }
    };
    results.push(rk3399);

    // RK3566 兼容性检查
    // H.264: 4K@30fps, Level 5.1
    // H.265: 4K@60fps, Level 5.1
    // VP9: 4K@60fps
    let rk3566 = {
        let mut compatible = true;
        let mut reason = String::from("兼容");

        if codec.contains("h264") || codec.contains("avc") {
            if info.width > 4096 || info.height > 2160 {
                compatible = false;
                reason = format!("分辨率 {}x{} 超过 4K 限制", info.width, info.height);
            } else if level_num > 5.1 {
                compatible = false;
                reason = format!("Level {} 超过 5.1 限制", info.level);
            } else if info.width > 1920 && info.framerate > 30.0 {
                compatible = false;
                reason = format!("4K@{}fps 超过 4K@30fps 限制", info.framerate as u32);
            }
        } else if codec.contains("h265") || codec.contains("hevc") {
            if info.width > 4096 || info.height > 2160 {
                compatible = false;
                reason = format!("分辨率 {}x{} 超过 4K 限制", info.width, info.height);
            } else if level_num > 5.1 {
                compatible = false;
                reason = format!("Level {} 超过 5.1 限制", info.level);
            }
        } else if codec.contains("vp9") {
            if info.width > 4096 || info.height > 2160 {
                compatible = false;
                reason = format!("分辨率 {}x{} 超过 4K 限制", info.width, info.height);
            }
        } else if codec.contains("av1") {
            compatible = false;
            reason = String::from("不支持 AV1 编码");
        } else {
            compatible = false;
            reason = format!("不支持 {} 编码", info.codec);
        }

        DeviceCompatibility {
            device: "RK3566".to_string(),
            compatible,
            reason,
        }
    };
    results.push(rk3566);

    // RK3588 兼容性检查
    // H.264: 8K@30fps, Level 6.0
    // H.265: 8K@60fps, Level 6.1
    // VP9: 8K@30fps
    // AV1: 8K@30fps
    let rk3588 = {
        let mut compatible = true;
        let mut reason = String::from("兼容");

        if codec.contains("h264") || codec.contains("avc") {
            if info.width > 7680 || info.height > 4320 {
                compatible = false;
                reason = format!("分辨率 {}x{} 超过 8K 限制", info.width, info.height);
            } else if level_num > 6.0 {
                compatible = false;
                reason = format!("Level {} 超过 6.0 限制", info.level);
            } else if info.width > 4096 && info.framerate > 30.0 {
                compatible = false;
                reason = format!("8K@{}fps 超过 8K@30fps 限制", info.framerate as u32);
            }
        } else if codec.contains("h265") || codec.contains("hevc") {
            if info.width > 7680 || info.height > 4320 {
                compatible = false;
                reason = format!("分辨率 {}x{} 超过 8K 限制", info.width, info.height);
            } else if level_num > 6.1 {
                compatible = false;
                reason = format!("Level {} 超过 6.1 限制", info.level);
            }
        } else if codec.contains("vp9") {
            if info.width > 7680 || info.height > 4320 {
                compatible = false;
                reason = format!("分辨率 {}x{} 超过 8K 限制", info.width, info.height);
            } else if info.framerate > 30.0 {
                compatible = false;
                reason = format!("VP9 {}fps 超过 30fps 限制", info.framerate as u32);
            }
        } else if codec.contains("av1") {
            if info.width > 7680 || info.height > 4320 {
                compatible = false;
                reason = format!("分辨率 {}x{} 超过 8K 限制", info.width, info.height);
            } else if info.framerate > 30.0 {
                compatible = false;
                reason = format!("AV1 {}fps 超过 30fps 限制", info.framerate as u32);
            }
        } else {
            compatible = false;
            reason = format!("不支持 {} 编码", info.codec);
        }

        DeviceCompatibility {
            device: "RK3588".to_string(),
            compatible,
            reason,
        }
    };
    results.push(rk3588);

    results
}

#[tauri::command]
async fn detect_video_compatibility(app: AppHandle, input_dir: String) -> Result<Vec<VideoCompatibilityResult>, String> {
    let files = scan_input_files(input_dir)?;
    let video_files: Vec<_> = files.into_iter().filter(|f| f.file_type == "video").collect();

    if video_files.is_empty() {
        return Err("没有找到视频文件".to_string());
    }

    let mut results = Vec::new();

    for file in video_files {
        // 使用ffprobe获取详细视频信息
        let output = app
            .shell()
            .sidecar("ffprobe")
            .map_err(|e| format!("无法找到FFprobe: {}", e))?
            .args(&[
                "-v", "error",
                "-select_streams", "v:0",
                "-show_entries", "stream=codec_name,profile,level,width,height,r_frame_rate,bit_rate",
                "-of", "json",
                &file.path,
            ])
            .output()
            .await
            .map_err(|e| format!("无法执行FFprobe: {}", e))?;

        if !output.status.success() {
            continue;
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_default();

        if let Some(streams) = json.get("streams").and_then(|s| s.as_array()) {
            if let Some(stream) = streams.first() {
                let codec = stream.get("codec_name").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
                let profile = stream.get("profile").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
                let level = stream.get("level").and_then(|v| v.as_i64()).unwrap_or(0);
                let level_str = if level > 0 { format!("{:.1}", level as f64 / 10.0) } else { "unknown".to_string() };
                let width = stream.get("width").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                let height = stream.get("height").and_then(|v| v.as_u64()).unwrap_or(0) as u32;

                // 解析帧率 (格式如 "30/1" 或 "30000/1001")
                let framerate_str = stream.get("r_frame_rate").and_then(|v| v.as_str()).unwrap_or("0/1");
                let framerate: f64 = {
                    let parts: Vec<&str> = framerate_str.split('/').collect();
                    if parts.len() == 2 {
                        let num: f64 = parts[0].parse().unwrap_or(0.0);
                        let den: f64 = parts[1].parse().unwrap_or(1.0);
                        if den > 0.0 { num / den } else { 0.0 }
                    } else {
                        framerate_str.parse().unwrap_or(0.0)
                    }
                };

                let bitrate = stream.get("bit_rate").and_then(|v| v.as_str()).and_then(|s| s.parse().ok()).unwrap_or(0u64);

                let video_info = VideoInfo {
                    path: file.path.clone(),
                    name: file.name.clone(),
                    codec,
                    profile,
                    level: level_str,
                    width,
                    height,
                    framerate,
                    bitrate,
                };

                let devices = check_device_compatibility(&video_info);

                // 提取缩略图
                let thumbnail = {
                    let temp_dir = std::env::temp_dir();
                    let thumb_path = temp_dir.join(format!("thumb_{}.jpg", generate_timestamp()));
                    let thumb_path_str = thumb_path.to_string_lossy().to_string();

                    // 使用ffmpeg提取缩略图 (缩小尺寸以节省空间)
                    let thumb_output = app
                        .shell()
                        .sidecar("ffmpeg")
                        .ok()
                        .map(|cmd| cmd.args(&[
                            "-i", &file.path,
                            "-vframes", "1",
                            "-vf", "scale=120:-1",
                            "-q:v", "5",
                            "-y",
                            &thumb_path_str,
                        ]));

                    let mut thumb_base64 = String::new();
                    if let Some(cmd) = thumb_output {
                        if let Ok(output) = cmd.output().await {
                            if output.status.success() {
                                if let Ok(image_data) = std::fs::read(&thumb_path) {
                                    use base64::{Engine as _, engine::general_purpose::STANDARD};
                                    thumb_base64 = format!("data:image/jpeg;base64,{}", STANDARD.encode(&image_data));
                                }
                                let _ = std::fs::remove_file(&thumb_path);
                            }
                        }
                    }
                    thumb_base64
                };

                results.push(VideoCompatibilityResult {
                    path: file.path,
                    name: file.name,
                    video_info,
                    devices,
                    thumbnail,
                });
            }
        }
    }

    Ok(results)
}

// 检查图片兼容性
fn check_image_compatibility(info: &ImageInfo) -> (bool, Vec<ImageCompatibilityIssue>) {
    let mut issues = Vec::new();
    let pix_fmt = info.pix_fmt.to_lowercase();
    let color_space = info.color_space.to_lowercase();

    // 检查像素格式兼容性
    // gbrap, gbrp 等非标准格式可能导致色彩问题
    let problematic_pix_fmts = ["gbrap", "gbrp", "gbrap16", "gbrp16"];
    if problematic_pix_fmts.iter().any(|fmt| pix_fmt.contains(fmt)) {
        issues.push(ImageCompatibilityIssue {
            issue_type: "pix_fmt".to_string(),
            severity: "error".to_string(),
            description: format!(
                "像素格式 {} 可能导致压缩后色彩失真（红色变紫色）",
                info.pix_fmt
            ),
        });
    }

    // 检查色彩空间兼容性
    // bt470bg 等非标准色彩空间可能导致色彩转换问题
    let problematic_color_spaces = ["bt470bg", "bt470m", "smpte170m", "smpte240m"];
    if problematic_color_spaces.iter().any(|cs| color_space.contains(cs)) {
        issues.push(ImageCompatibilityIssue {
            issue_type: "color_space".to_string(),
            severity: "warning".to_string(),
            description: format!(
                "色彩空间 {} 可能导致色彩转换偏差",
                info.color_space
            ),
        });
    }

    // 检查 gbrap + bt470bg 组合（已知会导致严重色彩问题）
    if pix_fmt.contains("gbrap") && color_space.contains("bt470bg") {
        // 移除之前的警告，添加更严重的错误
        issues.retain(|i| i.issue_type != "pix_fmt" && i.issue_type != "color_space");
        issues.push(ImageCompatibilityIssue {
            issue_type: "pix_fmt_colorspace".to_string(),
            severity: "error".to_string(),
            description: format!(
                "像素格式 {} + 色彩空间 {} 组合会导致严重色彩失真，建议使用其他工具处理",
                info.pix_fmt, info.color_space
            ),
        });
    }

    // 检查分辨率（超大图片可能处理缓慢）
    if info.width > 8000 || info.height > 8000 {
        issues.push(ImageCompatibilityIssue {
            issue_type: "resolution".to_string(),
            severity: "warning".to_string(),
            description: format!(
                "分辨率 {}x{} 过大，处理可能较慢",
                info.width, info.height
            ),
        });
    }

    // 检查是否有 alpha 通道（压缩为 JPEG 时会丢失）
    if pix_fmt.contains("a") && (pix_fmt.contains("rgba") || pix_fmt.contains("gbrap") || pix_fmt.contains("yuva")) {
        issues.push(ImageCompatibilityIssue {
            issue_type: "alpha".to_string(),
            severity: "info".to_string(),
            description: "图片包含透明通道，压缩为 JPEG 时会丢失透明度".to_string(),
        });
    }

    let compatible = !issues.iter().any(|i| i.severity == "error");
    (compatible, issues)
}

#[tauri::command]
async fn detect_image_compatibility(app: AppHandle, input_dir: String) -> Result<Vec<ImageCompatibilityResult>, String> {
    let files = scan_input_files(input_dir)?;
    let image_files: Vec<_> = files.into_iter().filter(|f| f.file_type == "image").collect();

    if image_files.is_empty() {
        return Err("没有找到图片文件".to_string());
    }

    let mut results = Vec::new();

    for file in image_files {
        // 使用 ffprobe 获取详细图片信息
        let output = app
            .shell()
            .sidecar("ffprobe")
            .map_err(|e| format!("无法找到FFprobe: {}", e))?
            .args(&[
                "-v", "error",
                "-select_streams", "v:0",
                "-show_entries", "stream=codec_name,pix_fmt,color_space,color_range,width,height",
                "-of", "json",
                &file.path,
            ])
            .output()
            .await
            .map_err(|e| format!("无法执行FFprobe: {}", e))?;

        if !output.status.success() {
            continue;
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_default();

        if let Some(streams) = json.get("streams").and_then(|s| s.as_array()) {
            if let Some(stream) = streams.first() {
                let codec = stream.get("codec_name").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
                let pix_fmt = stream.get("pix_fmt").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
                let color_space = stream.get("color_space").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
                let color_range = stream.get("color_range").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
                let width = stream.get("width").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                let height = stream.get("height").and_then(|v| v.as_u64()).unwrap_or(0) as u32;

                let image_info = ImageInfo {
                    path: file.path.clone(),
                    name: file.name.clone(),
                    codec,
                    pix_fmt,
                    color_space,
                    color_range,
                    width,
                    height,
                };

                let (compatible, issues) = check_image_compatibility(&image_info);

                // 生成缩略图
                let thumbnail = {
                    let temp_dir = std::env::temp_dir();
                    let thumb_path = temp_dir.join(format!("thumb_img_{}.jpg", generate_timestamp()));
                    let thumb_path_str = thumb_path.to_string_lossy().to_string();

                    let thumb_output = app
                        .shell()
                        .sidecar("ffmpeg")
                        .ok()
                        .map(|cmd| cmd.args(&[
                            "-i", &file.path,
                            "-vf", "scale=120:-1",
                            "-q:v", "5",
                            "-y",
                            &thumb_path_str,
                        ]));

                    let mut thumb_base64 = String::new();
                    if let Some(cmd) = thumb_output {
                        if let Ok(output) = cmd.output().await {
                            if output.status.success() {
                                if let Ok(image_data) = std::fs::read(&thumb_path) {
                                    use base64::{Engine as _, engine::general_purpose::STANDARD};
                                    thumb_base64 = format!("data:image/jpeg;base64,{}", STANDARD.encode(&image_data));
                                }
                                let _ = std::fs::remove_file(&thumb_path);
                            }
                        }
                    }
                    thumb_base64
                };

                results.push(ImageCompatibilityResult {
                    path: file.path,
                    name: file.name,
                    image_info,
                    compatible,
                    issues,
                    thumbnail,
                });
            }
        }
    }

    Ok(results)
}

async fn process_image(
    app: &AppHandle,
    input_path: &str,
    output_path: &str,
    options: &ProcessOptions,
) -> Result<(), String> {
    // 使用 ImageMagick 处理图片，避免 FFmpeg 的颜色转换问题
    let mut args: Vec<String> = vec![input_path.to_string()];

    // 旋转
    if options.rotate {
        args.push("-rotate".to_string());
        match options.rotation_degrees {
            90 | -270 => args.push("90".to_string()),
            180 | -180 => args.push("180".to_string()),
            270 | -90 => args.push("270".to_string()),
            _ => {}
        }
    }

    // 调整分辨率（独立选项）
    if options.reduce_resolution && options.target_width > 0 && options.target_height > 0 {
        args.push("-resize".to_string());
        args.push(format!("{}x{}!", options.target_width, options.target_height));
    }

    // 压缩时降低分辨率
    if options.compress && options.compress_resize && options.compress_width > 0 && options.compress_height > 0 {
        args.push("-resize".to_string());
        args.push(format!("{}x{}!", options.compress_width, options.compress_height));
    }

    // 质量设置（仅压缩时）
    if options.compress {
        args.push("-quality".to_string());
        args.push(options.compress_quality.to_string());
    }

    // 输出文件
    args.push(output_path.to_string());

    // 转换为 &str 引用
    let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

    // 使用 sidecar 调用 ImageMagick
    let output = app
        .shell()
        .sidecar("magick")
        .map_err(|e| format!("无法找到ImageMagick: {}", e))?
        .args(&args_ref)
        .output()
        .await
        .map_err(|e| format!("无法执行ImageMagick: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("ImageMagick处理失败: {}", stderr));
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

    // 裁剪（先裁剪，再缩放）
    if options.crop && options.crop_width > 0 && options.crop_height > 0 {
        video_filters.push(format!(
            "crop={}:{}:{}:{}",
            options.crop_width, options.crop_height, options.crop_x, options.crop_y
        ));
    }

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

    // 调整帧率（降帧/升帧）
    if options.change_framerate && options.target_framerate > 0.0 {
        video_filters.push(format!("fps={}", options.target_framerate));
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

    // Level 等级
    let level: String;
    if options.reduce_level && !options.target_level.is_empty() {
        level = options.target_level.clone();
        args.push("-level:v");
        args.push(&level);
    }

    // H.265 转 H.264
    if options.convert_h265_to_h264 {
        args.push("-c:v");
        args.push("libx264");
    }

    // 压缩（使用CRF）
    let crf_str: String;
    if options.compress {
        let crf = ((100 - options.compress_quality) as f32 * 0.51) as u32;
        crf_str = crf.to_string();
        args.push("-crf");
        args.push(&crf_str);
    }

    // 视频静音（去除音频）
    if options.mute {
        args.push("-an");
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
            get_video_dimensions,
            extract_video_frame,
            detect_video_compatibility,
            detect_image_compatibility,
            process_files,
            open_folder,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

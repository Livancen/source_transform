use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use tauri::{AppHandle, Manager};
use tauri_plugin_shell::ShellExt;

const THUMB_SIZE: &str = "96x96^";
const THUMB_EXTENT: &str = "96x96";
const THUMB_QUALITY: &str = "70";

fn cache_root(app: &AppHandle) -> Result<PathBuf, String> {
    let app_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let dir = app_dir.join("thumb_cache");
    fs::create_dir_all(&dir).map_err(|e| format!("创建缩略图缓存目录失败: {}", e))?;
    Ok(dir)
}

fn file_identity(path: &Path) -> Result<(u64, u64), String> {
    let meta = fs::metadata(path).map_err(|e| format!("读取文件信息失败: {}", e))?;
    let size = meta.len();
    let mtime = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0);
    Ok((size, mtime))
}

fn cache_key(path: &str, size: u64, mtime: u64) -> String {
    let mut hasher = DefaultHasher::new();
    path.hash(&mut hasher);
    size.hash(&mut hasher);
    mtime.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

fn cache_file(root: &Path, key: &str) -> PathBuf {
    root.join(format!("{}.jpg", key))
}

async fn generate_image_thumb(app: &AppHandle, src: &str, dest: &str) -> Result<(), String> {
    let output = app
        .shell()
        .sidecar("magick")
        .map_err(|e| format!("无法找到ImageMagick: {}", e))?
        .args(&[
            src,
            "-thumbnail",
            THUMB_SIZE,
            "-gravity",
            "center",
            "-extent",
            THUMB_EXTENT,
            "-quality",
            THUMB_QUALITY,
            dest,
        ])
        .output()
        .await
        .map_err(|e| format!("无法执行ImageMagick: {}", e))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

async fn generate_video_thumb(app: &AppHandle, src: &str, dest: &str) -> Result<(), String> {
    // 一步出 96x96 jpg，避免先抽帧再二次处理
    let vf = format!(
        "thumbnail,scale=96:96:force_original_aspect_ratio=increase,crop=96:96"
    );
    let output = app
        .shell()
        .sidecar("ffmpeg")
        .map_err(|e| format!("无法找到FFmpeg: {}", e))?
        .args(&[
            "-hide_banner",
            "-loglevel",
            "error",
            "-i",
            src,
            "-vf",
            &vf,
            "-frames:v",
            "1",
            "-q:v",
            "5",
            "-y",
            dest,
        ])
        .output()
        .await
        .map_err(|e| format!("无法执行FFmpeg: {}", e))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

/// 获取缩略图磁盘路径；命中缓存则直接返回，否则生成后写入 app data
pub async fn get_cached_thumbnail_path(
    app: &AppHandle,
    path: &str,
    file_type: &str,
) -> Result<PathBuf, String> {
    let src = PathBuf::from(path);
    if !src.is_file() {
        return Err("文件不存在".to_string());
    }

    let (size, mtime) = file_identity(&src)?;
    let root = cache_root(app)?;
    let key = cache_key(path, size, mtime);
    let dest = cache_file(&root, &key);

    if dest.is_file() {
        if let Ok(meta) = fs::metadata(&dest) {
            if meta.len() > 0 {
                return Ok(dest);
            }
        }
        let _ = fs::remove_file(&dest);
    }

    let dest_str = dest.to_string_lossy().to_string();
    let result = if file_type == "video" {
        generate_video_thumb(app, path, &dest_str).await
    } else {
        generate_image_thumb(app, path, &dest_str).await
    };

    match result {
        Ok(()) => {
            if dest.is_file() {
                Ok(dest)
            } else {
                Err("缩略图生成失败".to_string())
            }
        }
        Err(e) => {
            let _ = fs::remove_file(&dest);
            Err(e)
        }
    }
}

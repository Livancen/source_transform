use serde::{Deserialize, Serialize};

// 支持的文件扩展名
pub const IMAGE_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "gif", "bmp", "webp", "tiff"];
pub const VIDEO_EXTENSIONS: &[&str] = &["mp4", "avi", "mov", "mkv", "wmv", "flv", "webm", "m4v"];

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
    pub target_profile: String, // e.g., "baseline", "main", "high"
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
pub struct VideoMergeSlot {
    pub path: String,
    pub name: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VideoMergeOptions {
    pub layout: String, // "vertical" or "horizontal"
    pub slots: [VideoMergeSlot; 2],
    pub output_width: Option<u32>,
    pub output_height: Option<u32>,
    pub output_path: String,
}

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
    pub convert_h264_to_h265: bool,
    pub convert_format: bool,
    pub target_format: String, // e.g., "mp4", "avi", "mkv" / "jpg", "png"
    pub rotate: bool,
    pub rotation_degrees: i32, // 90, 180, 270, -90
    pub mute: bool,            // 视频静音（去除音频）
    pub change_framerate: bool, // 调整帧率
    pub target_framerate: f32,  // 目标帧率
    #[serde(default)]
    pub watermark: bool,
    /// "text" | "image"
    #[serde(default = "default_watermark_type")]
    pub watermark_type: String,
    #[serde(default = "default_watermark_text")]
    pub watermark_text: String,
    #[serde(default = "default_watermark_font_size")]
    pub watermark_font_size: u32,
    #[serde(default = "default_watermark_font_color")]
    pub watermark_font_color: String,
    #[serde(default = "default_watermark_opacity")]
    pub watermark_font_opacity: u32,
    #[serde(default = "default_true")]
    pub watermark_stroke: bool,
    #[serde(default = "default_stroke_width")]
    pub watermark_stroke_width: u32,
    #[serde(default = "default_stroke_color")]
    pub watermark_stroke_color: String,
    #[serde(default)]
    pub watermark_image_path: String,
    #[serde(default = "default_image_scale")]
    pub watermark_image_scale: u32,
    #[serde(default = "default_watermark_opacity")]
    pub watermark_image_opacity: u32,
    /// tl/tc/tr/ml/mc/mr/bl/bc/br
    #[serde(default = "default_watermark_position")]
    pub watermark_position: String,
    #[serde(default = "default_margin")]
    pub watermark_margin_x: u32,
    #[serde(default = "default_margin")]
    pub watermark_margin_y: u32,
    /// 旋转角度（度），顺时针为正
    #[serde(default)]
    pub watermark_rotation: f32,
    #[serde(default)]
    pub watermark_tile: bool,
    #[serde(default = "default_tile_gap")]
    pub watermark_tile_gap_x: u32,
    #[serde(default = "default_tile_gap")]
    pub watermark_tile_gap_y: u32,
}

fn default_watermark_type() -> String {
    "text".to_string()
}
fn default_watermark_text() -> String {
    "水印".to_string()
}
fn default_watermark_font_size() -> u32 {
    36
}
fn default_watermark_font_color() -> String {
    "#FFFFFF".to_string()
}
fn default_watermark_opacity() -> u32 {
    60
}
fn default_true() -> bool {
    true
}
fn default_stroke_width() -> u32 {
    2
}
fn default_stroke_color() -> String {
    "#000000".to_string()
}
fn default_image_scale() -> u32 {
    15
}
fn default_watermark_position() -> String {
    "br".to_string()
}
fn default_margin() -> u32 {
    16
}
fn default_tile_gap() -> u32 {
    80
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NamingOptions {
    pub use_original_name: bool,
    pub use_timestamp: bool,
    pub use_datetime: bool,
    pub custom_text: String,
}

impl Default for NamingOptions {
    fn default() -> Self {
        Self {
            use_original_name: true,
            use_timestamp: false,
            use_datetime: false,
            custom_text: String::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProcessProgress {
    pub current: usize,
    pub total: usize,
    pub current_file: String,
    pub status: String,
    /// 0～100 整体进度（含当前文件内 FFmpeg 进度）
    #[serde(default)]
    pub percent: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileInfo {
    pub path: String,
    pub name: String,
    pub file_type: String, // "image" or "video"
    /// 文件大小（字节）
    #[serde(default)]
    pub size_bytes: u64,
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
    /// "video" or "image"
    #[serde(default = "default_media_kind")]
    pub media_kind: String,
    /// 输出帧率 30 / 60；不足时 fps 滤镜补帧
    #[serde(default)]
    pub output_fps: Option<u32>,
    #[serde(default)]
    pub set_level: bool,
    #[serde(default)]
    pub video_level: Option<String>,
    #[serde(default)]
    pub video_profile: Option<String>,
}

fn default_media_kind() -> String {
    "video".to_string()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CustomCropOptions {
    pub input_path: String,
    pub output_dir: String,
    pub crop_x: u32,
    pub crop_y: u32,
    pub crop_width: u32,
    pub crop_height: u32,
    pub naming: NamingOptions,
}

fn default_blur() -> bool {
    false
}

fn default_blur_sigma() -> f64 {
    20.0
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JoinItem {
    pub id: String,
    pub path: String,
    pub name: String,
    /// "video" or "image"
    #[serde(default = "default_media_kind")]
    pub media_kind: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub z: u32,
    /// "cover" | "contain" | "fill"
    pub fit: String,
    /// 对该图层矩形区域做高斯模糊
    #[serde(default = "default_blur")]
    pub blur: bool,
    /// gblur sigma，约 1–50
    #[serde(default = "default_blur_sigma")]
    pub blur_sigma: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JoinOptions {
    /// 输出类型 "video" or "image"（含任意视频则为 video）
    pub media_kind: String,
    pub canvas_width: u32,
    pub canvas_height: u32,
    /// "#000000" | "#ffffff" | "transparent"
    pub background: String,
    pub items: Vec<JoinItem>,
    pub output_path: String,
    /// 输出帧率 30 / 60；不足时补帧
    #[serde(default)]
    pub output_fps: Option<u32>,
    #[serde(default)]
    pub set_level: bool,
    #[serde(default)]
    pub video_level: Option<String>,
    #[serde(default)]
    pub video_profile: Option<String>,
}

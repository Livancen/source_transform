use std::path::{Path, PathBuf};

use crate::types::ProcessOptions;

#[derive(Clone, Copy)]
pub struct GravityOffset {
    pub gravity: &'static str,
}

pub fn clamp_opacity(v: u32) -> u32 {
    v.min(100)
}

/// 规范化到 [-180, 180]
pub fn normalize_rotation_deg(deg: f32) -> f32 {
    if !deg.is_finite() {
        return 0.0;
    }
    let mut v = deg;
    while v > 180.0 {
        v -= 360.0;
    }
    while v < -180.0 {
        v += 360.0;
    }
    v
}

pub fn parse_hex_color(color: &str) -> (u8, u8, u8) {
    let s = color.trim().trim_start_matches('#');
    if s.len() >= 6 {
        let r = u8::from_str_radix(&s[0..2], 16).unwrap_or(255);
        let g = u8::from_str_radix(&s[2..4], 16).unwrap_or(255);
        let b = u8::from_str_radix(&s[4..6], 16).unwrap_or(255);
        (r, g, b)
    } else {
        (255, 255, 255)
    }
}

pub fn rgba_css(color: &str, opacity_pct: u32) -> String {
    let (r, g, b) = parse_hex_color(color);
    let a = clamp_opacity(opacity_pct) as f32 / 100.0;
    format!("rgba({},{},{},{:.3})", r, g, b, a)
}

pub fn gravity_for_position(pos: &str) -> GravityOffset {
    match pos {
        "tl" => GravityOffset {
            gravity: "NorthWest",
        },
        "tc" => GravityOffset {
            gravity: "North",
        },
        "tr" => GravityOffset {
            gravity: "NorthEast",
        },
        "ml" => GravityOffset {
            gravity: "West",
        },
        "mc" => GravityOffset {
            gravity: "Center",
        },
        "mr" => GravityOffset {
            gravity: "East",
        },
        "bl" => GravityOffset {
            gravity: "SouthWest",
        },
        "bc" => GravityOffset {
            gravity: "South",
        },
        _ => GravityOffset {
            gravity: "SouthEast",
        },
    }
}

/// FFmpeg overlay x/y 表达式（相对主画面）
pub fn ffmpeg_overlay_xy(pos: &str, margin_x: u32, margin_y: u32) -> (String, String) {
    let mx = margin_x;
    let my = margin_y;
    match pos {
        "tl" => (format!("{}", mx), format!("{}", my)),
        "tc" => (format!("(W-w)/2"), format!("{}", my)),
        "tr" => (format!("W-w-{}", mx), format!("{}", my)),
        "ml" => (format!("{}", mx), format!("(H-h)/2")),
        "mc" => ("(W-w)/2".to_string(), "(H-h)/2".to_string()),
        "mr" => (format!("W-w-{}", mx), format!("(H-h)/2")),
        "bl" => (format!("{}", mx), format!("H-h-{}", my)),
        "bc" => (format!("(W-w)/2"), format!("H-h-{}", my)),
        _ => (format!("W-w-{}", mx), format!("H-h-{}", my)),
    }
}

pub fn default_font_path() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        let candidates = [
            r"C:\Windows\Fonts\simhei.ttf",
            r"C:\Windows\Fonts\simkai.ttf",
            r"C:\Windows\Fonts\msyh.ttc",
            r"C:\Windows\Fonts\msyhbd.ttc",
            r"C:\Windows\Fonts\simsun.ttc",
            r"C:\Windows\Fonts\arial.ttf",
        ];
        for c in candidates {
            let p = PathBuf::from(c);
            if p.exists() {
                return Some(p);
            }
        }
        None
    }
    #[cfg(target_os = "macos")]
    {
        // ImageMagick annotate 对路径空格较敏感，优先无空格字体
        let candidates = [
            "/System/Library/Fonts/PingFang.ttc",
            "/System/Library/Fonts/Helvetica.ttc",
            "/System/Library/Fonts/Supplemental/Arial.ttf",
            "/Library/Fonts/Arial Unicode.ttf",
            "/System/Library/Fonts/Supplemental/Arial Unicode.ttf",
            "/System/Library/Fonts/Hiragino Sans GB.ttc",
            "/System/Library/Fonts/STHeiti Light.ttc",
            "/System/Library/Fonts/STHeiti Medium.ttc",
        ];
        for c in candidates {
            let p = PathBuf::from(c);
            if p.exists() {
                return Some(p);
            }
        }
        None
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let candidates = [
            "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
            "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
        ];
        for c in candidates {
            let p = PathBuf::from(c);
            if p.exists() {
                return Some(p);
            }
        }
        None
    }
}

pub fn watermark_enabled(options: &ProcessOptions) -> bool {
    if !options.watermark {
        return false;
    }
    match options.watermark_type.as_str() {
        "image" => {
            let p = options.watermark_image_path.trim();
            !p.is_empty() && Path::new(p).is_file()
        }
        _ => !options.watermark_text.trim().is_empty(),
    }
}

pub fn validate_watermark(options: &ProcessOptions) -> Result<(), String> {
    if !options.watermark {
        return Ok(());
    }
    match options.watermark_type.as_str() {
        "image" => {
            let p = options.watermark_image_path.trim();
            if p.is_empty() {
                return Err("请先选择水印图片".to_string());
            }
            if !Path::new(p).is_file() {
                return Err(format!("水印图片不存在: {}", p));
            }
            Ok(())
        }
        _ => {
            if options.watermark_text.trim().is_empty() {
                return Err("水印文字不能为空".to_string());
            }
            Ok(())
        }
    }
}

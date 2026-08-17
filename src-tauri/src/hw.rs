use std::sync::Mutex;
use tauri::AppHandle;
use tauri_plugin_shell::ShellExt;

/// 用户偏好：auto 优先硬编；off 强制软编；其余为指定编码器名
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HwAccelMode {
    Auto,
    Off,
    Prefer(String),
}

impl HwAccelMode {
    pub fn from_config(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "" | "auto" => Self::Auto,
            "off" | "none" | "software" | "cpu" => Self::Off,
            other => Self::Prefer(other.to_string()),
        }
    }

    pub fn as_config(&self) -> String {
        match self {
            Self::Auto => "auto".to_string(),
            Self::Off => "off".to_string(),
            Self::Prefer(name) => name.clone(),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HwEncoderInfo {
    pub id: String,
    pub label: String,
    pub codec: String, // "h264" | "hevc"
    pub available: bool,
}

#[derive(Debug, Clone)]
struct DetectedEncoders {
    h264: Vec<String>,
    hevc: Vec<String>,
}

static DETECTED: Mutex<Option<DetectedEncoders>> = Mutex::new(None);
static PREFERENCE: Mutex<HwAccelMode> = Mutex::new(HwAccelMode::Auto);

/// 探测优先级（同平台内越靠前越优先）
const H264_CANDIDATES: &[&str] = &[
    "h264_nvenc",
    "h264_qsv",
    "h264_amf",
    "h264_videotoolbox",
    "h264_mf",
];
const HEVC_CANDIDATES: &[&str] = &[
    "hevc_nvenc",
    "hevc_qsv",
    "hevc_amf",
    "hevc_videotoolbox",
    "hevc_mf",
];

fn encoder_label(id: &str) -> String {
    match id {
        "h264_nvenc" | "hevc_nvenc" => format!("{} (NVIDIA NVENC)", id),
        "h264_qsv" | "hevc_qsv" => format!("{} (Intel QSV)", id),
        "h264_amf" | "hevc_amf" => format!("{} (AMD AMF)", id),
        "h264_videotoolbox" | "hevc_videotoolbox" => format!("{} (VideoToolbox)", id),
        "h264_mf" | "hevc_mf" => format!("{} (MediaFoundation)", id),
        "libx264" => "libx264 (CPU)".to_string(),
        "libx265" => "libx265 (CPU)".to_string(),
        other => other.to_string(),
    }
}

fn codec_of(id: &str) -> &'static str {
    if id.contains("265") || id.contains("hevc") {
        "hevc"
    } else {
        "h264"
    }
}

async fn list_ffmpeg_encoders(app: &AppHandle) -> Result<String, String> {
    let output = app
        .shell()
        .sidecar("ffmpeg")
        .map_err(|e| format!("无法找到FFmpeg: {}", e))?
        .args(["-hide_banner", "-encoders"])
        .output()
        .await
        .map_err(|e| format!("无法执行FFmpeg: {}", e))?;
    // ffmpeg 把列表打到 stdout；部分构建也可能混在 stderr
    let mut text = String::from_utf8_lossy(&output.stdout).to_string();
    text.push_str(&String::from_utf8_lossy(&output.stderr));
    Ok(text)
}

fn parse_available(list_text: &str, candidates: &[&str]) -> Vec<String> {
    let mut found = Vec::new();
    for name in candidates {
        // 行形如： V..... h264_nvenc           NVIDIA NVENC H.264 encoder
        let needle = format!(" {}", name);
        if list_text.lines().any(|line| {
            let t = line.trim_start();
            t.contains(&needle) || t.ends_with(name) || t.split_whitespace().any(|w| w == *name)
        }) {
            found.push((*name).to_string());
        }
    }
    found
}

/// 探测并缓存可用硬编（进程内只跑一次，除非 force）
pub async fn ensure_detected(app: &AppHandle, force: bool) -> Result<(), String> {
    {
        let guard = DETECTED.lock().map_err(|e| e.to_string())?;
        if guard.is_some() && !force {
            return Ok(());
        }
    }
    let text = list_ffmpeg_encoders(app).await?;
    let detected = DetectedEncoders {
        h264: parse_available(&text, H264_CANDIDATES),
        hevc: parse_available(&text, HEVC_CANDIDATES),
    };
    let mut guard = DETECTED.lock().map_err(|e| e.to_string())?;
    *guard = Some(detected);
    Ok(())
}

pub fn set_preference(mode: HwAccelMode) {
    if let Ok(mut g) = PREFERENCE.lock() {
        *g = mode;
    }
}

pub fn get_preference() -> HwAccelMode {
    PREFERENCE
        .lock()
        .map(|g| g.clone())
        .unwrap_or(HwAccelMode::Auto)
}

pub fn list_encoder_infos() -> Vec<HwEncoderInfo> {
    let detected = DETECTED.lock().ok().and_then(|g| g.clone());
    let mut out = Vec::new();
    out.push(HwEncoderInfo {
        id: "auto".to_string(),
        label: "自动（优先硬件）".to_string(),
        codec: "any".to_string(),
        available: true,
    });
    out.push(HwEncoderInfo {
        id: "off".to_string(),
        label: "关闭（仅 CPU）".to_string(),
        codec: "any".to_string(),
        available: true,
    });

    let h264_avail = detected
        .as_ref()
        .map(|d| d.h264.clone())
        .unwrap_or_default();
    let hevc_avail = detected
        .as_ref()
        .map(|d| d.hevc.clone())
        .unwrap_or_default();

    for id in H264_CANDIDATES {
        out.push(HwEncoderInfo {
            id: (*id).to_string(),
            label: encoder_label(id),
            codec: "h264".to_string(),
            available: h264_avail.iter().any(|x| x == *id),
        });
    }
    for id in HEVC_CANDIDATES {
        out.push(HwEncoderInfo {
            id: (*id).to_string(),
            label: encoder_label(id),
            codec: "hevc".to_string(),
            available: hevc_avail.iter().any(|x| x == *id),
        });
    }
    out.push(HwEncoderInfo {
        id: "libx264".to_string(),
        label: encoder_label("libx264"),
        codec: "h264".to_string(),
        available: true,
    });
    out.push(HwEncoderInfo {
        id: "libx265".to_string(),
        label: encoder_label("libx265"),
        codec: "hevc".to_string(),
        available: true,
    });
    out
}

/// 解析实际使用的编码器：codec = "h264" | "hevc"
pub fn resolve_encoder(codec: &str) -> String {
    let pref = get_preference();
    let soft = if codec == "hevc" { "libx265" } else { "libx264" };

    match pref {
        HwAccelMode::Off => soft.to_string(),
        HwAccelMode::Prefer(name) => {
            if name == "libx264" || name == "libx265" {
                return if codec == "hevc" {
                    "libx265".to_string()
                } else {
                    "libx264".to_string()
                };
            }
            // 指定硬编但 codec 不匹配时回退 soft
            if codec_of(&name) != codec {
                return soft.to_string();
            }
            let ok = DETECTED
                .lock()
                .ok()
                .and_then(|g| g.clone())
                .map(|d| {
                    if codec == "hevc" {
                        d.hevc.iter().any(|x| x == &name)
                    } else {
                        d.h264.iter().any(|x| x == &name)
                    }
                })
                .unwrap_or(false);
            if ok {
                name
            } else {
                soft.to_string()
            }
        }
        HwAccelMode::Auto => {
            let detected = DETECTED.lock().ok().and_then(|g| g.clone());
            if let Some(d) = detected {
                let list = if codec == "hevc" { &d.hevc } else { &d.h264 };
                if let Some(first) = list.first() {
                    return first.clone();
                }
            }
            soft.to_string()
        }
    }
}

/// 是否为硬件编码器
pub fn is_hardware_encoder(name: &str) -> bool {
    !matches!(name, "libx264" | "libx265" | "mpeg4" | "libvpx" | "libvpx-vp9")
        && (name.contains("nvenc")
            || name.contains("qsv")
            || name.contains("amf")
            || name.contains("videotoolbox")
            || name.contains("_mf"))
}

/// 将编码相关参数追加到 args（含 -c:v 与质量/码率/profile）
pub fn append_video_encode_args(
    args: &mut Vec<String>,
    codec: &str, // "h264" | "hevc"
    quality: Option<u32>, // 1-100，越高越好；None 用默认
    bitrate: Option<&str>,
    profile: Option<&str>,
    level: Option<&str>,
    tag_hvc1: bool,
) {
    let encoder = resolve_encoder(codec);
    args.push("-c:v".to_string());
    args.push(encoder.clone());

    if tag_hvc1 && (encoder.contains("hevc") || encoder == "libx265") {
        args.push("-tag:v".to_string());
        args.push("hvc1".to_string());
    }

    args.push("-pix_fmt".to_string());
    args.push("yuv420p".to_string());

    if let Some(br) = bitrate {
        if !br.is_empty() {
            args.push("-b:v".to_string());
            args.push(br.to_string());
        }
    } else if let Some(q) = quality {
        let q = q.clamp(1, 100);
        if is_hardware_encoder(&encoder) {
            // 硬编：CQ / quality 近似映射
            if encoder.contains("nvenc") {
                let cq = ((100 - q) as f32 * 0.35).round() as u32; // 0-35
                args.push("-rc".to_string());
                args.push("vbr".to_string());
                args.push("-cq".to_string());
                args.push(cq.min(51).to_string());
                args.push("-b:v".to_string());
                args.push("0".to_string());
            } else if encoder.contains("qsv") {
                let gq = ((100 - q) as f32 * 0.40 + 10.0).round() as u32;
                args.push("-global_quality".to_string());
                args.push(gq.min(51).to_string());
            } else if encoder.contains("amf") {
                let qp = ((100 - q) as f32 * 0.40 + 10.0).round() as u32;
                args.push("-rc".to_string());
                args.push("cqp".to_string());
                args.push("-qp_i".to_string());
                args.push(qp.min(51).to_string());
                args.push("-qp_p".to_string());
                args.push(qp.min(51).to_string());
            } else if encoder.contains("videotoolbox") {
                // 0-100，越高越好
                args.push("-q:v".to_string());
                args.push(q.to_string());
            } else {
                let crf = ((100 - q) as f32 * 0.51) as u32;
                args.push("-crf".to_string());
                args.push(crf.to_string());
            }
        } else {
            let crf = ((100 - q) as f32 * 0.51) as u32;
            args.push("-crf".to_string());
            args.push(crf.to_string());
        }
    } else if is_hardware_encoder(&encoder) {
        // 默认中等质量
        if encoder.contains("nvenc") {
            args.extend([
                "-rc".into(),
                "vbr".into(),
                "-cq".into(),
                "23".into(),
                "-b:v".into(),
                "0".into(),
            ]);
        } else if encoder.contains("qsv") {
            args.extend(["-global_quality".into(), "23".into()]);
        } else if encoder.contains("amf") {
            args.extend([
                "-rc".into(),
                "cqp".into(),
                "-qp_i".into(),
                "23".into(),
                "-qp_p".into(),
                "23".into(),
            ]);
        } else if encoder.contains("videotoolbox") {
            args.extend(["-q:v".into(), "65".into()]);
        } else {
            args.extend(["-crf".into(), "23".into()]);
        }
    } else {
        args.extend(["-crf".into(), "23".into()]);
    }

    // profile / level：硬编支持度不一，尽量附加；失败时由调用方回退
    if let Some(p) = profile {
        if !p.is_empty() {
            args.push("-profile:v".to_string());
            args.push(p.to_lowercase());
        }
    }
    if let Some(l) = level {
        if !l.is_empty() {
            args.push("-level:v".to_string());
            args.push(l.to_string());
        }
    }
}

async fn run_ffmpeg_once(app: &AppHandle, args: &[String]) -> Result<(), String> {
    let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let output = app
        .shell()
        .sidecar("ffmpeg")
        .map_err(|e| format!("无法找到FFmpeg: {}", e))?
        .args(&refs)
        .output()
        .await
        .map_err(|e| format!("无法执行FFmpeg: {}", e))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

/// 执行 ffmpeg；硬编失败时自动回退 libx264/libx265 再试一次
pub async fn run_ffmpeg_with_fallback(app: &AppHandle, args: &[String]) -> Result<(), String> {
    let _ = ensure_detected(app, false).await;

    match run_ffmpeg_once(app, args).await {
        Ok(()) => Ok(()),
        Err(err) => {
            let has_hw = args.iter().any(|a| is_hardware_encoder(a));
            if !has_hw {
                return Err(format!("FFmpeg处理失败: {}", err));
            }
            let soft_args = replace_encoder_with_software(args);
            match run_ffmpeg_once(app, &soft_args).await {
                Ok(()) => Ok(()),
                Err(err2) => Err(format!(
                    "FFmpeg处理失败（硬编失败已回退软编）: {}\n硬编错误: {}",
                    err2, err
                )),
            }
        }
    }
}

fn replace_encoder_with_software(args: &[String]) -> Vec<String> {
    let mut out = args.to_vec();
    let mut i = 0;
    while i < out.len() {
        if out[i] == "-c:v" && i + 1 < out.len() {
            let enc = out[i + 1].clone();
            if is_hardware_encoder(&enc) {
                out[i + 1] = if codec_of(&enc) == "hevc" {
                    "libx265".to_string()
                } else {
                    "libx264".to_string()
                };
                // 去掉硬编专用参数，补 CRF
                strip_hw_only_params(&mut out);
                if !out.iter().any(|a| a == "-crf") {
                    // 在输出路径前插入
                    if let Some(pos) = out.iter().rposition(|a| a == "-y") {
                        out.insert(pos, "23".to_string());
                        out.insert(pos, "-crf".to_string());
                    } else {
                        out.push("-crf".to_string());
                        out.push("23".to_string());
                    }
                }
            }
            break;
        }
        i += 1;
    }
    out
}

fn strip_hw_only_params(args: &mut Vec<String>) {
    let keys = [
        "-rc",
        "-cq",
        "-global_quality",
        "-qp_i",
        "-qp_p",
        "-q:v",
        "-b:v",
    ];
    let mut i = 0;
    while i < args.len() {
        if keys.contains(&args[i].as_str()) {
            args.remove(i);
            if i < args.len() {
                args.remove(i);
            }
        } else {
            i += 1;
        }
    }
}

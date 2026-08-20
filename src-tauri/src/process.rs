use std::path::{Path, PathBuf};

use tauri::AppHandle;
use tauri_plugin_shell::ShellExt;

use crate::hw::{
    self, append_video_encode_args, run_ffmpeg_with_fallback_progress, ProgressCallback,
};
use crate::types::ProcessOptions;
use crate::watermark::{
    self, clamp_opacity, ffmpeg_overlay_xy, gravity_for_position, normalize_rotation_deg, rgba_css,
    validate_watermark, watermark_enabled,
};

pub async fn process_image(
    app: &AppHandle,
    input_path: &str,
    output_path: &str,
    options: &ProcessOptions,
) -> Result<(), String> {
    validate_watermark(options)?;

    let mut args: Vec<String> = vec![input_path.to_string()];

    if options.rotate {
        args.push("-rotate".to_string());
        match options.rotation_degrees {
            90 | -270 => args.push("90".to_string()),
            180 | -180 => args.push("180".to_string()),
            270 | -90 => args.push("270".to_string()),
            _ => {}
        }
    }

    if options.reduce_resolution && options.target_width > 0 && options.target_height > 0 {
        args.push("-resize".to_string());
        args.push(format!("{}x{}!", options.target_width, options.target_height));
    }

    if options.compress
        && options.compress_resize
        && options.compress_width > 0
        && options.compress_height > 0
    {
        args.push("-resize".to_string());
        args.push(format!("{}x{}!", options.compress_width, options.compress_height));
    }

    if watermark_enabled(options) {
        append_imagemagick_watermark(app, &mut args, options).await?;
    }

    if options.compress {
        args.push("-quality".to_string());
        args.push(options.compress_quality.to_string());
    }

    args.push(output_path.to_string());

    let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
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

async fn probe_image_size(app: &AppHandle, path: &str) -> Result<(u32, u32), String> {
    let output = app
        .shell()
        .sidecar("magick")
        .map_err(|e| format!("无法找到ImageMagick: {}", e))?
        .args(&["identify", "-format", "%w %h", path])
        .output()
        .await
        .map_err(|e| format!("无法执行ImageMagick: {}", e))?;
    if !output.status.success() {
        return Err("无法读取图片尺寸".to_string());
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parts: Vec<&str> = stdout.trim().split_whitespace().collect();
    if parts.len() >= 2 {
        let w = parts[0].parse().map_err(|_| "无法解析宽度")?;
        let h = parts[1].parse().map_err(|_| "无法解析高度")?;
        Ok((w, h))
    } else {
        Err("无法解析图片尺寸".to_string())
    }
}

async fn append_imagemagick_watermark(
    app: &AppHandle,
    args: &mut Vec<String>,
    options: &ProcessOptions,
) -> Result<(), String> {
    let (canvas_w, canvas_h) = estimate_output_size(app, args, options).await?;

    if options.watermark_type == "image" {
        let wm_path = options.watermark_image_path.trim();
        let scale = options.watermark_image_scale.clamp(1, 100);
        let opacity = clamp_opacity(options.watermark_image_opacity) as f64 / 100.0;
        let target_w = ((canvas_w as f64) * (scale as f64) / 100.0).round().max(1.0) as u32;

        if options.watermark_tile {
            let (stamp_w, stamp_h) = probe_image_size(app, wm_path).await.unwrap_or((target_w, target_w));
            let stamp_h = if stamp_w > 0 {
                ((stamp_h as f64) * (target_w as f64) / (stamp_w as f64)).round().max(1.0) as u32
            } else {
                target_w
            };
            let stamp_w = target_w;
            let gap_x = options.watermark_tile_gap_x;
            let gap_y = options.watermark_tile_gap_y;
            let step_x = (stamp_w + gap_x).max(1);
            let step_y = (stamp_h + gap_y).max(1);
            let mut y = options.watermark_margin_y;
            let mut rows = 0u32;
            while y < canvas_h && rows < 40 {
                let mut x = options.watermark_margin_x;
                let mut cols = 0u32;
                while x < canvas_w && cols < 40 {
                    push_magick_image_stamp(
                        args,
                        wm_path,
                        target_w,
                        opacity,
                        "NorthWest",
                        x,
                        y,
                        options.watermark_rotation,
                    );
                    x = x.saturating_add(step_x);
                    cols += 1;
                }
                y = y.saturating_add(step_y);
                rows += 1;
            }
        } else {
            let g = gravity_for_position(&options.watermark_position);
            push_magick_image_stamp(
                args,
                wm_path,
                target_w,
                opacity,
                g.gravity,
                options.watermark_margin_x,
                options.watermark_margin_y,
                options.watermark_rotation,
            );
        }
        return Ok(());
    }

    // 文字水印
    // ImageMagick 会把 Windows 路径中的 `\` 当转义，字体路径必须用 `/`
    let text = options.watermark_text.trim();
    let fill = rgba_css(
        &options.watermark_font_color,
        options.watermark_font_opacity,
    );
    let font_size = options.watermark_font_size.max(8);
    if let Some(font) = watermark::default_font_path() {
        // Windows 下 `\` 会被 ImageMagick 当转义吃掉，必须用正斜杠
        args.push("-font".to_string());
        args.push(magick_path(&font.to_string_lossy()));
    }
    args.push("-pointsize".to_string());
    args.push(font_size.to_string());

    if options.watermark_stroke && options.watermark_stroke_width > 0 {
        args.push("-stroke".to_string());
        args.push(rgba_css(
            &options.watermark_stroke_color,
            options.watermark_font_opacity,
        ));
        args.push("-strokewidth".to_string());
        args.push(options.watermark_stroke_width.to_string());
    } else {
        args.push("-stroke".to_string());
        args.push("none".to_string());
    }
    args.push("-fill".to_string());
    args.push(fill);

    // annotate 文本：转义 %；不要用反斜杠（IM 会当转义）
    let escaped = text.replace('%', "%%");
    // ImageMagick annotate：`{angle}x{angle}+x+y`，顺时针；角度必须 ≥0，否则被当成 CLI 选项
    let rot = magick_positive_degrees(options.watermark_rotation);

    if options.watermark_tile {
        let approx_w = ((font_size as f32)
            * options.watermark_text.chars().count().max(1) as f32
            * 0.9)
            .max(font_size as f32) as u32;
        let approx_h = font_size + 8;
        let step_x = (approx_w + options.watermark_tile_gap_x).max(1);
        let step_y = (approx_h + options.watermark_tile_gap_y).max(1);
        let mut y = options.watermark_margin_y;
        let mut rows = 0u32;
        while y < canvas_h && rows < 40 {
            let mut x = options.watermark_margin_x;
            let mut cols = 0u32;
            while x < canvas_w && cols < 40 {
                args.push("-gravity".to_string());
                args.push("NorthWest".to_string());
                args.push("-annotate".to_string());
                args.push(format!("{:.3}x{:.3}+{}+{}", rot, rot, x, y));
                args.push(escaped.clone());
                x = x.saturating_add(step_x);
                cols += 1;
            }
            y = y.saturating_add(step_y);
            rows += 1;
        }
    } else {
        let g = gravity_for_position(&options.watermark_position);
        args.push("-gravity".to_string());
        args.push(g.gravity.to_string());
        args.push("-annotate".to_string());
        args.push(format!(
            "{:.3}x{:.3}+{}+{}",
            rot, rot, options.watermark_margin_x, options.watermark_margin_y
        ));
        args.push(escaped);
    }

    Ok(())
}

/// ImageMagick 参数路径：统一为正斜杠，避免 `\` 被当成转义
fn magick_path(path: &str) -> String {
    path.replace('\\', "/")
}

/// ImageMagick CLI 会把以 `-` 开头的参数当成选项。
/// 将角度规范到 `[0, 360)`，使 annotate / rotate 参数永不带负号。
fn magick_positive_degrees(deg: f32) -> f32 {
    let mut v = normalize_rotation_deg(deg) % 360.0;
    if v < 0.0 {
        v += 360.0;
    }
    // -0.0 → 0
    if v.abs() < 0.001 {
        0.0
    } else {
        v
    }
}

fn push_magick_image_stamp(
    args: &mut Vec<String>,
    wm_path: &str,
    target_w: u32,
    opacity: f64,
    gravity: &str,
    margin_x: u32,
    margin_y: u32,
    rotation_deg: f32,
) {
    let rot = magick_positive_degrees(rotation_deg);
    args.push("(".to_string());
    args.push(magick_path(wm_path));
    args.push("-resize".to_string());
    args.push(format!("{}x", target_w));
    if rot > 0.01 {
        // 透明背景旋转，避免黑边；角度用 [0,360) 避免负号被当 CLI 选项
        args.push("-background".to_string());
        args.push("none".to_string());
        args.push("-rotate".to_string());
        args.push(format!("{:.3}", rot));
    }
    args.push("-alpha".to_string());
    args.push("set".to_string());
    args.push("-channel".to_string());
    args.push("A".to_string());
    args.push("-evaluate".to_string());
    args.push("multiply".to_string());
    args.push(format!("{:.3}", opacity));
    args.push("+channel".to_string());
    args.push(")".to_string());
    args.push("-gravity".to_string());
    args.push(gravity.to_string());
    args.push("-geometry".to_string());
    args.push(format!("+{}+{}", margin_x, margin_y));
    args.push("-compose".to_string());
    args.push("over".to_string());
    args.push("-composite".to_string());
}

async fn estimate_output_size(
    app: &AppHandle,
    args: &[String],
    options: &ProcessOptions,
) -> Result<(u32, u32), String> {
    if options.reduce_resolution && options.target_width > 0 && options.target_height > 0 {
        return Ok((options.target_width, options.target_height));
    }
    if options.compress
        && options.compress_resize
        && options.compress_width > 0
        && options.compress_height > 0
    {
        return Ok((options.compress_width, options.compress_height));
    }
    let input = args.first().map(|s| s.as_str()).unwrap_or("");
    if !input.is_empty() && Path::new(input).is_file() {
        if let Ok(size) = probe_image_size(app, input).await {
            // 旋转 90/270 时宽高对调（粗略）
            if options.rotate {
                match options.rotation_degrees {
                    90 | -270 | 270 | -90 => return Ok((size.1, size.0)),
                    _ => {}
                }
            }
            return Ok(size);
        }
    }
    Ok((1920, 1080))
}

pub async fn process_video(
    app: &AppHandle,
    input_path: &str,
    output_path: &str,
    options: &ProcessOptions,
) -> Result<(), String> {
    process_video_with_progress(app, input_path, output_path, options, None).await
}

pub async fn process_video_with_progress(
    app: &AppHandle,
    input_path: &str,
    output_path: &str,
    options: &ProcessOptions,
    on_progress: Option<ProgressCallback>,
) -> Result<(), String> {
    validate_watermark(options)?;
    let _ = hw::ensure_detected(app, false).await;

    let use_wm = watermark_enabled(options);
    let use_image_wm = use_wm && options.watermark_type == "image";

    let mut video_filters: Vec<String> = Vec::new();
    let mut has_scale_filter = false;

    if options.reduce_resolution && options.target_width > 0 && options.target_height > 0 {
        video_filters.push(format!(
            "scale={}:{}",
            options.target_width, options.target_height
        ));
        has_scale_filter = true;
    }

    if options.compress
        && options.compress_resize
        && options.compress_width > 0
        && options.compress_height > 0
    {
        video_filters.push(format!(
            "scale={}:{}",
            options.compress_width, options.compress_height
        ));
        has_scale_filter = true;
    }

    if options.rotate {
        match options.rotation_degrees {
            90 | -270 => video_filters.push("transpose=1".to_string()),
            180 | -180 => video_filters.push("transpose=1,transpose=1".to_string()),
            270 | -90 => video_filters.push("transpose=2".to_string()),
            _ => {}
        }
    }

    if options.change_framerate && options.target_framerate > 0.0 {
        video_filters.push(format!("fps={}", options.target_framerate));
    }

    if has_scale_filter {
        video_filters.push("setsar=1".to_string());
    }

    // 文字水印统一渲成 RGBA 图章再 overlay（避免 drawtext 难见字 / 平铺撑爆命令行 / pad 透明失效）
    let mut temp_files: Vec<PathBuf> = Vec::new();
    let text_stamp = if use_wm && !use_image_wm {
        let stamp = render_text_watermark_stamp(app, options).await?;
        temp_files.push(stamp.clone());
        Some(stamp)
    } else {
        None
    };

    let overlay_wm_path = if use_image_wm {
        Some(options.watermark_image_path.trim().to_string())
    } else if let Some(ref stamp) = text_stamp {
        Some(stamp.to_string_lossy().to_string())
    } else {
        None
    };

    let mut args: Vec<String> = Vec::new();
    args.push("-i".to_string());
    args.push(input_path.to_string());

    if let Some(ref wm_path) = overlay_wm_path {
        // 静态图必须 loop，否则 overlay 只有一帧
        args.push("-loop".to_string());
        args.push("1".to_string());
        args.push("-i".to_string());
        args.push(wm_path.clone());
    }

    args.push("-y".to_string());

    if let Some(ref wm_path) = overlay_wm_path {
        let is_text_stamp = text_stamp.is_some();
        let opacity = if is_text_stamp {
            1.0
        } else {
            clamp_opacity(options.watermark_image_opacity) as f32 / 100.0
        };
        let scale_pct = if is_text_stamp {
            None
        } else {
            Some(options.watermark_image_scale.clamp(1, 100))
        };
        let (canvas_w, canvas_h) = estimate_video_canvas(app, input_path, options).await;
        let stamp_size = probe_image_size(app, wm_path).await.ok();
        let filter = build_video_image_watermark_filter(
            &video_filters,
            options,
            opacity,
            scale_pct,
            canvas_w,
            canvas_h,
            stamp_size,
            // 文字图章旋转已在 ImageMagick 中烘焙
            !is_text_stamp,
        );

        args.push("-filter_complex".to_string());
        args.push(filter);
        args.push("-map".to_string());
        args.push("[outv]".to_string());
        if options.mute {
            args.push("-an".to_string());
        } else {
            args.push("-map".to_string());
            args.push("0:a?".to_string());
        }
    } else {
        if !video_filters.is_empty() {
            args.push("-vf".to_string());
            args.push(video_filters.join(","));
        }
        if options.mute {
            args.push("-map".to_string());
            args.push("0:v".to_string());
        }
    }

    let force_encode = options.convert_h265_to_h264
        || options.convert_h264_to_h265
        || options.compress
        || options.reduce_bitrate
        || options.reduce_level
        || !video_filters.is_empty()
        || use_wm;

    let want_hevc = options.convert_h264_to_h265 && !options.convert_h265_to_h264;

    if force_encode {
        let codec = if want_hevc { "hevc" } else { "h264" };
        let quality = if options.compress {
            Some(options.compress_quality)
        } else {
            None
        };
        let bitrate = if options.reduce_bitrate && !options.target_bitrate.is_empty() {
            Some(options.target_bitrate.as_str())
        } else {
            None
        };
        let profile = if options.reduce_level && !options.target_profile.is_empty() {
            Some(options.target_profile.as_str())
        } else {
            None
        };
        let level = if options.reduce_level && !options.target_level.is_empty() {
            Some(options.target_level.as_str())
        } else {
            None
        };
        append_video_encode_args(
            &mut args,
            codec,
            quality,
            bitrate,
            profile,
            level,
            want_hevc,
        );
    }

    args.push(output_path.to_string());

    let result = run_ffmpeg_with_fallback_progress(app, &args, on_progress).await;
    for tf in temp_files {
        let _ = std::fs::remove_file(tf);
    }
    result
}

fn build_video_image_watermark_filter(
    video_filters: &[String],
    options: &ProcessOptions,
    opacity: f32,
    scale_pct: Option<u32>,
    canvas_w: u32,
    canvas_h: u32,
    stamp_size: Option<(u32, u32)>,
    apply_ffmpeg_rotation: bool,
) -> String {
    let base_chain = if video_filters.is_empty() {
        "[0:v]format=yuv420p[base]".to_string()
    } else {
        format!("[0:v]{},format=yuv420p[base]", video_filters.join(","))
    };
    let (ox, oy) = ffmpeg_overlay_xy(
        &options.watermark_position,
        options.watermark_margin_x,
        options.watermark_margin_y,
    );

    let rot = normalize_rotation_deg(options.watermark_rotation);
    let rot_filter = if apply_ffmpeg_rotation && rot.abs() > 0.01 {
        let rad = -(rot as f64) * std::f64::consts::PI / 180.0;
        format!(
            ",rotate={:.6}:c=none:ow=rotw({:.6}):oh=roth({:.6})",
            rad, rad, rad
        )
    } else {
        String::new()
    };

    let sized = if let Some(scale) = scale_pct {
        format!(
            "[1:v]format=rgba,colorchannelmixer=aa={:.3}[wm0];\
[wm0][base]scale2ref=w=main_w*{}/100:h=ow/mdar[wm][base2];\
[wm]format=rgba{}[wmr]",
            opacity, scale, rot_filter
        )
    } else {
        format!(
            "[1:v]format=rgba,colorchannelmixer=aa={:.3}{}[wmr];[base]null[base2]",
            opacity, rot_filter
        )
    };

    if options.watermark_tile {
        let gap_x = options.watermark_tile_gap_x;
        let gap_y = options.watermark_tile_gap_y;
        let (stamp_w, stamp_h) = stamp_size.unwrap_or_else(|| {
            if let Some(scale) = scale_pct {
                let w = ((canvas_w as f64) * (scale as f64) / 100.0).round().max(1.0) as u32;
                (w, w)
            } else {
                (120, 40)
            }
        });
        let (cols, rows) = tile_grid_count(
            canvas_w,
            canvas_h,
            stamp_w,
            stamp_h,
            gap_x,
            gap_y,
            options.watermark_margin_x,
            options.watermark_margin_y,
        );
        // pad 必须用 black@0.0；0x00000000 在部分构建上会变成不透明黑
        format!(
            "{};{};\
[wmr]pad=iw+{gap_x}:ih+{gap_y}:0:0:color=black@0.0[wmp];\
[wmp]tile={cols}x{rows}[wmt];\
[base2][wmt]overlay={mx}:{my}:shortest=1:format=auto[outv]",
            base_chain,
            sized,
            gap_x = gap_x,
            gap_y = gap_y,
            cols = cols,
            rows = rows,
            mx = options.watermark_margin_x,
            my = options.watermark_margin_y
        )
    } else {
        format!(
            "{};{};[base2][wmr]overlay={}:{}:format=auto[outv]",
            base_chain, sized, ox, oy
        )
    }
}

fn tile_grid_count(
    canvas_w: u32,
    canvas_h: u32,
    stamp_w: u32,
    stamp_h: u32,
    gap_x: u32,
    gap_y: u32,
    margin_x: u32,
    margin_y: u32,
) -> (u32, u32) {
    let step_x = (stamp_w.saturating_add(gap_x)).max(1);
    let step_y = (stamp_h.saturating_add(gap_y)).max(1);
    let usable_w = canvas_w.saturating_sub(margin_x).max(1);
    let usable_h = canvas_h.saturating_sub(margin_y).max(1);
    let cols = ((usable_w + step_x - 1) / step_x).clamp(1, 40);
    let rows = ((usable_h + step_y - 1) / step_y).clamp(1, 40);
    (cols, rows)
}

async fn render_text_watermark_stamp(
    app: &AppHandle,
    options: &ProcessOptions,
) -> Result<PathBuf, String> {
    let path = std::env::temp_dir().join(format!(
        "source_transform_wm_stamp_{}.png",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0)
    ));

    let text = options.watermark_text.trim().replace('%', "%%");
    let fill = rgba_css(
        &options.watermark_font_color,
        options.watermark_font_opacity,
    );
    let font_size = options.watermark_font_size.max(8);
    let mut args: Vec<String> = vec![
        "-background".into(),
        "none".into(),
        "-fill".into(),
        fill,
    ];
    if options.watermark_stroke && options.watermark_stroke_width > 0 {
        args.push("-stroke".into());
        args.push(rgba_css(
            &options.watermark_stroke_color,
            options.watermark_font_opacity,
        ));
        args.push("-strokewidth".into());
        args.push(options.watermark_stroke_width.to_string());
    } else {
        args.push("-stroke".into());
        args.push("none".into());
    }
    if let Some(font) = watermark::default_font_path() {
        args.push("-font".into());
        args.push(magick_path(&font.to_string_lossy()));
    }
    args.push("-pointsize".into());
    args.push(font_size.to_string());
    args.push(format!("label:{}", text));
    // 强制 RGBA，避免 GrayAlpha 导致 FFmpeg pad/overlay 透明失效变黑底
    args.push("-type".into());
    args.push("TrueColorAlpha".into());

    let rot = magick_positive_degrees(options.watermark_rotation);
    if rot > 0.01 {
        args.push("-background".into());
        args.push("none".into());
        args.push("-rotate".into());
        args.push(format!("{:.3}", rot));
    }
    args.push(format!("png32:{}", path.to_string_lossy()));

    let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let output = app
        .shell()
        .sidecar("magick")
        .map_err(|e| format!("无法找到ImageMagick: {}", e))?
        .args(&refs)
        .output()
        .await
        .map_err(|e| format!("无法执行ImageMagick: {}", e))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("生成文字水印图章失败: {}", stderr));
    }
    Ok(path)
}

async fn probe_video_size(app: &AppHandle, path: &str) -> Result<(u32, u32), String> {
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
            path,
        ])
        .output()
        .await
        .map_err(|e| format!("无法执行FFprobe: {}", e))?;
    if !output.status.success() {
        return Err("无法读取视频尺寸".to_string());
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parts: Vec<&str> = stdout.trim().split(',').collect();
    if parts.len() >= 2 {
        let w = parts[0].parse().map_err(|_| "无法解析宽度")?;
        let h = parts[1].parse().map_err(|_| "无法解析高度")?;
        Ok((w, h))
    } else {
        Err("无法解析视频尺寸".to_string())
    }
}

async fn estimate_video_canvas(
    app: &AppHandle,
    input_path: &str,
    options: &ProcessOptions,
) -> (u32, u32) {
    if options.reduce_resolution && options.target_width > 0 && options.target_height > 0 {
        return (options.target_width, options.target_height);
    }
    if options.compress
        && options.compress_resize
        && options.compress_width > 0
        && options.compress_height > 0
    {
        return (options.compress_width, options.compress_height);
    }
    if let Ok((w, h)) = probe_video_size(app, input_path).await {
        if options.rotate {
            match options.rotation_degrees {
                90 | -270 | 270 | -90 => return (h, w),
                _ => {}
            }
        }
        return (w, h);
    }
    (1920, 1080)
}

pub async fn crop_image_region(
    app: &AppHandle,
    input_path: &str,
    output_path: &str,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) -> Result<(), String> {
    let geometry = format!("{}x{}+{}+{}", width, height, x, y);
    let args = [input_path, "-crop", &geometry, "+repage", output_path];

    let output = app
        .shell()
        .sidecar("magick")
        .map_err(|e| format!("无法找到ImageMagick: {}", e))?
        .args(&args)
        .output()
        .await
        .map_err(|e| format!("无法执行ImageMagick: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("图片裁剪失败: {}", stderr));
    }
    Ok(())
}

pub async fn crop_video_region(
    app: &AppHandle,
    input_path: &str,
    output_path: &str,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) -> Result<(), String> {
    crop_video_region_with_progress(app, input_path, output_path, x, y, width, height, None).await
}

pub async fn crop_video_region_with_progress(
    app: &AppHandle,
    input_path: &str,
    output_path: &str,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    on_progress: Option<ProgressCallback>,
) -> Result<(), String> {
    let _ = hw::ensure_detected(app, false).await;

    let even = |v: u32| (v & !1u32).max(2);
    let crop_w = even(width);
    let crop_h = even(height);
    let crop_x = x & !1u32;
    let crop_y = y & !1u32;
    let filter = format!("crop={}:{}:{}:{}", crop_w, crop_h, crop_x, crop_y);

    let mut args: Vec<String> = vec![
        "-i".to_string(),
        input_path.to_string(),
        "-vf".to_string(),
        filter,
        "-an".to_string(),
    ];
    append_video_encode_args(
        &mut args,
        "h264",
        Some(80),
        None,
        Some("main"),
        Some("5.1"),
        false,
    );
    args.push("-y".to_string());
    args.push(output_path.to_string());

    run_ffmpeg_with_fallback_progress(app, &args, on_progress).await
}

pub async fn crop_image_by_ratio(
    app: &AppHandle,
    input_path: &str,
    output_path: &str,
    crop_w: u32,
    crop_h: u32,
    crop_x: u32,
    crop_y: u32,
) -> Result<(), String> {
    crop_image_region(app, input_path, output_path, crop_x, crop_y, crop_w, crop_h).await
}

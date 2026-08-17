use tauri::AppHandle;
use tauri_plugin_shell::ShellExt;

use crate::hw::{self, append_video_encode_args, run_ffmpeg_with_fallback};
use crate::types::ProcessOptions;

pub async fn process_image(
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

pub async fn process_video(
    app: &AppHandle,
    input_path: &str,
    output_path: &str,
    options: &ProcessOptions,
) -> Result<(), String> {
    let _ = hw::ensure_detected(app, false).await;

    let mut args: Vec<String> = vec![
        "-i".to_string(),
        input_path.to_string(),
        "-y".to_string(),
    ];

    // 视频滤镜
    let mut video_filters: Vec<String> = Vec::new();
    let mut has_scale_filter = false;

    // 调整分辨率（独立选项）
    if options.reduce_resolution && options.target_width > 0 && options.target_height > 0 {
        video_filters.push(format!(
            "scale={}:{}",
            options.target_width, options.target_height
        ));
        has_scale_filter = true;
    }

    // 压缩时降低分辨率
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

    if has_scale_filter {
        video_filters.push("setsar=1".to_string());
    }

    let vf_arg = if !video_filters.is_empty() {
        Some(video_filters.join(","))
    } else {
        None
    };
    if let Some(ref vf) = vf_arg {
        args.push("-vf".to_string());
        args.push(vf.clone());
    }

    // 编码选择：有滤镜/压缩/转码/码率/level 时强制重编码
    let want_hevc = options.convert_h264_to_h265 && !options.convert_h265_to_h264;
    let force_encode = options.convert_h265_to_h264
        || options.convert_h264_to_h265
        || options.compress
        || options.reduce_bitrate
        || options.reduce_level
        || !video_filters.is_empty();

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

    // 视频静音（去除音轨）
    if options.mute {
        args.push("-map".to_string());
        args.push("0:v".to_string());
    }

    args.push(output_path.to_string());

    run_ffmpeg_with_fallback(app, &args).await
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

    run_ffmpeg_with_fallback(app, &args).await
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

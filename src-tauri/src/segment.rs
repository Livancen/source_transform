use std::path::PathBuf;
use std::sync::Arc;

use tauri::AppHandle;
use tauri_plugin_shell::ShellExt;

use crate::commands::{
    build_ratio_video_encode, compute_cover_crop, emit_progress, normalize_video_ext,
};
use crate::hw::{self, append_video_encode_args, run_ffmpeg_with_fallback_progress};
use crate::logger;
use crate::naming::{build_output_name, join_output_path};
use crate::process::{probe_video_codec, VideoRegionEncode};
use crate::types::{NamingOptions, RatioVideoOutput, SegmentCropJob, SegmentPieceInput};

#[derive(Debug, Clone, PartialEq)]
pub struct KeptSeg {
    pub start: f64,
    pub end: f64,
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
    pub out_w: u32,
    pub out_h: u32,
}

fn parse_ratio(ratio: &str) -> Result<f64, String> {
    let parts: Vec<&str> = ratio.split(':').collect();
    if parts.len() != 2 {
        return Err(format!("比例格式无效: {ratio}"));
    }
    let w: f64 = parts[0]
        .trim()
        .parse()
        .map_err(|_| format!("比例格式无效: {ratio}"))?;
    let h: f64 = parts[1]
        .trim()
        .parse()
        .map_err(|_| format!("比例格式无效: {ratio}"))?;
    if w <= 0.0 || h <= 0.0 {
        return Err(format!("比例值无效: {ratio}"));
    }
    Ok(w / h)
}

fn fit_rect(width: u32, height: u32, x: u32, y: u32, w: u32, h: u32) -> (u32, u32, u32, u32) {
    let max_w = (width & !1).max(2);
    let max_h = (height & !1).max(2);
    let w = if max_w < 16 { max_w } else { (w & !1).clamp(16, max_w) };
    let h = if max_h < 16 { max_h } else { (h & !1).clamp(16, max_h) };
    let (x, y) = clamp_rect(width, height, x, y, w, h);
    (x, y, w, h)
}

fn clamp_rect(width: u32, height: u32, mut x: u32, mut y: u32, w: u32, h: u32) -> (u32, u32) {
    let max_x = width.saturating_sub(w);
    let max_y = height.saturating_sub(h);
    x = x.min(max_x) & !1;
    y = y.min(max_y) & !1;
    if x + w > width {
        x = width.saturating_sub(w) & !1;
    }
    if y + h > height {
        y = height.saturating_sub(h) & !1;
    }
    (x, y)
}

/// 把用户分段收成可拼接的裁切段。空列表表示整段按偏移裁。丢弃段和空隙不输出。
pub fn resolve_segments(
    width: u32,
    height: u32,
    duration: f64,
    ratio: f64,
    fallback_offset_x: f64,
    fallback_offset_y: f64,
    pieces: &[SegmentPieceInput],
) -> Result<Vec<KeptSeg>, String> {
    if width < 2 || height < 2 || !(duration > 0.0) {
        return Err("视频尺寸或时长无效".to_string());
    }
    let (canon_w, canon_h, _, _) = compute_cover_crop(width, height, ratio, 0.0, 0.0);
    if canon_w < 2 || canon_h < 2 {
        return Err("目标比例无法生成有效裁切".to_string());
    }

    let source: Vec<SegmentPieceInput> = if pieces.is_empty() {
        vec![SegmentPieceInput {
            start: 0.0,
            end: duration,
            mode: "offset".to_string(),
            offset_x: fallback_offset_x,
            offset_y: fallback_offset_y,
            crop_x: 0,
            crop_y: 0,
            crop_w: 0,
            crop_h: 0,
        }]
    } else {
        let mut items = pieces.to_vec();
        items.sort_by(|a, b| a.start.partial_cmp(&b.start).unwrap_or(std::cmp::Ordering::Equal));
        items
    };

    let mut kept = Vec::new();
    for piece in source {
        if piece.mode == "drop" {
            continue;
        }
        let start = piece.start.clamp(0.0, duration);
        let end = piece.end.clamp(0.0, duration);
        if end - start < 0.08 {
            continue;
        }
        let (x, y, w, h) = if piece.mode == "position" && piece.crop_w >= 16 && piece.crop_h >= 16 {
            fit_rect(width, height, piece.crop_x, piece.crop_y, piece.crop_w, piece.crop_h)
        } else if piece.mode == "position" {
            let (x, y) = clamp_rect(width, height, piece.crop_x, piece.crop_y, canon_w, canon_h);
            (x, y, canon_w, canon_h)
        } else {
            let (_, _, cx, cy) =
                compute_cover_crop(width, height, ratio, piece.offset_x, piece.offset_y);
            (cx, cy, canon_w, canon_h)
        };
        kept.push(KeptSeg {
            start,
            end,
            x,
            y,
            w,
            h,
            out_w: canon_w,
            out_h: canon_h,
        });
    }
    if kept.is_empty() {
        return Err("没有可导出的片段".to_string());
    }
    Ok(kept)
}

fn fps_text(fps: f32) -> String {
    if (fps - fps.round()).abs() < 0.05 {
        format!("{}", fps.round() as i32)
    } else {
        format!("{:.2}", fps)
    }
}

pub fn filter_graph(segs: &[KeptSeg], audio: bool, encode: &VideoRegionEncode) -> String {
    let mut parts = Vec::new();
    for (i, seg) in segs.iter().enumerate() {
        let mut chain = format!(
            "[0:v]trim=start={:.3}:end={:.3},setpts=PTS-STARTPTS,crop={}:{}:{}:{},scale={}:{},setsar=1",
            seg.start, seg.end, seg.w, seg.h, seg.x, seg.y, seg.out_w, seg.out_h
        );
        if encode.scale_width > 0 && encode.scale_height > 0 {
            chain.push_str(&format!(
                ",scale={}:{}",
                encode.scale_width, encode.scale_height
            ));
        }
        if encode.framerate > 0.0 {
            chain.push_str(&format!(",fps={}", fps_text(encode.framerate)));
        }
        chain.push_str(&format!(",format=yuv420p[v{i}]"));
        parts.push(chain);
        if audio {
            parts.push(format!(
                "[0:a]atrim=start={:.3}:end={:.3},asetpts=PTS-STARTPTS[a{i}]",
                seg.start, seg.end
            ));
        }
    }
    let mut concat_inputs = String::new();
    for i in 0..segs.len() {
        concat_inputs.push_str(&format!("[v{i}]"));
        if audio {
            concat_inputs.push_str(&format!("[a{i}]"));
        }
    }
    let audio_flag = if audio { 1 } else { 0 };
    let n = segs.len();
    if audio {
        parts.push(format!(
            "{concat_inputs}concat=n={n}:v=1:a={audio_flag}[outv][outa]"
        ));
    } else {
        parts.push(format!(
            "{concat_inputs}concat=n={n}:v=1:a=0[outv]"
        ));
    }
    parts.join(";")
}

async fn has_audio(app: &AppHandle, path: &str) -> bool {
    let Ok(sidecar) = app.shell().sidecar("ffprobe") else {
        return false;
    };
    let Ok(output) = sidecar
        .args([
            "-v",
            "error",
            "-select_streams",
            "a:0",
            "-show_entries",
            "stream=index",
            "-of",
            "csv=p=0",
            path,
        ])
        .output()
        .await
    else {
        return false;
    };
    output.status.success() && !String::from_utf8_lossy(&output.stdout).trim().is_empty()
}

async fn encode_segments(
    app: &AppHandle,
    input: &str,
    output: &str,
    segs: &[KeptSeg],
    encode: &VideoRegionEncode,
    with_audio: bool,
    faststart: bool,
    on_progress: Option<Arc<dyn Fn(f64) + Send + Sync>>,
) -> Result<(), String> {
    let graph = filter_graph(segs, with_audio, encode);
    let mut args = vec![
        "-i".to_string(),
        input.to_string(),
        "-filter_complex".to_string(),
        graph,
        "-map".to_string(),
        "[outv]".to_string(),
    ];
    if with_audio {
        args.extend([
            "-map".into(),
            "[outa]".into(),
            "-c:a".into(),
            "aac".into(),
            "-b:a".into(),
            "128k".into(),
        ]);
    } else {
        args.push("-an".into());
    }
    let hevc = encode.codec == "hevc";
    append_video_encode_args(
        &mut args,
        if hevc { "hevc" } else { "h264" },
        encode.quality,
        encode.bitrate.as_deref(),
        encode.profile.as_deref(),
        encode.level.as_deref(),
        hevc,
    );
    if faststart {
        args.push("-movflags".into());
        args.push("+faststart".into());
    }
    args.push("-y".into());
    args.push(output.to_string());
    run_ffmpeg_with_fallback_progress(app, &args, on_progress).await
}

#[tauri::command]
pub async fn segment_crop_videos(
    app: AppHandle,
    output_dir: String,
    jobs: Vec<SegmentCropJob>,
    video: RatioVideoOutput,
    naming: NamingOptions,
) -> Result<String, String> {
    if jobs.is_empty() {
        return Err("请先勾选要处理的视频".to_string());
    }
    std::fs::create_dir_all(&output_dir).map_err(|e| e.to_string())?;
    logger::info(format!(
        "开始分段裁剪 {} 个视频 mute={} compress={}",
        jobs.len(),
        video.mute,
        video.compress
    ));

    let total = jobs.len();
    let mut success = 0usize;
    let mut errors = Vec::new();

    for (index, job) in jobs.iter().enumerate() {
        let current = index + 1;
        let name = PathBuf::from(&job.path)
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| job.path.clone());
        let name = format!("{name} ({})", job.ratio);
        emit_progress(&app, "segment-progress", current, total, &name, "processing", 0.0);

        let result = export_one(&app, &output_dir, job, &video, &naming, current, total).await;
        match result {
            Ok(_) => success += 1,
            Err(e) => {
                logger::warn(format!("分段裁剪失败 {name}: {e}"));
                errors.push(format!("{name}: {e}"));
            }
        }
    }

    emit_progress(&app, "segment-progress", total, total, "", "completed", 100.0);
    if errors.is_empty() {
        let msg = format!("分段裁剪完成: {success} 个成片");
        logger::info(&msg);
        Ok(msg)
    } else {
        Ok(format!(
            "分段裁剪完成: {success} 个成片成功, {} 失败\n{}",
            errors.len(),
            errors.join("\n")
        ))
    }
}

async fn export_one(
    app: &AppHandle,
    output_dir: &str,
    job: &SegmentCropJob,
    video: &RatioVideoOutput,
    naming: &NamingOptions,
    current: usize,
    total: usize,
) -> Result<(), String> {
    let input = PathBuf::from(&job.path);
    if !input.is_file() {
        return Err("文件不存在".to_string());
    }
    let ratio = parse_ratio(&job.ratio)?;
    let (width, height) = crate::commands::get_video_dimensions(app.clone(), job.path.clone()).await?;
    let duration = hw::probe_duration_secs(app, &job.path)
        .await
        .ok_or_else(|| "无法读取视频时长".to_string())?;
    let kept = resolve_segments(
        width,
        height,
        duration,
        ratio,
        job.offset_x,
        job.offset_y,
        &job.segments,
    )?;

    let file_name = input
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "segment.mp4".to_string());
    let stem = PathBuf::from(&file_name)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "segment".to_string());
    let ratio_tag = job.ratio.replace(':', "-");
    let orig_ext = input
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("mp4");
    let ext = if video.convert_mp4 {
        "mp4".to_string()
    } else {
        normalize_video_ext(orig_ext)
    };
    let aliased = format!("{stem}-seg-{ratio_tag}.{ext}");
    let out_name = build_output_name(&aliased, naming, Some(&ext));
    let output_path = join_output_path(output_dir, &out_name);
    let output = output_path.to_string_lossy().to_string();

    let label = format!("{file_name} ({})", job.ratio);
    let app_cb = app.clone();
    let on_progress: Arc<dyn Fn(f64) + Send + Sync> = Arc::new(move |pct: f64| {
        emit_progress(&app_cb, "segment-progress", current, total, &label, "processing", pct);
    });

    let source_codec = probe_video_codec(app, &job.path).await.unwrap_or_default();
    let encode = build_ratio_video_encode(Some(video), &source_codec);
    let audio = !encode.mute && has_audio(app, &job.path).await;
    let faststart = ext == "mp4" || ext == "m4v";
    if let Err(err) = encode_segments(
        app,
        &job.path,
        &output,
        &kept,
        &encode,
        audio,
        faststart,
        Some(on_progress.clone()),
    )
    .await
    {
        if audio {
            logger::warn(format!("分段裁剪保留音轨失败，改为静音重试: {err}"));
            encode_segments(app, &job.path, &output, &kept, &encode, false, faststart, Some(on_progress)).await?;
        } else {
            return Err(err);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_job_uses_centered_cover() {
        let kept = resolve_segments(1920, 1080, 10.0, 1.0, 0.0, 0.0, &[]).unwrap();
        assert_eq!(kept.len(), 1);
        assert_eq!(kept[0].w, 1080);
        assert_eq!(kept[0].h, 1080);
        assert_eq!(kept[0].x, 420);
        assert_eq!(kept[0].start, 0.0);
        assert_eq!(kept[0].end, 10.0);
    }

    #[test]
    fn drop_is_omitted_and_position_stays_in_frame() {
        let pieces = vec![
            SegmentPieceInput {
                start: 0.0,
                end: 5.0,
                mode: "drop".into(),
                offset_x: 0.0,
                offset_y: 0.0,
                crop_x: 0,
                crop_y: 0,
                crop_w: 0,
                crop_h: 0,
            },
            SegmentPieceInput {
                start: 5.0,
                end: 10.0,
                mode: "position".into(),
                offset_x: 0.0,
                offset_y: 0.0,
                crop_x: 9000,
                crop_y: 9000,
                crop_w: 0,
                crop_h: 0,
            },
        ];
        let kept = resolve_segments(1920, 1080, 10.0, 1.0, 0.0, 0.0, &pieces).unwrap();
        assert_eq!(kept.len(), 1);
        assert!((kept[0].start - 5.0).abs() < 0.001);
        assert_eq!(kept[0].w, 1080);
        assert!(kept[0].x + kept[0].w <= 1920);
        assert!(kept[0].y + kept[0].h <= 1080);
    }

    #[test]
    fn filter_graph_concatenates_video_only() {
        let segs = vec![
            KeptSeg { start: 0.0, end: 1.5, x: 0, y: 0, w: 1080, h: 1920, out_w: 1080, out_h: 1920 },
            KeptSeg { start: 1.5, end: 3.0, x: 100, y: 0, w: 1080, h: 1920, out_w: 1080, out_h: 1920 },
        ];
        let graph = filter_graph(&segs, false, &VideoRegionEncode::legacy());
        assert!(graph.contains("concat=n=2:v=1:a=0[outv]"));
        assert!(!graph.contains("[outa]"));
        assert!(graph.contains("crop=1080:1920:100:0"));
    }
}

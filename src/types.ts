export interface FileInfo {
  path: string;
  name: string;
  file_type: string;
  /** 字节 */
  size_bytes: number;
}

export function formatFileSizeMb(bytes: number): string {
  if (!bytes || bytes <= 0) return "0 MB";
  const mb = bytes / (1024 * 1024);
  if (mb < 0.01) return "<0.01 MB";
  if (mb < 10) return `${mb.toFixed(2)} MB`;
  if (mb < 100) return `${mb.toFixed(1)} MB`;
  return `${Math.round(mb)} MB`;
}

export interface ProcessProgress {
  current: number;
  total: number;
  current_file: string;
  status: string;
}

export type WatermarkType = "text" | "image";

/** 九宫格位置 */
export type WatermarkPosition =
  | "tl"
  | "tc"
  | "tr"
  | "ml"
  | "mc"
  | "mr"
  | "bl"
  | "bc"
  | "br";

export interface ProcessOptions {
  compress: boolean;
  compress_quality: number;
  compress_resize: boolean;
  compress_width: number;
  compress_height: number;
  reduce_resolution: boolean;
  target_width: number;
  target_height: number;
  reduce_bitrate: boolean;
  target_bitrate: string;
  reduce_level: boolean;
  target_level: string;
  target_profile: string;
  convert_h265_to_h264: boolean;
  convert_h264_to_h265: boolean;
  convert_format: boolean;
  target_format: string;
  rotate: boolean;
  rotation_degrees: number;
  mute: boolean;
  change_framerate: boolean;
  target_framerate: number;
  watermark: boolean;
  watermark_type: WatermarkType;
  watermark_text: string;
  watermark_font_size: number;
  watermark_font_color: string;
  watermark_font_opacity: number;
  watermark_stroke: boolean;
  watermark_stroke_width: number;
  watermark_stroke_color: string;
  watermark_image_path: string;
  watermark_image_scale: number;
  watermark_image_opacity: number;
  watermark_position: WatermarkPosition;
  watermark_margin_x: number;
  watermark_margin_y: number;
  /** 旋转角度（度），顺时针为正，范围约 -180～180 */
  watermark_rotation: number;
  watermark_tile: boolean;
  watermark_tile_gap_x: number;
  watermark_tile_gap_y: number;
}

export interface NamingOptions {
  use_original_name: boolean;
  use_timestamp: boolean;
  use_datetime: boolean;
  custom_text: string;
}

export interface HwEncoderInfo {
  id: string;
  label: string;
  codec: string;
  available: boolean;
}

export interface HwAccelOptions {
  mode: string;
  active_h264: string;
  active_hevc: string;
  encoders: HwEncoderInfo[];
}

export type WorkMode = "image" | "video" | "ratio" | "crop" | "merge" | "join";

export type VideoMergeLayout = "vertical" | "horizontal";

export interface VideoMergeSlot {
  path: string;
  name: string;
  width: number;
  height: number;
}

export interface VideoMergeOptions {
  layout: VideoMergeLayout;
  slots: [VideoMergeSlot, VideoMergeSlot];
  output_width?: number;
  output_height?: number;
  output_path: string;
  media_kind: "video" | "image";
  /** 输出帧率：30 | 60；不足时由 FFmpeg fps 滤镜补帧 */
  output_fps?: 30 | 60;
  /** 是否指定 H.264 Level */
  set_level?: boolean;
  /** 如 "4.0" "4.1" "5.1" */
  video_level?: string;
  /** 如 "high" "main" "baseline" */
  video_profile?: string;
}

export interface CustomCropOptions {
  input_path: string;
  output_dir: string;
  crop_x: number;
  crop_y: number;
  crop_width: number;
  crop_height: number;
  naming: NamingOptions;
}

export type JoinFit = "cover" | "contain" | "fill";

export interface JoinItem {
  id: string;
  path: string;
  name: string;
  /** 该图层素材类型 */
  media_kind: "video" | "image";
  x: number;
  y: number;
  width: number;
  height: number;
  z: number;
  fit: JoinFit;
  /** 对该图层区域做高斯模糊（位置/大小即模糊区） */
  blur?: boolean;
  /** 模糊强度 sigma，约 1–50，默认 20 */
  blur_sigma?: number;
}

export interface JoinOptions {
  /**
   * 输出类型：全图 → image；含任意视频 → video
   * 可由后端根据 items 再校验
   */
  media_kind: "video" | "image";
  canvas_width: number;
  canvas_height: number;
  /** "#000000" | "#ffffff" | "transparent" */
  background: string;
  items: JoinItem[];
  output_path: string;
  /** 输出帧率：30 | 60；不足时补帧（仅视频输出） */
  output_fps?: 30 | 60;
  set_level?: boolean;
  video_level?: string;
  video_profile?: string;
}

export function defaultProcessOptions(): ProcessOptions {
  return {
    compress: false,
    compress_quality: 80,
    compress_resize: false,
    compress_width: 1280,
    compress_height: 720,
    reduce_resolution: false,
    target_width: 1920,
    target_height: 1080,
    reduce_bitrate: false,
    target_bitrate: "2M",
    reduce_level: false,
    target_level: "4.0",
    target_profile: "high",
    convert_h265_to_h264: false,
    convert_h264_to_h265: false,
    convert_format: false,
    target_format: "mp4",
    rotate: false,
    rotation_degrees: 90,
    mute: false,
    change_framerate: false,
    target_framerate: 30,
    watermark: false,
    watermark_type: "text",
    watermark_text: "水印",
    watermark_font_size: 36,
    watermark_font_color: "#FFFFFF",
    watermark_font_opacity: 60,
    watermark_stroke: true,
    watermark_stroke_width: 2,
    watermark_stroke_color: "#000000",
    watermark_image_path: "",
    watermark_image_scale: 15,
    watermark_image_opacity: 60,
    watermark_position: "br",
    watermark_margin_x: 16,
    watermark_margin_y: 16,
    watermark_rotation: 0,
    watermark_tile: false,
    watermark_tile_gap_x: 80,
    watermark_tile_gap_y: 80,
  };
}

export const WATERMARK_POSITIONS: {
  id: WatermarkPosition;
  label: string;
}[] = [
  { id: "tl", label: "左上" },
  { id: "tc", label: "中上" },
  { id: "tr", label: "右上" },
  { id: "ml", label: "左中" },
  { id: "mc", label: "居中" },
  { id: "mr", label: "右中" },
  { id: "bl", label: "左下" },
  { id: "bc", label: "中下" },
  { id: "br", label: "右下" },
];

export function watermarkSummary(options: ProcessOptions): string {
  if (!options.watermark) return "";
  const pos =
    WATERMARK_POSITIONS.find((p) => p.id === options.watermark_position)
      ?.label || options.watermark_position;
  const rot = Math.round(options.watermark_rotation || 0);
  const rotPart = rot !== 0 ? ` · ${rot}°` : "";
  if (options.watermark_type === "text") {
    const text = (options.watermark_text || "").trim() || "文字";
    const short = text.length > 8 ? `${text.slice(0, 8)}…` : text;
    return `${short} · ${pos}${rotPart}`;
  }
  const scale = Math.max(1, Math.min(100, options.watermark_image_scale || 15));
  return `图片 ${scale}% · ${pos}${rotPart}`;
}

export function defaultNamingOptions(): NamingOptions {
  return {
    use_original_name: true,
    use_timestamp: false,
    use_datetime: false,
    custom_text: "",
  };
}

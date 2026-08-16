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
}

export interface NamingOptions {
  use_original_name: boolean;
  use_timestamp: boolean;
  use_datetime: boolean;
  custom_text: string;
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
  };
}

export function defaultNamingOptions(): NamingOptions {
  return {
    use_original_name: true,
    use_timestamp: false,
    use_datetime: false,
    custom_text: "",
  };
}

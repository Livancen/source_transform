export interface FileInfo {
  path: string;
  name: string;
  file_type: string;
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
  convert_format: boolean;
  target_format: string;
  crop: boolean;
  crop_width: number;
  crop_height: number;
  crop_x: number;
  crop_y: number;
  rotate: boolean;
  rotation_degrees: number;
  mute: boolean;
  change_framerate: boolean;
  target_framerate: number;
}

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
}

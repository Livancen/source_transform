import type { FileInfo } from "../types";
import { computeCoverCrop, parseRatio } from "./ratioCrop";

export type SegmentMode = "offset" | "position" | "drop";

export interface SegmentPiece {
  id: string;
  start: number;
  end: number;
  mode: SegmentMode;
  offsetX: number;
  offsetY: number;
  cropX: number;
  cropY: number;
  /** 位置模式的框大小。0 表示还没改过，用比例裁剪的默认框。 */
  cropW: number;
  cropH: number;
}

export interface SegmentFileConfig {
  ratio: string;
  segments: SegmentPiece[];
}

export interface SegmentGlobal {
  ratio: string;
  offsetX: number;
  offsetY: number;
  mute: boolean;
}

export interface SegmentExportPiece {
  start: number;
  end: number;
  mode: SegmentMode;
  offset_x: number;
  offset_y: number;
  crop_x: number;
  crop_y: number;
  crop_w: number;
  crop_h: number;
}

export interface SegmentExportJob {
  path: string;
  ratio: string;
  offset_x: number;
  offset_y: number;
  segments: SegmentExportPiece[];
}

const GLOBAL_KEY = "segment-crop-global";
const CONFIG_KEY = "segment-crop-configs";

export function defaultSegmentGlobal(): SegmentGlobal {
  return { ratio: "9:16", offsetX: 0, offsetY: 0, mute: false };
}

export function clampOffset(value: number): number {
  if (!Number.isFinite(value)) return 0;
  return Math.min(100, Math.max(-100, Math.round(value)));
}

export function loadSegmentGlobal(): SegmentGlobal {
  const base = defaultSegmentGlobal();
  try {
    const raw = localStorage.getItem(GLOBAL_KEY);
    if (!raw) return base;
    const parsed = JSON.parse(raw) as Partial<SegmentGlobal>;
    if (typeof parsed.ratio === "string" && parsed.ratio.trim()) base.ratio = parsed.ratio.trim();
    base.offsetX = clampOffset(Number(parsed.offsetX));
    base.offsetY = clampOffset(Number(parsed.offsetY));
    base.mute = Boolean(parsed.mute);
  } catch {
    /* ignore */
  }
  return base;
}

export function saveSegmentGlobal(value: SegmentGlobal) {
  const next: SegmentGlobal = {
    ratio: value.ratio.trim() || "9:16",
    offsetX: clampOffset(value.offsetX),
    offsetY: clampOffset(value.offsetY),
    mute: Boolean(value.mute),
  };
  localStorage.setItem(GLOBAL_KEY, JSON.stringify(next));
}

function readAll(): Record<string, SegmentFileConfig> {
  try {
    const raw = localStorage.getItem(CONFIG_KEY);
    if (!raw) return {};
    const parsed = JSON.parse(raw) as Record<string, SegmentFileConfig>;
    return parsed && typeof parsed === "object" ? parsed : {};
  } catch {
    return {};
  }
}

export function loadSegmentConfig(path: string): SegmentFileConfig | null {
  const cfg = readAll()[path];
  if (!cfg || !Array.isArray(cfg.segments) || cfg.segments.length === 0) return null;
  if (!parseRatio(cfg.ratio || "")) return null;
  return cfg;
}

export function saveSegmentConfig(path: string, config: SegmentFileConfig) {
  const all = readAll();
  all[path] = config;
  localStorage.setItem(CONFIG_KEY, JSON.stringify(all));
}

export function clearSegmentConfig(path: string) {
  const all = readAll();
  delete all[path];
  localStorage.setItem(CONFIG_KEY, JSON.stringify(all));
}

export function segmentConfigPaths(): Set<string> {
  return new Set(Object.keys(readAll()).filter((path) => loadSegmentConfig(path)));
}

export function newSegmentId(): string {
  return `seg-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 7)}`;
}

export function fullSpanSegment(duration: number, global: SegmentGlobal): SegmentPiece {
  const box = { offsetX: global.offsetX, offsetY: global.offsetY };
  return {
    id: newSegmentId(),
    start: 0,
    end: Math.max(0.1, duration),
    mode: "offset",
    offsetX: box.offsetX,
    offsetY: box.offsetY,
    cropX: 0,
    cropY: 0,
    cropW: 0,
    cropH: 0,
  };
}

export function formatClock(sec: number): string {
  if (!Number.isFinite(sec) || sec < 0) return "0:00";
  const total = Math.floor(sec);
  const m = Math.floor(total / 60);
  const s = total % 60;
  const tenth = Math.floor((sec - total) * 10);
  return `${m}:${String(s).padStart(2, "0")}.${tenth}`;
}

export function buildSegmentJobs(files: FileInfo[], global: SegmentGlobal): SegmentExportJob[] {
  return files
    .filter((file) => file.file_type === "video")
    .map((file) => {
      const cfg = loadSegmentConfig(file.path);
      if (!cfg) {
        return {
          path: file.path,
          ratio: global.ratio,
          offset_x: global.offsetX,
          offset_y: global.offsetY,
          segments: [],
        };
      }
      return {
        path: file.path,
        ratio: cfg.ratio,
        offset_x: global.offsetX,
        offset_y: global.offsetY,
        segments: cfg.segments.map((piece) => ({
          start: piece.start,
          end: piece.end,
          mode: piece.mode,
          offset_x: piece.offsetX,
          offset_y: piece.offsetY,
          crop_x: piece.cropX,
          crop_y: piece.cropY,
          crop_w: piece.cropW || 0,
          crop_h: piece.cropH || 0,
        })),
      };
    });
}

export function coverBox(
  width: number,
  height: number,
  ratioText: string,
  piece: Pick<SegmentPiece, "mode" | "offsetX" | "offsetY" | "cropX" | "cropY" | "cropW" | "cropH">,
) {
  const ratio = parseRatio(ratioText);
  if (!ratio || width < 2 || height < 2) return null;
  const canon = computeCoverCrop(width, height, ratio, 0, 0);
  if (piece.mode === "position") {
    const cropW = Math.min(width, Math.max(16, piece.cropW >= 16 ? piece.cropW : canon.cropW));
    const cropH = Math.min(height, Math.max(16, piece.cropH >= 16 ? piece.cropH : canon.cropH));
    return {
      cropW,
      cropH,
      cropX: Math.min(Math.max(0, width - cropW), Math.max(0, Math.round(piece.cropX))),
      cropY: Math.min(Math.max(0, height - cropH), Math.max(0, Math.round(piece.cropY))),
    };
  }
  return computeCoverCrop(width, height, ratio, piece.offsetX, piece.offsetY);
}

function gcd(a: number, b: number): number {
  let x = Math.abs(Math.round(a));
  let y = Math.abs(Math.round(b));
  while (y) {
    const next = x % y;
    x = y;
    y = next;
  }
  return x || 1;
}

/** 把裁切框宽高收成可读比例。能约成小整数就用整数，否则贴近常见比例，再不行用一位小数。 */
export function formatBoxRatio(width: number, height: number): string {
  const w = Math.round(width);
  const h = Math.round(height);
  if (!(w > 0) || !(h > 0)) return "";
  const g = gcd(w, h);
  const rw = w / g;
  const rh = h / g;
  if (rw <= 24 && rh <= 24) return `${rw}:${rh}`;

  const target = w / h;
  let bestA = 1;
  let bestB = 1;
  let bestErr = Infinity;
  for (let b = 1; b <= 24; b += 1) {
    const a = Math.max(1, Math.round(target * b));
    if (a > 24) continue;
    const err = Math.abs(a / b - target) / target;
    if (err < bestErr) {
      bestErr = err;
      bestA = a;
      bestB = b;
    }
  }
  if (bestErr <= 0.008) {
    const gg = gcd(bestA, bestB);
    return `${bestA / gg}:${bestB / gg}`;
  }
  const major = target >= 1 ? target : h / w;
  const text = (Math.round(major * 100) / 100).toString();
  return target >= 1 ? `${text}:1` : `1:${text}`;
}

export function offsetsFromPosition(
  width: number,
  height: number,
  ratioText: string,
  cropX: number,
  cropY: number,
): { offsetX: number; offsetY: number } {
  const ratio = parseRatio(ratioText);
  if (!ratio) return { offsetX: 0, offsetY: 0 };
  const canon = computeCoverCrop(width, height, ratio, 0, 0);
  const slackX = Math.max(0, width - canon.cropW);
  const slackY = Math.max(0, height - canon.cropH);
  const axis = (slack: number, origin: number) => {
    if (slack <= 1) return 0;
    const half = slack / 2;
    return clampOffset(((origin - half) / half) * 100);
  };
  return { offsetX: axis(slackX, cropX), offsetY: axis(slackY, cropY) };
}

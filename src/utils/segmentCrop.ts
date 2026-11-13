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
  /** 锁上后拖动手柄只等比缩放。 */
  ratioLocked: boolean;
  /** 锁定时的宽高比（宽/高）。未锁定为 0。 */
  lockRatio: number;
}

export interface SegmentFileConfig {
  /** 当前正在编辑的比例 */
  ratio: string;
  /** 这个视频要输出的比例，顺序即界面顺序 */
  ratios: string[];
  /** 每个比例一份独立分段 */
  tracks: Record<string, SegmentPiece[]>;
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
/** 只在分段裁剪里生效，不和比例裁剪的比例列表共用。 */
const RATIO_KEY = "segment-crop-ratios";

export interface SegmentRatioSet {
  ratios: string[];
  active: string;
}

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

function cleanRatioList(values: unknown): string[] {
  if (!Array.isArray(values)) return [];
  const ratios: string[] = [];
  for (const item of values) {
    const ratio = String(item ?? "").trim();
    if (!parseRatio(ratio) || ratios.includes(ratio)) continue;
    ratios.push(ratio);
  }
  return ratios;
}

function migrateSegmentRatios(): SegmentRatioSet {
  const ratios: string[] = [];
  const push = (ratio: string) => {
    const text = ratio.trim();
    if (!parseRatio(text) || ratios.includes(text)) return;
    ratios.push(text);
  };
  push(loadSegmentGlobal().ratio);
  for (const raw of Object.values(readAll())) {
    const cfg = normalizeSegmentConfig(raw);
    cfg?.ratios.forEach(push);
  }
  if (ratios.length === 0) ratios.push("9:16");
  const preferred = loadSegmentGlobal().ratio.trim();
  return {
    ratios,
    active: ratios.includes(preferred) ? preferred : ratios[0],
  };
}

export function loadSegmentRatios(): SegmentRatioSet {
  try {
    const raw = localStorage.getItem(RATIO_KEY);
    if (raw) {
      const parsed = JSON.parse(raw) as Partial<SegmentRatioSet>;
      const ratios = cleanRatioList(parsed.ratios);
      if (ratios.length > 0) {
        const active = String(parsed.active ?? "").trim();
        return { ratios, active: ratios.includes(active) ? active : ratios[0] };
      }
    }
  } catch {
    /* ignore */
  }
  const migrated = migrateSegmentRatios();
  saveSegmentRatios(migrated);
  return migrated;
}

export function saveSegmentRatios(value: SegmentRatioSet) {
  const ratios = cleanRatioList(value.ratios);
  const next = ratios.length > 0 ? ratios : ["9:16"];
  const active = next.includes(value.active) ? value.active : next[0];
  localStorage.setItem(RATIO_KEY, JSON.stringify({ ratios: next, active }));
}

function readAll(): Record<string, unknown> {
  try {
    const raw = localStorage.getItem(CONFIG_KEY);
    if (!raw) return {};
    const parsed = JSON.parse(raw) as Record<string, unknown>;
    return parsed && typeof parsed === "object" ? parsed : {};
  } catch {
    return {};
  }
}

function normalizePiece(piece: SegmentPiece): SegmentPiece {
  return {
    ...piece,
    ratioLocked: !(piece.ratioLocked === false && Number(piece.lockRatio) > 0),
    lockRatio: Number(piece.lockRatio) > 0 ? Number(piece.lockRatio) : 0,
  };
}

/** 旧配置只有一个 ratio + segments，读出来收成多比例结构。 */
export function normalizeSegmentConfig(raw: unknown): SegmentFileConfig | null {
  if (!raw || typeof raw !== "object") return null;
  const obj = raw as {
    ratio?: string;
    segments?: SegmentPiece[];
    ratios?: string[];
    tracks?: Record<string, SegmentPiece[]>;
  };
  const tracks: Record<string, SegmentPiece[]> = {};
  if (obj.tracks && typeof obj.tracks === "object") {
    for (const [key, pieces] of Object.entries(obj.tracks)) {
      const ratio = key.trim();
      if (!parseRatio(ratio) || !Array.isArray(pieces) || pieces.length === 0) continue;
      tracks[ratio] = pieces.map((piece) => normalizePiece(piece));
    }
  }
  const legacyRatio = (obj.ratio || "").trim();
  if (
    Object.keys(tracks).length === 0 &&
    parseRatio(legacyRatio) &&
    Array.isArray(obj.segments) &&
    obj.segments.length > 0
  ) {
    tracks[legacyRatio] = obj.segments.map((piece) => normalizePiece(piece));
  }
  const ratios = (Array.isArray(obj.ratios) ? obj.ratios : Object.keys(tracks))
    .map((item) => String(item).trim())
    .filter((ratio, index, list) => parseRatio(ratio) && tracks[ratio] && list.indexOf(ratio) === index);
  for (const ratio of Object.keys(tracks)) {
    if (!ratios.includes(ratio)) ratios.push(ratio);
  }
  if (ratios.length === 0) return null;
  const active = parseRatio(legacyRatio) && ratios.includes(legacyRatio) ? legacyRatio : ratios[0];
  return { ratio: active, ratios, tracks };
}

export function loadSegmentConfig(path: string): SegmentFileConfig | null {
  return normalizeSegmentConfig(readAll()[path]);
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
    ratioLocked: true,
    lockRatio: 0,
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

function toExportPieces(pieces: SegmentPiece[]): SegmentExportPiece[] {
  return pieces.map((piece) => ({
    start: piece.start,
    end: piece.end,
    mode: piece.mode,
    offset_x: piece.offsetX,
    offset_y: piece.offsetY,
    crop_x: piece.cropX,
    crop_y: piece.cropY,
    crop_w: piece.cropW || 0,
    crop_h: piece.cropH || 0,
  }));
}

export function buildSegmentJobs(files: FileInfo[], _global: SegmentGlobal): SegmentExportJob[] {
  const ratios = loadSegmentRatios().ratios;
  return files
    .filter((file) => file.file_type === "video")
    .flatMap((file) => {
      const cfg = loadSegmentConfig(file.path);
      return ratios.map((ratio) => ({
        path: file.path,
        ratio,
        offset_x: 0,
        offset_y: 0,
        segments: toExportPieces(cfg?.tracks[ratio] || []),
      }));
    });
}

/** 复制一份分段到另一个比例。偏移和丢弃原样保留，位置框按目标比例重算。 */
export function cloneSegmentsForRatio(
  pieces: SegmentPiece[],
  sourceRatio: string,
  targetRatio: string,
  mediaW: number,
  mediaH: number,
): SegmentPiece[] {
  const target = parseRatio(targetRatio) || 0;
  return pieces.map((piece) => {
    const next = normalizePiece({ ...piece, id: newSegmentId() });
    if (next.mode !== "position" || !(target > 0) || mediaW < 16 || mediaH < 16) return next;
    const src = coverBox(mediaW, mediaH, sourceRatio, piece);
    if (!src) return next;
    const fitted = resizeBoxToRatio(mediaW, mediaH, src, target);
    next.cropX = fitted.cropX;
    next.cropY = fitted.cropY;
    next.cropW = fitted.cropW;
    next.cropH = fitted.cropH;
    if (next.ratioLocked) next.lockRatio = target;
    return next;
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

function evenClamp(value: number, min: number, max: number): number {
  const evenMax = Math.max(2, max & ~1);
  const evenMin = Math.min(evenMax, Math.max(2, min & ~1));
  let even = Math.round(value) & ~1;
  if (even < evenMin) even = evenMin;
  if (even > evenMax) even = evenMax;
  return even;
}

/** 按目标宽高比改裁切框。尽量保持中心，必要时缩小以留在画面内。 */
export function resizeBoxToRatio(
  mediaW: number,
  mediaH: number,
  box: { cropX: number; cropY: number; cropW: number; cropH: number },
  ratio: number,
): { cropX: number; cropY: number; cropW: number; cropH: number } {
  const maxW = Math.max(16, mediaW & ~1);
  const maxH = Math.max(16, mediaH & ~1);
  if (!(ratio > 0) || !Number.isFinite(ratio) || mediaW < 16 || mediaH < 16) {
    return { ...box };
  }
  const cx = box.cropX + box.cropW / 2;
  const cy = box.cropY + box.cropH / 2;
  let width = Math.max(16, box.cropW);
  let height = Math.max(16, box.cropH);
  const current = width / height;
  if (ratio > current) {
    const grownW = height * ratio;
    if (grownW <= maxW) width = grownW;
    else {
      width = maxW;
      height = width / ratio;
    }
  } else if (ratio < current) {
    const grownH = width / ratio;
    if (grownH <= maxH) height = grownH;
    else {
      height = maxH;
      width = height * ratio;
    }
  }
  let cropW = evenClamp(width, 16, maxW);
  let cropH = evenClamp(cropW / ratio, 16, maxH);
  if (Math.abs(cropW / cropH - ratio) / ratio > 0.02) {
    cropW = evenClamp(cropH * ratio, 16, maxW);
  }
  let cropX = Math.round(cx - cropW / 2);
  let cropY = Math.round(cy - cropH / 2);
  const maxX = Math.max(0, mediaW - cropW);
  const maxY = Math.max(0, mediaH - cropH);
  cropX = Math.min(maxX, Math.max(0, cropX));
  cropY = Math.min(maxY, Math.max(0, cropY));
  const evenX = cropX & ~1;
  const evenY = cropY & ~1;
  cropX = evenX + cropW <= mediaW ? evenX : maxX & ~1;
  cropY = evenY + cropH <= mediaH ? evenY : maxY & ~1;
  return { cropX, cropY, cropW, cropH };
}

type ScaleHandle = "n" | "s" | "e" | "w" | "nw" | "ne" | "sw" | "se";

/** 以起始框的比例缩放。对角钉住对角，四边钉住对边并在另一轴居中。 */
export function scaleLockedBox(
  handle: ScaleHandle,
  start: { cropX: number; cropY: number; cropW: number; cropH: number },
  dx: number,
  dy: number,
  ratio: number,
  mediaW: number,
  mediaH: number,
): { cropX: number; cropY: number; cropW: number; cropH: number } {
  if (!(start.cropW > 0) || !(start.cropH > 0)) return { ...start };
  const min = 16;
  const left = start.cropX;
  const top = start.cropY;
  const right = start.cropX + start.cropW;
  const bottom = start.cropY + start.cropH;
  const centerX = left + start.cropW / 2;
  const centerY = top + start.cropH / 2;

  let widthDelta = 0;
  let heightDelta = 0;
  if (handle.includes("e")) widthDelta = dx;
  if (handle.includes("w")) widthDelta = -dx;
  if (handle.includes("s")) heightDelta = dy;
  if (handle.includes("n")) heightDelta = -dy;

  const affectsW = handle.includes("e") || handle.includes("w");
  const affectsH = handle.includes("n") || handle.includes("s");
  const scaleW = (start.cropW + widthDelta) / start.cropW;
  const scaleH = (start.cropH + heightDelta) / start.cropH;
  let scale = 1;
  if (affectsW && affectsH) {
    scale = Math.abs(scaleW - 1) >= Math.abs(scaleH - 1) ? scaleW : scaleH;
  } else if (affectsW) {
    scale = scaleW;
  } else {
    scale = scaleH;
  }

  let maxScale = Number.POSITIVE_INFINITY;
  if (handle.includes("w")) maxScale = Math.min(maxScale, right / start.cropW);
  else if (handle.includes("e")) maxScale = Math.min(maxScale, (mediaW - left) / start.cropW);
  else {
    maxScale = Math.min(
      maxScale,
      (2 * centerX) / start.cropW,
      (2 * (mediaW - centerX)) / start.cropW,
    );
  }
  if (handle.includes("n")) maxScale = Math.min(maxScale, bottom / start.cropH);
  else if (handle.includes("s")) maxScale = Math.min(maxScale, (mediaH - top) / start.cropH);
  else {
    maxScale = Math.min(
      maxScale,
      (2 * centerY) / start.cropH,
      (2 * (mediaH - centerY)) / start.cropH,
    );
  }

  const minScale = Math.max(min / start.cropW, min / start.cropH);
  if (!Number.isFinite(maxScale) || maxScale <= 0) maxScale = minScale;
  scale = Math.min(Math.max(scale, Math.min(minScale, maxScale)), maxScale);

  const aspect = ratio > 0 ? ratio : start.cropW / start.cropH;
  let cropW = Math.max(2, Math.floor(start.cropW * scale));
  let cropH = Math.max(2, Math.round(cropW / aspect));
  if (cropH > mediaH || cropW > mediaW) {
    cropH = Math.max(2, Math.min(mediaH, cropH));
    cropW = Math.max(2, Math.round(cropH * aspect));
  }
  let cropX = left;
  let cropY = top;
  if (handle.includes("w")) cropX = right - cropW;
  else if (!handle.includes("e")) cropX = centerX - cropW / 2;
  if (handle.includes("n")) cropY = bottom - cropH;
  else if (!handle.includes("s")) cropY = centerY - cropH / 2;

  cropX = Math.min(mediaW - cropW, Math.max(0, Math.round(cropX)));
  cropY = Math.min(mediaH - cropH, Math.max(0, Math.round(cropY)));
  return { cropX, cropY, cropW, cropH };
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

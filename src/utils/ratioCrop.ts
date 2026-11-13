/**
 * 与 src-tauri/src/commands.rs 的 compute_cover_crop / offset_origin 保持一致。
 * 0 偏移走整数除法居中，避免预览框和实际导出差一两个像素。
 */

export interface CoverCrop {
  cropW: number;
  cropH: number;
  cropX: number;
  cropY: number;
}

function evenDim(v: number): number {
  return Math.max(2, v & ~1);
}

function offsetOrigin(slack: number, percent: number): number {
  if (slack <= 0) return 0;
  const pct = Number.isFinite(percent)
    ? Math.min(100, Math.max(-100, percent))
    : 0;
  if (pct === 0) return Math.floor(slack / 2);
  const half = slack / 2;
  const origin = Math.round(half + half * (pct / 100));
  if (origin <= 0) return 0;
  return Math.min(origin, slack);
}

export function parseRatio(ratio: string): number | null {
  const parts = ratio.split(":");
  if (parts.length !== 2) return null;
  const w = Number(parts[0]);
  const h = Number(parts[1]);
  if (!(w > 0) || !(h > 0) || !Number.isFinite(w) || !Number.isFinite(h)) {
    return null;
  }
  return w / h;
}

export function computeCoverCrop(
  width: number,
  height: number,
  targetRatio: number,
  offsetXPct: number,
  offsetYPct: number,
): CoverCrop {
  const videoRatio = width / height;
  let cropW: number;
  let cropH: number;
  let cropX: number;
  let cropY: number;
  if (targetRatio < videoRatio) {
    cropH = height;
    cropW = Math.min(width, Math.max(1, Math.trunc(height * targetRatio)));
    cropX = offsetOrigin(Math.max(0, width - cropW), offsetXPct);
    cropY = 0;
  } else {
    cropW = width;
    cropH = Math.min(height, Math.max(1, Math.trunc(width / targetRatio)));
    cropY = offsetOrigin(Math.max(0, height - cropH), offsetYPct);
    cropX = 0;
  }
  cropW = evenDim(cropW);
  cropH = evenDim(cropH);
  cropX = cropX & ~1;
  cropY = cropY & ~1;
  if (cropX + cropW > width) {
    cropX = evenDim(Math.max(0, width - cropW)) & ~1;
  }
  if (cropY + cropH > height) {
    cropY = evenDim(Math.max(0, height - cropH)) & ~1;
  }
  return { cropW, cropH, cropX, cropY };
}

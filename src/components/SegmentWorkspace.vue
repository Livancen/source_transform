<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from "vue";
import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import type { FileInfo } from "../types";
import { computeCoverCrop, parseRatio } from "../utils/ratioCrop";
import {
  clampOffset,
  clearSegmentConfig,
  cloneSegmentsForRatio,
  coverBox,
  formatBoxRatio,
  formatClock,
  fullSpanSegment,
  resizeBoxToRatio,
  scaleLockedBox,
  loadSegmentConfig,
  loadSegmentGlobal,
  loadSegmentRatios,
  newSegmentId,
  offsetsFromPosition,
  saveSegmentConfig,
  saveSegmentGlobal,
  saveSegmentRatios,
  segmentConfigPaths,
  type SegmentGlobal,
  type SegmentPiece,
} from "../utils/segmentCrop";
import FilePane from "./FilePane.vue";

const props = defineProps<{
  dir: string;
  files: FileInfo[];
  selectedPath: string;
  checkedPaths: Set<string>;
  allChecked: boolean;
  locked: boolean;
}>();

const emit = defineEmits<{
  selectDir: [];
  openFolder: [];
  selectFile: [file: FileInfo];
  toggleCheck: [path: string];
  toggleCheckAll: [];
  deleteFile: [file: FileInfo];
}>();

const globalOpts = ref<SegmentGlobal>(loadSegmentGlobal());
const initialRatios = loadSegmentRatios();
const configTick = ref(0);
const segments = ref<SegmentPiece[]>([]);
const fileRatios = ref<string[]>([...initialRatios.ratios]);
const tracks = ref<Record<string, SegmentPiece[]>>({});
const fileRatio = ref(initialRatios.active);
const ratioInput = ref("");
const ratioError = ref("");
const copyOpen = ref(false);
const saved = ref(false);
const selectedIds = ref<string[]>([]);
const duration = ref(0);
const mediaW = ref(0);
const mediaH = ref(0);
const loading = ref(false);
const loadError = ref("");
const currentTime = ref(0);
const playing = ref(false);
const playbackRate = ref(1);
const outputPreview = ref(false);
const videoRef = ref<HTMLVideoElement | null>(null);
const frameRef = ref<HTMLElement | null>(null);
const timelineRef = ref<HTMLElement | null>(null);
const speeds = [0.5, 1, 1.5, 2];
const ratioDraft = ref("");
const ratioEditing = ref(false);
let hydrating = false;

const selected = computed(
  () => props.files.find((file) => file.path === props.selectedPath) || null,
);
const markedPaths = computed(() => {
  configTick.value;
  return segmentConfigPaths();
});
const active = computed(() => {
  if (selectedIds.value.length !== 1) return null;
  return (
    segments.value.find((piece) => piece.id === selectedIds.value[0]) ?? null
  );
});
const multi = computed(() => selectedIds.value.length > 1);
const previewPiece = computed(() => {
  if (active.value) return active.value;
  const time = currentTime.value;
  return (
    segments.value.find((piece) => time >= piece.start && time < piece.end) ??
    segments.value[0] ??
    null
  );
});
const selectionContiguous = computed(() => {
  const indexes = segments.value
    .map((piece, index) => (selectedIds.value.includes(piece.id) ? index : -1))
    .filter((index) => index >= 0);
  if (indexes.length < 2) return false;
  return indexes[indexes.length - 1] - indexes[0] + 1 === indexes.length;
});
const activeIndex = computed(() =>
  segments.value.findIndex((piece) => piece.id === active.value?.id),
);
const otherRatios = computed(() =>
  fileRatios.value.filter((ratio) => ratio !== fileRatio.value),
);
const ratioCustom = computed(() =>
  Boolean(tracks.value[fileRatio.value]?.length),
);
const box = computed(() => {
  const piece = previewPiece.value;
  if (!piece || piece.mode === "drop" || !mediaW.value) return null;
  return coverBox(mediaW.value, mediaH.value, fileRatio.value, piece);
});
const boxRatioLabel = computed(() => {
  const piece = previewPiece.value;
  const rect = box.value;
  if (!piece || piece.mode !== "position" || !rect) return "";
  if (piece.ratioLocked && piece.lockRatio > 0) {
    return formatBoxRatio(Math.round(piece.lockRatio * 10000), 10000);
  }
  return formatBoxRatio(rect.cropW, rect.cropH);
});
const boxStyle = computed(() => {
  const rect = box.value;
  if (!rect || !mediaW.value || !mediaH.value) return {};
  return {
    left: `${(rect.cropX / mediaW.value) * 100}%`,
    top: `${(rect.cropY / mediaH.value) * 100}%`,
    width: `${(rect.cropW / mediaW.value) * 100}%`,
    height: `${(rect.cropH / mediaH.value) * 100}%`,
  };
});
const shade = computed(() => {
  const rect = box.value;
  if (!rect || !mediaW.value || !mediaH.value) return null;
  const x = (rect.cropX / mediaW.value) * 100;
  const y = (rect.cropY / mediaH.value) * 100;
  const w = (rect.cropW / mediaW.value) * 100;
  const h = (rect.cropH / mediaH.value) * 100;
  return {
    top: { height: `${y}%` },
    bottom: { top: `${y + h}%` },
    left: { top: `${y}%`, width: `${x}%`, height: `${h}%` },
    right: { top: `${y}%`, left: `${x + w}%`, height: `${h}%` },
  };
});
const ticks = computed(() => {
  if (!(duration.value > 0)) return [];
  const count = duration.value > 120 ? 6 : 4;
  return Array.from({ length: count + 1 }, (_, index) => ({
    left: (index / count) * 100,
    label: formatClock((duration.value * index) / count),
  }));
});
function pieceAtTime(time: number): SegmentPiece | null {
  const list = segments.value;
  if (!list.length) return null;
  const hit = list.find((piece) => time >= piece.start && time < piece.end);
  if (hit) return hit;
  const last = list[list.length - 1];
  return time >= last.end - 0.001 ? last : list[0];
}

function isKeptPiece(piece: SegmentPiece) {
  return piece.mode !== "drop" && piece.end - piece.start >= 0.08;
}

const outputAspect = computed(() => {
  const ratio = parseRatio(fileRatio.value);
  if (!ratio || mediaW.value < 2 || mediaH.value < 2) return null;
  const canon = computeCoverCrop(mediaW.value, mediaH.value, ratio, 0, 0);
  if (canon.cropW < 2 || canon.cropH < 2) return null;
  return canon;
});

const outputPiece = computed(() => pieceAtTime(currentTime.value));

const outputCrop = computed(() => {
  const piece = outputPiece.value;
  if (!outputPreview.value || !piece || !isKeptPiece(piece) || !mediaW.value)
    return null;
  return coverBox(mediaW.value, mediaH.value, fileRatio.value, piece);
});

const hasKeptSegment = computed(() =>
  segments.value.some((piece) => isKeptPiece(piece)),
);

const frameStyle = computed(() => {
  if (outputPreview.value && outputAspect.value) {
    return {
      aspectRatio: `${outputAspect.value.cropW} / ${outputAspect.value.cropH}`,
      height: "100%",
    };
  }
  if (mediaW.value && mediaH.value) {
    return { aspectRatio: `${mediaW.value} / ${mediaH.value}`, height: "100%" };
  }
  return { width: "100%" };
});

const outputVideoStyle = computed(() => {
  if (!outputPreview.value) return undefined;
  const rect = outputCrop.value;
  if (!rect || !mediaW.value || !mediaH.value)
    return { visibility: "hidden" as const };
  return {
    width: `${(mediaW.value / rect.cropW) * 100}%`,
    height: `${(mediaH.value / rect.cropH) * 100}%`,
    left: `${(-rect.cropX / rect.cropW) * 100}%`,
    top: `${(-rect.cropY / rect.cropH) * 100}%`,
  };
});

let skipUntil = 0;

function nextKeptStart(time: number): number | null {
  const list = [...segments.value].sort((a, b) => a.start - b.start);
  for (const piece of list) {
    if (piece.end <= time + 0.02) continue;
    if (!isKeptPiece(piece)) continue;
    if (time >= piece.start - 0.02 && time < piece.end) return null;
    return piece.start;
  }
  return null;
}

function skipDropped(video: HTMLVideoElement | null) {
  if (!video || !outputPreview.value || performance.now() < skipUntil) return;
  const time = video.currentTime || 0;
  const hit = pieceAtTime(time);
  if (!hit || isKeptPiece(hit)) return;
  const jump = nextKeptStart(time);
  if (jump == null) {
    video.pause();
    return;
  }
  if (Math.abs(video.currentTime - jump) < 0.03) return;
  skipUntil = performance.now() + 250;
  video.currentTime = jump;
  currentTime.value = jump;
}

function toggleOutputPreview() {
  outputPreview.value = !outputPreview.value;
  const video = videoRef.value;
  if (outputPreview.value && video && !video.paused) skipDropped(video);
}

function onPlay() {
  playing.value = true;
  if (outputPreview.value) skipDropped(videoRef.value);
}

function onTimeUpdate() {
  const video = videoRef.value;
  if (!video) return;
  let time = video.currentTime || 0;
  if (outputPreview.value && !video.paused && performance.now() >= skipUntil) {
    const hit = pieceAtTime(time);
    if (hit && !isKeptPiece(hit)) {
      const jump = nextKeptStart(time);
      if (jump == null) {
        video.pause();
      } else if (Math.abs(time - jump) >= 0.03) {
        skipUntil = performance.now() + 250;
        video.currentTime = jump;
        time = jump;
      }
    }
  }
  currentTime.value = time;
  if (video.paused || selectedIds.value.length > 1) return;
  const hit = pieceAtTime(time);
  if (!hit || selectedIds.value[0] === hit.id) return;
  selectOnly(hit.id);
}

function pieceFrame(piece: SegmentPiece) {
  const d = duration.value;
  if (!(d > 0)) return { left: "0%", width: "0%", maxWidth: "0%" };
  const start = Math.min(Math.max(piece.start, 0), d);
  const end = Math.min(Math.max(piece.end, start), d);
  const left = (start / d) * 100;
  const width = ((end - start) / d) * 100;
  return {
    left: `${left}%`,
    width: `${width}%`,
    maxWidth: `${width}%`,
  };
}

const playheadLeft = computed(() =>
  duration.value > 0 ? `${(currentTime.value / duration.value) * 100}%` : "0%",
);

watch(globalOpts, (value) => saveSegmentGlobal(value), { deep: true });

function centeredSpan(end: number) {
  return fullSpanSegment(end, { ...globalOpts.value, offsetX: 0, offsetY: 0 });
}

function rememberRatios() {
  saveSegmentRatios({ ratios: [...fileRatios.value], active: fileRatio.value });
}

watch(
  () => props.selectedPath,
  async (path) => {
    if (!path) {
      segments.value = [];
      saved.value = false;
      return;
    }
    const file = props.files.find((item) => item.path === path);
    if (!file) return;
    await openFile(file);
  },
  { immediate: true },
);

function touchConfig() {
  configTick.value += 1;
}

function snapshotTracks() {
  const active = fileRatio.value.trim();
  if (!active || segments.value.length === 0) return;
  tracks.value = {
    ...tracks.value,
    [active]: segments.value.map((piece) => ({ ...piece })),
  };
}

function persist() {
  const file = selected.value;
  if (!file || hydrating || !(duration.value > 0)) return;
  snapshotTracks();
  const ratios = Object.keys(tracks.value).filter(
    (ratio) => tracks.value[ratio]?.length,
  );
  if (ratios.length === 0) {
    clearSegmentConfig(file.path);
    saved.value = false;
    touchConfig();
    return;
  }
  saveSegmentConfig(file.path, {
    ratio: fileRatio.value,
    ratios,
    tracks: Object.fromEntries(
      ratios.map((ratio) => [
        ratio,
        tracks.value[ratio].map((piece) => ({ ...piece })),
      ]),
    ),
  });
  saved.value = true;
  touchConfig();
}

function applyTrack(ratio: string) {
  const pieces = tracks.value[ratio];
  segments.value = pieces?.length
    ? pieces.map((piece) => ({ ...piece }))
    : [centeredSpan(duration.value || 1)];
  const time = currentTime.value;
  const hit =
    segments.value.find((piece) => time >= piece.start && time < piece.end) ||
    segments.value[0];
  selectOnly(hit?.id || "");
}

async function openFile(file: FileInfo) {
  hydrating = true;
  loading.value = true;
  loadError.value = "";
  duration.value = 0;
  mediaW.value = 0;
  mediaH.value = 0;
  currentTime.value = 0;
  const cfg = loadSegmentConfig(file.path);
  saved.value = !!cfg;
  copyOpen.value = false;
  tracks.value = cfg
    ? Object.fromEntries(
        cfg.ratios.map((ratio) => [
          ratio,
          cfg.tracks[ratio].map((piece) => ({ ...piece })),
        ]),
      )
    : {};
  try {
    const dimensions = await invoke<[number, number]>("get_video_dimensions", {
      videoPath: file.path,
    });
    mediaW.value = dimensions[0];
    mediaH.value = dimensions[1];
  } catch (error) {
    loadError.value = String(error);
  } finally {
    loading.value = false;
    hydrating = false;
  }
  applyTrack(fileRatio.value);
}

function onVideoMeta() {
  const video = videoRef.value;
  if (!video || !Number.isFinite(video.duration) || video.duration <= 0) return;
  duration.value = video.duration;
  video.playbackRate = playbackRate.value;
  if (!tracks.value[fileRatio.value]?.length) {
    const only = segments.value[0];
    if (
      segments.value.length === 1 &&
      only?.start === 0 &&
      only.mode === "offset" &&
      only.end <= 1.05
    ) {
      hydrating = true;
      segments.value = [centeredSpan(video.duration)];
      selectOnly(segments.value[0].id);
      hydrating = false;
    }
  }
}

function selectOnly(id: string) {
  selectedIds.value = id ? [id] : [];
}

function isSelected(id: string) {
  return selectedIds.value.includes(id);
}

function modeName(mode: SegmentPiece["mode"]) {
  if (mode === "drop") return "不导出";
  if (mode === "position") return "拖动位置";
  return "按偏移";
}

function onClipClick(id: string, event: MouseEvent) {
  if (event.ctrlKey || event.metaKey) {
    const next = new Set(selectedIds.value);
    if (next.has(id)) next.delete(id);
    else next.add(id);
    selectedIds.value = next.size ? [...next] : [id];
    return;
  }
  selectOnly(id);
  const piece = segments.value.find((item) => item.id === id);
  const video = videoRef.value;
  if (!piece || !video) return;
  if (
    video.currentTime < piece.start ||
    video.currentTime >= piece.end - 0.01
  ) {
    video.currentTime = piece.start;
  }
}

function updateActive(patch: Partial<SegmentPiece>) {
  const index = activeIndex.value;
  if (index < 0) return;
  const prev = segments.value[index];
  const next = { ...prev, ...patch };
  if (patch.mode === "position" && prev.mode !== "position" && mediaW.value) {
    const rect = computeCoverCrop(
      mediaW.value,
      mediaH.value,
      parseRatio(fileRatio.value) || 1,
      next.offsetX,
      next.offsetY,
    );
    next.cropX = rect.cropX;
    next.cropY = rect.cropY;
    next.cropW = rect.cropW;
    next.cropH = rect.cropH;
  }
  if (patch.mode === "offset" && prev.mode === "position" && mediaW.value) {
    const offsets = offsetsFromPosition(
      mediaW.value,
      mediaH.value,
      fileRatio.value,
      next.cropX,
      next.cropY,
    );
    next.offsetX = offsets.offsetX;
    next.offsetY = offsets.offsetY;
  }
  segments.value.splice(index, 1, next);
  persist();
}

function switchRatio(ratio: string) {
  if (
    props.locked ||
    ratio === fileRatio.value ||
    !fileRatios.value.includes(ratio)
  )
    return;
  fileRatio.value = ratio;
  rememberRatios();
  copyOpen.value = false;
  if (!selected.value) return;
  hydrating = true;
  applyTrack(ratio);
  hydrating = false;
}

function addFileRatio() {
  ratioError.value = "";
  const value = ratioInput.value.trim();
  if (!parseRatio(value)) {
    ratioError.value = "格式应为 W:H，如 1:1";
    return;
  }
  if (fileRatios.value.includes(value)) {
    ratioError.value = "该比例已存在";
    return;
  }
  fileRatios.value = [...fileRatios.value, value];
  ratioInput.value = "";
  switchRatio(value);
}

function removeFileRatio(ratio: string) {
  if (props.locked || fileRatios.value.length <= 1) return;
  const nextRatios = fileRatios.value.filter((item) => item !== ratio);
  fileRatios.value = nextRatios;
  if (fileRatio.value === ratio) {
    fileRatio.value = nextRatios[0];
    if (selected.value) {
      hydrating = true;
      applyTrack(fileRatio.value);
      hydrating = false;
    }
  }
  rememberRatios();
}

function copyActiveTo(target: string) {
  if (props.locked) return;
  const targets = target === "*" ? otherRatios.value : [target];
  if (!targets.length) return;
  snapshotTracks();
  const next = { ...tracks.value };
  for (const ratio of targets) {
    next[ratio] = cloneSegmentsForRatio(
      segments.value,
      fileRatio.value,
      ratio,
      mediaW.value,
      mediaH.value,
    );
  }
  tracks.value = next;
  copyOpen.value = false;
  persist();
}

function onRatioFocus() {
  ratioEditing.value = true;
  ratioDraft.value = boxRatioLabel.value;
}

function commitBoxRatio(input: HTMLInputElement) {
  ratioEditing.value = false;
  const rect = box.value;
  const ratio = parseRatio(input.value.trim());
  if (
    !ratio ||
    !rect ||
    !mediaW.value ||
    active.value?.mode !== "position" ||
    multi.value
  ) {
    input.value = boxRatioLabel.value;
    ratioDraft.value = boxRatioLabel.value;
    return;
  }
  const next = resizeBoxToRatio(mediaW.value, mediaH.value, rect, ratio);
  const unchanged =
    next.cropX === rect.cropX &&
    next.cropY === rect.cropY &&
    next.cropW === rect.cropW &&
    next.cropH === rect.cropH;
  if (!unchanged) {
    updateActive({
      cropX: next.cropX,
      cropY: next.cropY,
      cropW: next.cropW,
      cropH: next.cropH,
      ...(active.value?.ratioLocked ? { lockRatio: ratio } : {}),
    });
  }
  const label = formatBoxRatio(next.cropW, next.cropH);
  ratioDraft.value = label;
  input.value = label;
}

function toggleRatioLock() {
  const piece = active.value;
  const rect = box.value;
  if (!piece || piece.mode !== "position" || multi.value || !rect) return;
  if (piece.ratioLocked) {
    updateActive({ ratioLocked: false });
    return;
  }
  updateActive({
    ratioLocked: true,
    lockRatio: rect.cropH > 0 ? rect.cropW / rect.cropH : 0,
  });
}

function splitAtPlayhead() {
  const piece = active.value;
  const time = currentTime.value;
  if (!piece || time <= piece.start + 0.15 || time >= piece.end - 0.15) return;
  const right: SegmentPiece = { ...piece, id: newSegmentId(), start: time };
  const left: SegmentPiece = { ...piece, end: time };
  const index = activeIndex.value;
  segments.value.splice(index, 1, left, right);
  selectOnly(right.id);
  persist();
}

function dropActive() {
  const ids = selectedIds.value.length
    ? selectedIds.value
    : active.value
      ? [active.value.id]
      : [];
  if (!ids.length) return;
  const picked = segments.value.filter((piece) => ids.includes(piece.id));
  const restore =
    picked.length > 0 && picked.every((piece) => piece.mode === "drop");
  segments.value = segments.value.map((piece) =>
    ids.includes(piece.id)
      ? { ...piece, mode: restore ? "offset" : "drop" }
      : piece,
  );
  persist();
}

function mergeWithPrev() {
  if (multi.value) {
    if (!selectionContiguous.value) return;
    const indexes = segments.value
      .map((piece, index) =>
        selectedIds.value.includes(piece.id) ? index : -1,
      )
      .filter((index) => index >= 0);
    const first = segments.value[indexes[0]];
    const last = segments.value[indexes[indexes.length - 1]];
    const merged = { ...first, end: last.end };
    segments.value.splice(indexes[0], indexes.length, merged);
    selectOnly(merged.id);
    persist();
    return;
  }
  const index = activeIndex.value;
  if (index <= 0) return;
  const prev = segments.value[index - 1];
  const cur = segments.value[index];
  segments.value.splice(index - 1, 2, { ...prev, end: cur.end });
  selectOnly(prev.id);
  persist();
}

function resetFile() {
  const file = selected.value;
  if (!file) return;
  clearSegmentConfig(file.path);
  saved.value = false;
  copyOpen.value = false;
  hydrating = true;
  tracks.value = {};
  segments.value = [centeredSpan(duration.value || 1)];
  selectOnly(segments.value[0].id);
  hydrating = false;
  touchConfig();
}

function onBoundaryDown(index: number, event: PointerEvent) {
  if (props.locked) return;
  const bar = timelineRef.value;
  if (!bar || duration.value <= 0) return;
  event.preventDefault();
  event.stopPropagation();
  const rect = bar.getBoundingClientRect();
  const move = (ev: PointerEvent) => {
    const ratio = Math.min(
      1,
      Math.max(0, (ev.clientX - rect.left) / rect.width),
    );
    const time = ratio * duration.value;
    const left = segments.value[index];
    const right = segments.value[index + 1];
    if (!left || !right) return;
    const next = Math.min(right.end - 0.2, Math.max(left.start + 0.2, time));
    left.end = Number(next.toFixed(3));
    right.start = left.end;
  };
  const prevCursor = document.body.style.cursor;
  document.body.style.cursor = "col-resize";
  const up = () => {
    document.body.style.cursor = prevCursor;
    window.removeEventListener("pointermove", move);
    persist();
  };
  window.addEventListener("pointermove", move);
  window.addEventListener("pointerup", up, { once: true });
}

let stopScrub: (() => void) | null = null;

function applyTimelineTime(clientX: number, selectClip: boolean) {
  const bar = timelineRef.value;
  const video = videoRef.value;
  if (!bar || !video || !(duration.value > 0)) return;
  const rect = bar.getBoundingClientRect();
  const time = Math.min(
    duration.value,
    Math.max(
      0,
      ((clientX - rect.left) / Math.max(1, rect.width)) * duration.value,
    ),
  );
  currentTime.value = time;
  if (Math.abs(video.currentTime - time) > 0.01) video.currentTime = time;
  if (!selectClip) return;
  const hit =
    segments.value.find((piece) => time >= piece.start && time < piece.end) ||
    segments.value[segments.value.length - 1];
  if (hit) selectOnly(hit.id);
}

function seekFromTimeline(event: PointerEvent) {
  if (!(duration.value > 0)) return;
  event.preventDefault();
  const bar = timelineRef.value;
  const fromPlayhead =
    event.currentTarget instanceof HTMLElement &&
    event.currentTarget.classList.contains("seg-playhead");
  let grab = 0;
  if (fromPlayhead && bar) {
    const rect = bar.getBoundingClientRect();
    const center =
      rect.left + (currentTime.value / duration.value) * rect.width;
    grab = event.clientX - center;
  }
  const selectClip = !event.ctrlKey && !event.metaKey;
  applyTimelineTime(event.clientX - grab, selectClip);
  stopScrub?.();
  const prevCursor = document.body.style.cursor;
  document.body.style.cursor = "grabbing";
  const move = (ev: PointerEvent) =>
    applyTimelineTime(ev.clientX - grab, selectClip);
  const up = () => {
    document.body.style.cursor = prevCursor;
    window.removeEventListener("pointermove", move);
    window.removeEventListener("pointerup", up);
    stopScrub = null;
  };
  stopScrub = up;
  window.addEventListener("pointermove", move);
  window.addEventListener("pointerup", up);
}

type BoxHandle = "move" | "n" | "s" | "e" | "w" | "nw" | "ne" | "sw" | "se";
const boxHandles: BoxHandle[] = ["nw", "n", "ne", "e", "se", "s", "sw", "w"];
let drag: {
  handle: BoxHandle;
  x: number;
  y: number;
  cropX: number;
  cropY: number;
  cropW: number;
  cropH: number;
  scale: number;
} | null = null;

function onBoxDown(event: PointerEvent, handle: BoxHandle) {
  const piece = active.value;
  const rect = box.value;
  const frame = frameRef.value;
  if (
    !piece ||
    piece.mode !== "position" ||
    !rect ||
    !frame ||
    props.locked ||
    multi.value
  )
    return;
  event.preventDefault();
  event.stopPropagation();
  const bounds = frame.getBoundingClientRect();
  const scale = mediaW.value / Math.max(1, bounds.width);
  drag = {
    handle,
    x: event.clientX,
    y: event.clientY,
    cropX: rect.cropX,
    cropY: rect.cropY,
    cropW: rect.cropW,
    cropH: rect.cropH,
    scale,
  };
  window.addEventListener("pointermove", onBoxMove);
  window.addEventListener("pointerup", onBoxUp);
}

function onBoxMove(event: PointerEvent) {
  if (!drag) return;
  const dx = (event.clientX - drag.x) * drag.scale;
  const dy = (event.clientY - drag.y) * drag.scale;
  let cropX = drag.cropX;
  let cropY = drag.cropY;
  let cropW = drag.cropW;
  let cropH = drag.cropH;
  const min = 16;
  const handle = drag.handle;
  if (handle === "move") {
    cropX = Math.round(Math.min(mediaW.value - cropW, Math.max(0, cropX + dx)));
    cropY = Math.round(Math.min(mediaH.value - cropH, Math.max(0, cropY + dy)));
  } else if (active.value?.ratioLocked) {
    const scaled = scaleLockedBox(
      handle,
      drag,
      dx,
      dy,
      active.value?.lockRatio || drag.cropW / drag.cropH,
      mediaW.value,
      mediaH.value,
    );
    cropX = scaled.cropX;
    cropY = scaled.cropY;
    cropW = scaled.cropW;
    cropH = scaled.cropH;
  } else {
    if (handle.includes("e"))
      cropW = Math.round(
        Math.min(mediaW.value - cropX, Math.max(min, cropW + dx)),
      );
    if (handle.includes("s"))
      cropH = Math.round(
        Math.min(mediaH.value - cropY, Math.max(min, cropH + dy)),
      );
    if (handle.includes("w")) {
      const nextX = Math.round(
        Math.min(cropX + cropW - min, Math.max(0, cropX + dx)),
      );
      cropW = cropW - (nextX - cropX);
      cropX = nextX;
    }
    if (handle.includes("n")) {
      const nextY = Math.round(
        Math.min(cropY + cropH - min, Math.max(0, cropY + dy)),
      );
      cropH = cropH - (nextY - cropY);
      cropY = nextY;
    }
  }
  const index = activeIndex.value;
  if (index < 0) return;
  segments.value[index] = {
    ...segments.value[index],
    cropX,
    cropY,
    cropW,
    cropH,
  };
}

function onBoxUp() {
  drag = null;
  window.removeEventListener("pointermove", onBoxMove);
  window.removeEventListener("pointerup", onBoxUp);
  persist();
}

function togglePlay() {
  const video = videoRef.value;
  if (!video) return;
  if (video.paused) void video.play();
  else video.pause();
}

function onScrub(value: number) {
  const video = videoRef.value;
  currentTime.value = value;
  if (video) video.currentTime = value;
}

function setRate(rate: number) {
  playbackRate.value = rate;
  if (videoRef.value) videoRef.value.playbackRate = rate;
}

onBeforeUnmount(() => {
  onBoxUp();
  stopScrub?.();
});
</script>

<template>
  <div class="flex-1 min-h-0 flex flex-col bg-bg0">
    <div
      class="shrink-0 px-12px py-6px border-b border-border bg-bg1 flex flex-wrap items-center gap-6px"
    >
      <span class="text-11px color-t2 shrink-0">输出比例</span>
      <span class="text-10px color-t3 shrink-0"
        >仅分段裁剪，勾选的素材都会各出一条</span
      >
      <button
        v-for="ratio in fileRatios"
        :key="ratio"
        type="button"
        class="inline-flex items-center gap-4px h-26px px-8px rounded-full border text-12px font-mono"
        :class="
          ratio === fileRatio
            ? 'bg-secondary-soft border-secondary/30 color-secondary font-600'
            : 'bg-bg0 border-border color-t2'
        "
        :disabled="locked"
        @click="switchRatio(ratio)"
      >
        {{ ratio }}
        <span
          v-if="fileRatios.length > 1"
          class="color-t3 hover:color-danger"
          @click.stop="removeFileRatio(ratio)"
          >×</span
        >
      </button>
      <input
        class="field w-72px! h-26px! text-12px! font-mono"
        :value="ratioInput"
        placeholder="W:H"
        :disabled="locked"
        @input="ratioInput = ($event.target as HTMLInputElement).value"
        @keyup.enter="addFileRatio"
      />
      <button
        class="tb-btn h-26px! px-8px! text-11px!"
        type="button"
        :disabled="locked"
        @click="addFileRatio"
      >
        添加
      </button>
      <span v-if="ratioError" class="text-11px color-danger">{{
        ratioError
      }}</span>
    </div>
    <div
      class="flex-1 min-h-0 grid"
      style="grid-template-columns: minmax(280px, 42%) 6px minmax(0, 1fr)"
    >
      <FilePane
        kind="input"
        :dir="dir"
        :files="files"
        :selected-path="selectedPath"
        selectable
        isolate-check
        :checked-paths="checkedPaths"
        :all-checked="allChecked"
        :locked="locked"
        :marked-paths="markedPaths"
        @select-dir="emit('selectDir')"
        @open-folder="emit('openFolder')"
        @select-file="emit('selectFile', $event)"
        @toggle-check="emit('toggleCheck', $event)"
        @toggle-check-all="emit('toggleCheckAll')"
        @delete-file="emit('deleteFile', $event)"
      />
      <div class="bg-bg0 border-l border-r border-border"></div>

      <section class="min-w-0 min-h-0 bg-bg1 flex flex-col">
        <div
          v-if="!selected"
          class="m-auto text-13px color-t3 text-center px-24px leading-relaxed"
        >
          左侧选择视频。下面的时间轴用来分段，选中一段后再改这一段怎么裁。
        </div>
        <template v-else>
          <div
            class="shrink-0 min-h-40px px-12px py-6px flex flex-wrap items-center gap-8px border-b border-border"
          >
            <div
              class="min-w-0 flex-1 truncate text-13px font-600"
              :title="selected.name"
            >
              {{ selected.name }}
            </div>
            <span
              v-if="ratioCustom"
              class="text-10px color-secondary font-600 shrink-0"
              >此比例已分段</span
            >
            <span v-else class="text-10px color-t3 shrink-0">此比例整段</span>
            <div v-if="fileRatios.length > 1" class="relative">
              <button
                class="tb-btn h-26px! px-8px! text-11px!"
                type="button"
                :disabled="locked"
                @click="copyOpen = !copyOpen"
              >
                复制此配置
              </button>
              <div
                v-if="copyOpen"
                class="absolute right-0 top-30px z-20 min-w-140px py-4px rounded-4px border border-border bg-bg1 shadow-fluent"
              >
                <button
                  v-for="ratio in otherRatios"
                  :key="ratio"
                  type="button"
                  class="block w-full px-10px py-6px text-left text-12px font-mono color-t1 bg-transparent border-none cursor-pointer hover:bg-bg0"
                  @click="copyActiveTo(ratio)"
                >
                  到 {{ ratio }}
                </button>
                <button
                  v-if="otherRatios.length > 1"
                  type="button"
                  class="block w-full px-10px py-6px text-left text-12px color-t1 bg-transparent border-none cursor-pointer hover:bg-bg0"
                  @click="copyActiveTo('*')"
                >
                  到其余全部
                </button>
              </div>
            </div>
            <button
              class="tb-btn h-26px! px-8px! text-11px!"
              type="button"
              :disabled="!saved || locked"
              @click="resetFile"
            >
              恢复整段
            </button>
          </div>

          <div v-if="loading" class="m-auto text-12px color-t3">
            读取视频信息…
          </div>
          <div v-else-if="loadError" class="m-auto text-12px color-danger">
            {{ loadError }}
          </div>
          <template v-else>
            <div
              class="flex-1 min-h-0 grid place-items-center bg-[#141414] p-12px"
            >
              <div
                ref="frameRef"
                class="relative max-w-full max-h-full bg-black"
                :class="outputPreview ? 'overflow-hidden' : ''"
                :style="frameStyle"
              >
                <video
                  ref="videoRef"
                  :key="selected.path"
                  class="absolute object-fill"
                  :class="outputPreview ? '' : 'inset-0 w-full h-full'"
                  :style="outputVideoStyle"
                  :src="convertFileSrc(selected.path)"
                  playsinline
                  @loadedmetadata="onVideoMeta"
                  @timeupdate="onTimeUpdate"
                  @play="onPlay"
                  @pause="playing = false"
                />
                <div
                  v-if="outputPreview"
                  class="absolute left-8px top-8px z-10 px-6px h-18px leading-18px rounded-2px bg-black/70 text-white text-10px pointer-events-none"
                >
                  成片预览
                </div>
                <template v-if="shade && !outputPreview">
                  <div
                    class="absolute left-0 right-0 top-0 bg-black/55 pointer-events-none"
                    :style="shade.top"
                  ></div>
                  <div
                    class="absolute left-0 right-0 bottom-0 bg-black/55 pointer-events-none"
                    :style="shade.bottom"
                  ></div>
                  <div
                    class="absolute left-0 bg-black/55 pointer-events-none"
                    :style="shade.left"
                  ></div>
                  <div
                    class="absolute right-0 bg-black/55 pointer-events-none"
                    :style="shade.right"
                  ></div>
                </template>
                <div
                  v-if="box && !outputPreview"
                  class="absolute border-1 border-dotted border-white shadow-[0_0_0_1px_rgba(0,0,0,.45)]"
                  :class="
                    playing || active?.mode !== 'position' || multi
                      ? 'pointer-events-none'
                      : 'cursor-move'
                  "
                  :style="boxStyle"
                  @pointerdown="onBoxDown($event, 'move')"
                >
                  <span
                    class="absolute left-4px top-4px px-4px h-16px leading-16px rounded-2px bg-black/60 text-white text-10px pointer-events-none"
                  >
                    {{ previewPiece ? modeName(previewPiece.mode) : "" }}
                  </span>
                  <span
                    v-if="boxRatioLabel"
                    class="absolute inset-0 grid place-items-center pointer-events-none"
                  >
                    <span
                      v-if="!playing && active?.mode === 'position' && !multi"
                      class="pointer-events-auto inline-flex items-center gap-2px"
                    >
                      <input
                        class="w-76px h-20px px-4px rounded-2px border border-white/40 bg-black/75 text-white text-11px font-600 font-mono text-center outline-none"
                        :value="ratioEditing ? ratioDraft : boxRatioLabel"
                        title="输入比例，如 9:16，回车确认"
                        @pointerdown.stop
                        @mousedown.stop
                        @click.stop
                        @focus="onRatioFocus"
                        @input="
                          ratioDraft = ($event.target as HTMLInputElement).value
                        "
                        @keydown.enter.prevent="
                          ($event.target as HTMLInputElement).blur()
                        "
                        @blur="
                          commitBoxRatio($event.target as HTMLInputElement)
                        "
                      />
                      <button
                        type="button"
                        class="w-20px h-20px grid place-items-center rounded-2px border"
                        :class="
                          active.ratioLocked
                            ? 'bg-secondary border-secondary text-white'
                            : 'bg-black/75 border-white/40 text-white'
                        "
                        :title="
                          active.ratioLocked
                            ? '已锁定比例，只能等比缩放'
                            : '锁定比例'
                        "
                        @pointerdown.stop
                        @mousedown.stop
                        @click.stop="toggleRatioLock"
                      >
                        <svg
                          viewBox="0 0 24 24"
                          class="w-12px h-12px"
                          fill="none"
                          stroke="currentColor"
                          stroke-width="2"
                        >
                          <rect x="5" y="11" width="14" height="10" rx="2" />
                          <path
                            v-if="active.ratioLocked"
                            d="M8 11V8a4 4 0 0 1 8 0v3"
                          />
                          <path v-else d="M8 11V8a4 4 0 0 1 7.2-2.2" />
                        </svg>
                      </button>
                    </span>
                    <span
                      v-else
                      class="px-6px h-18px leading-18px rounded-2px bg-black/70 text-white text-11px font-600 font-mono"
                    >
                      {{ boxRatioLabel }}
                    </span>
                  </span>
                  <i
                    v-for="handle in boxHandles"
                    v-show="!playing && active?.mode === 'position' && !multi"
                    :key="handle"
                    class="crop-handle"
                    :class="'is-' + handle"
                    @pointerdown="onBoxDown($event, handle)"
                  ></i>
                </div>
                <div
                  v-else-if="
                    outputPreview && outputPiece && !isKeptPiece(outputPiece)
                  "
                  class="absolute inset-0 grid place-items-center bg-black/55 text-white text-14px font-600 pointer-events-none"
                >
                  {{ hasKeptSegment ? "这段不进入成片" : "没有可预览的片段" }}
                </div>
                <div
                  v-else-if="!outputPreview && active?.mode === 'drop'"
                  class="absolute inset-0 grid place-items-center bg-black/55 text-white text-14px font-600 pointer-events-none"
                >
                  这段不导出
                </div>
              </div>
            </div>

            <div
              class="shrink-0 h-40px px-12px flex items-center gap-8px border-t border-border bg-bg1"
            >
              <button
                class="play-btn"
                type="button"
                :title="playing ? '暂停' : '播放'"
                @click="togglePlay"
              >
                <svg
                  v-if="playing"
                  viewBox="0 0 24 24"
                  class="w-14px h-14px"
                  fill="currentColor"
                >
                  <rect x="6" y="5" width="4" height="14" rx="1" />
                  <rect x="14" y="5" width="4" height="14" rx="1" />
                </svg>
                <svg
                  v-else
                  viewBox="0 0 24 24"
                  class="w-14px h-14px"
                  fill="currentColor"
                >
                  <polygon points="8,5 19,12 8,19" />
                </svg>
              </button>
              <span class="text-11px color-t2 font-mono tabular-nums shrink-0">
                {{ formatClock(currentTime) }} / {{ formatClock(duration) }}
              </span>
              <input
                class="flex-1 min-w-80px accent-secondary"
                type="range"
                min="0"
                :max="duration || 0"
                step="0.05"
                :value="currentTime"
                :disabled="!duration"
                @input="
                  onScrub(Number(($event.target as HTMLInputElement).value))
                "
              />
              <button
                v-for="rate in speeds"
                :key="rate"
                type="button"
                class="h-22px px-6px rounded-4px border text-10px font-600 cursor-pointer"
                :class="
                  playbackRate === rate
                    ? 'bg-secondary color-white border-secondary'
                    : 'bg-bg1 color-t2 border-border'
                "
                @click="setRate(rate)"
              >
                {{ rate }}x
              </button>
            </div>

            <div
              v-if="multi"
              class="shrink-0 px-12px py-10px border-t border-border bg-bg2 text-12px color-t2"
            >
              已选 {{ selectedIds.length }} 段。按住 Ctrl
              继续点选。此时只能删除或合并，合并要求选中的段彼此相连。
            </div>
            <div
              v-else-if="active"
              class="shrink-0 px-12px py-8px border-t border-border bg-bg2 flex flex-col gap-8px"
            >
              <div class="flex items-center gap-8px">
                <span class="text-12px font-600 color-t1"
                  >第 {{ activeIndex + 1 }} 段</span
                >
                <span class="text-11px color-t3 font-mono">
                  {{ formatClock(active.start) }} –
                  {{ formatClock(active.end) }}
                </span>
                <span v-if="box" class="text-11px color-t3 font-mono ml-auto">
                  {{ box.cropW }}×{{ box.cropH }}
                </span>
              </div>
              <div class="flex flex-wrap items-center gap-8px">
                <div
                  class="inline-flex p-2px rounded-4px bg-bg0 border border-border"
                >
                  <button
                    v-for="mode in ['offset', 'position', 'drop'] as const"
                    :key="mode"
                    type="button"
                    class="h-26px px-10px rounded-2px border-none text-12px cursor-pointer"
                    :class="
                      active.mode === mode
                        ? 'bg-bg1 color-t1 font-600 shadow-fluent-sm'
                        : 'bg-transparent color-t2'
                    "
                    :disabled="locked"
                    @click="
                      mode === 'drop'
                        ? active.mode === 'drop'
                          ? updateActive({ mode: 'offset' })
                          : updateActive({ mode: 'drop' })
                        : updateActive({ mode })
                    "
                  >
                    {{ modeName(mode) }}
                  </button>
                </div>
                <template v-if="active.mode === 'offset'">
                  <label class="inline-flex items-center gap-6px">
                    <span class="text-11px color-t2 w-28px">水平</span>
                    <input
                      class="w-120px accent-secondary"
                      type="range"
                      min="-100"
                      max="100"
                      :value="active.offsetX"
                      @input="
                        updateActive({
                          offsetX: clampOffset(
                            Number(($event.target as HTMLInputElement).value),
                          ),
                        })
                      "
                    />
                    <span class="text-11px color-t3 w-28px font-mono">{{
                      active.offsetX
                    }}</span>
                  </label>
                  <label class="inline-flex items-center gap-6px">
                    <span class="text-11px color-t2 w-28px">垂直</span>
                    <input
                      class="w-120px accent-secondary"
                      type="range"
                      min="-100"
                      max="100"
                      :value="active.offsetY"
                      @input="
                        updateActive({
                          offsetY: clampOffset(
                            Number(($event.target as HTMLInputElement).value),
                          ),
                        })
                      "
                    />
                    <span class="text-11px color-t3 w-28px font-mono">{{
                      active.offsetY
                    }}</span>
                  </label>
                </template>
                <span
                  v-else-if="active.mode === 'position'"
                  class="text-11px color-t3"
                  >拖动白框移动，拖四边或四角改大小，框中间可输入比例。默认锁定，只能等比缩放；点锁可以解开。刚切换时与比例裁剪的框重合。</span
                >
                <span v-else class="text-11px color-t3"
                  >播放到这里会跳过，不进入成片。</span
                >
              </div>
            </div>

            <div
              class="seg-dock shrink-0 border-t border-border bg-[#1b1b1b] text-white"
            >
              <div class="h-36px px-8px flex items-center gap-6px">
                <button
                  class="play-btn play-btn-dark"
                  type="button"
                  :title="playing ? '暂停' : '播放'"
                  @click="togglePlay"
                >
                  <svg
                    v-if="playing"
                    viewBox="0 0 24 24"
                    class="w-14px h-14px"
                    fill="currentColor"
                  >
                    <rect x="6" y="5" width="4" height="14" />
                    <rect x="14" y="5" width="4" height="14" />
                  </svg>
                  <svg
                    v-else
                    viewBox="0 0 24 24"
                    class="w-14px h-14px"
                    fill="currentColor"
                  >
                    <polygon points="8,5 19,12 8,19" />
                  </svg>
                </button>
                <button
                  class="seg-tool"
                  type="button"
                  :disabled="locked || multi"
                  @click="splitAtPlayhead"
                >
                  分割
                </button>
                <button
                  class="seg-tool"
                  type="button"
                  :disabled="locked || selectedIds.length === 0"
                  @click="dropActive"
                >
                  {{
                    selectedIds.length &&
                    segments
                      .filter((piece) => selectedIds.includes(piece.id))
                      .every((piece) => piece.mode === "drop")
                      ? "恢复"
                      : "删除"
                  }}
                </button>
                <button
                  class="seg-tool"
                  type="button"
                  :disabled="
                    locked || (multi ? !selectionContiguous : activeIndex <= 0)
                  "
                  @click="mergeWithPrev"
                >
                  {{ multi ? "合并" : "与上段合并" }}
                </button>
                <button
                  class="seg-tool"
                  type="button"
                  :class="{ 'is-on': outputPreview }"
                  :disabled="!mediaW || !duration"
                  title="按成片比例预览裁切，不重新编码"
                  @click="toggleOutputPreview"
                >
                  {{ outputPreview ? "退出预览" : "预览" }}
                </button>
                <span class="ml-auto text-10px color-white/50"
                  >拖红线移动时间 · 悬停边缘改时长 · Ctrl 点选</span
                >
              </div>
              <div
                ref="timelineRef"
                class="seg-scale relative mx-8px mb-8px select-none"
              >
                <div
                  class="seg-ruler relative h-28px border-b border-white/15"
                  @pointerdown="seekFromTimeline"
                >
                  <span
                    v-for="tick in ticks"
                    :key="tick.left"
                    class="absolute bottom-1px text-10px color-white/45 font-mono -translate-x-1/2 pointer-events-none"
                    :style="{ left: `${tick.left}%` }"
                    >{{ tick.label }}</span
                  >
                </div>
                <div
                  class="seg-track relative h-64px bg-[#2a2a2a] overflow-hidden"
                  @pointerdown="seekFromTimeline"
                >
                  <div class="absolute inset-0">
                    <div
                      v-for="(piece, index) in segments"
                      :key="piece.id"
                      class="seg-piece absolute top-0 bottom-0 pointer-events-none"
                      :style="pieceFrame(piece)"
                    >
                      <button
                        type="button"
                        class="seg-clip"
                        :class="{
                          'is-drop': piece.mode === 'drop',
                          'is-position': piece.mode === 'position',
                          'is-on': isSelected(piece.id),
                        }"
                        @pointerdown.stop
                        @click.stop="onClipClick(piece.id, $event)"
                      >
                        <span class="text-11px font-600 truncate max-w-full"
                          >{{ index + 1 }} · {{ modeName(piece.mode) }}</span
                        >
                        <span
                          class="text-10px opacity-75 font-mono truncate max-w-full"
                          >{{ formatClock(piece.end - piece.start) }}</span
                        >
                      </button>
                      <i
                        v-if="index < segments.length - 1"
                        class="seg-handle"
                        title="拖动改变分段长度"
                        @pointerdown="onBoundaryDown(index, $event)"
                      ></i>
                    </div>
                  </div>
                </div>
                <div
                  class="seg-playhead"
                  :style="{ left: playheadLeft }"
                  title="拖动移动时间"
                  @pointerdown.stop="seekFromTimeline"
                >
                  <span class="seg-playhead-head"></span>
                  <span class="seg-playhead-line"></span>
                </div>
              </div>
            </div>
          </template>
        </template>
      </section>
    </div>
  </div>
</template>

<style scoped>
.seg-dock .play-btn,
.seg-dock .seg-tool,
.seg-dock .seg-clip {
  border-radius: 0;
}
.seg-tool {
  height: 24px;
  padding: 0 8px;
  border: 1px solid rgba(255, 255, 255, 0.16);
  border-radius: 0;
  background: transparent;
  color: #fff;
  font-size: 12px;
  cursor: pointer;
}
.seg-tool:disabled {
  opacity: 0.35;
  cursor: not-allowed;
}
.seg-tool:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.08);
}
.seg-tool.is-on {
  background: #fff;
  color: #1b1b1b;
  border-color: #fff;
}
.seg-tool.is-on:hover:not(:disabled) {
  background: #fff;
}
.play-btn {
  width: 28px;
  height: 28px;
  display: grid;
  place-items: center;
  flex-shrink: 0;
  border: none;
  border-radius: 50%;
  background: #0f6cbd;
  color: #fff;
  cursor: pointer;
}
.play-btn-dark {
  border-radius: 0;
  background: #fff;
  color: #1b1b1b;
}
.seg-clip {
  position: relative;
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  justify-content: center;
  gap: 2px;
  width: 100%;
  min-width: 0;
  height: 100%;
  margin: 0;
  padding: 0 8px;
  border: none;
  overflow: hidden;
  appearance: none;
  background: #0f6cbd;
  color: #fff;
  cursor: pointer;
  text-align: left;
  pointer-events: auto;
}
.seg-clip::after {
  content: "";
  position: absolute;
  top: 0;
  right: 0;
  bottom: 0;
  width: 1px;
  pointer-events: none;
  background: repeating-linear-gradient(
    to bottom,
    #fff 0 1px,
    transparent 1px 3px
  );
}
.seg-piece:last-child > .seg-clip::after {
  content: none;
}
.seg-clip.is-position {
  background: #5c2e91;
}
.seg-clip.is-drop {
  background: repeating-linear-gradient(
    -45deg,
    #3a3a3a,
    #3a3a3a 6px,
    #4a4a4a 6px,
    #4a4a4a 12px
  );
  color: rgba(255, 255, 255, 0.7);
}
.seg-clip.is-on {
  outline: 1px solid #fff;
  outline-offset: -1px;
  z-index: 1;
}
.seg-piece:has(+ .seg-piece > .seg-clip.is-on) > .seg-clip::after {
  opacity: 0;
}
.seg-ruler {
  cursor: grab;
  touch-action: none;
}
.seg-track {
  z-index: 1;
}
.seg-handle {
  position: absolute;
  top: 0;
  right: -4px;
  z-index: 2;
  width: 8px;
  height: 100%;
  cursor: col-resize;
  background: transparent;
  pointer-events: auto;
  touch-action: none;
}
.seg-handle::after {
  content: "";
  position: absolute;
  top: 0;
  bottom: 0;
  left: 50%;
  width: 1px;
  transform: translateX(-50%);
  background: transparent;
}
.seg-handle:hover::after {
  background: #fff;
}
.crop-handle {
  position: absolute;
  width: 8px;
  height: 8px;
  background: #fff;
  border: 1px solid #1b1b1b;
  z-index: 2;
}
.crop-handle.is-nw {
  left: -4px;
  top: -4px;
  cursor: nwse-resize;
}
.crop-handle.is-ne {
  right: -4px;
  top: -4px;
  cursor: nesw-resize;
}
.crop-handle.is-sw {
  left: -4px;
  bottom: -4px;
  cursor: nesw-resize;
}
.crop-handle.is-se {
  right: -4px;
  bottom: -4px;
  cursor: nwse-resize;
}
.crop-handle.is-n {
  left: 50%;
  top: -4px;
  margin-left: -4px;
  cursor: ns-resize;
}
.crop-handle.is-s {
  left: 50%;
  bottom: -4px;
  margin-left: -4px;
  cursor: ns-resize;
}
.crop-handle.is-e {
  right: -4px;
  top: 50%;
  margin-top: -4px;
  cursor: ew-resize;
}
.crop-handle.is-w {
  left: -4px;
  top: 50%;
  margin-top: -4px;
  cursor: ew-resize;
}
.seg-playhead {
  position: absolute;
  top: 0;
  bottom: 0;
  z-index: 6;
  width: 16px;
  transform: translateX(-50%);
  cursor: grab;
  touch-action: none;
}
.seg-playhead-line {
  position: absolute;
  top: 0;
  bottom: 0;
  left: 50%;
  width: 1px;
  transform: translateX(-50%);
  background: #c50f1f;
  pointer-events: none;
}
.seg-playhead-head {
  position: absolute;
  top: 0;
  left: 50%;
  transform: translateX(-50%);
  border-left: 7px solid transparent;
  border-right: 7px solid transparent;
  border-top: 9px solid #c50f1f;
  pointer-events: none;
}
</style>

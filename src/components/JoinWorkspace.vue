<script setup lang="ts">
import {
  computed,
  nextTick,
  onBeforeUnmount,
  onMounted,
  reactive,
  ref,
  watch,
} from "vue";
import { invoke } from "@tauri-apps/api/core";
import type {
  FileInfo,
  JoinFit,
  JoinItem,
  JoinOptions,
  NamingOptions,
} from "../types";
import { formatFileSizeMb } from "../types";
import { useFileThumbs } from "../composables/useFileThumbs";
import MediaPreviewModal from "./MediaPreviewModal.vue";

const props = defineProps<{
  outputDir: string;
  inputFiles: FileInfo[];
  naming: NamingOptions;
}>();

const emit = defineEmits<{
  started: [];
  completed: [message: string];
  finished: [];
}>();

type CanvasPreset = {
  id: string;
  label: string;
  w: number;
  h: number;
};

const CANVAS_PRESETS: CanvasPreset[] = [
  { id: "9:16", label: "9:16 · 1080×1920", w: 1080, h: 1920 },
  { id: "16:9", label: "16:9 · 1920×1080", w: 1920, h: 1080 },
  { id: "1:1", label: "1:1 · 1080×1080", w: 1080, h: 1080 },
  { id: "4:5", label: "4:5 · 1080×1350", w: 1080, h: 1350 },
  { id: "custom", label: "自定义", w: 0, h: 0 },
];

const FIT_OPTIONS: { id: JoinFit; label: string }[] = [
  { id: "cover", label: "覆盖" },
  { id: "contain", label: "包含" },
  { id: "fill", label: "拉伸" },
];

const listFilter = ref<"all" | "image" | "video">("all");
const canvasPresetId = ref("9:16");
const canvasWidth = ref(1080);
const canvasHeight = ref(1920);
const background = ref<"#000000" | "#ffffff" | "transparent">("#000000");
/** 拖到画布边缘时吸附 */
const snapEnabled = ref(true);
const SNAP_THRESHOLD = 12;
/** 视频输出帧率 / Level */
const outputFps = ref<30 | 60>(30);
const setLevel = ref(true);
const videoLevel = ref("4.0");
const videoProfile = ref("high");
const items = ref<JoinItem[]>([]);
const selectedId = ref<string | null>(null);
const statusMessage = ref("");
const isExporting = ref(false);
const itemPreviews = reactive<Record<string, string>>({});
const itemNatives = reactive<Record<string, { w: number; h: number }>>({});
/** 当前吸附到的边，用于画辅助线 */
const snapGuides = ref<{ v: number | null; h: number | null }>({
  v: null,
  h: null,
});

const canvasHostRef = ref<HTMLElement | null>(null);
const viewScale = ref(0.25);

const pickerFiles = computed(() => {
  if (listFilter.value === "all") return props.inputFiles;
  return props.inputFiles.filter((f) => f.file_type === listFilter.value);
});
const pickerFilesRef = computed(() => pickerFiles.value);
const { thumbs } = useFileThumbs(pickerFilesRef);

const selectedItem = computed(
  () => items.value.find((i) => i.id === selectedId.value) || null,
);

const sortedItems = computed(() => [...items.value].sort((a, b) => a.z - b.z));

/** 各素材路径在画布上的使用次数（同一素材可多次添加） */
const pathUseCount = computed(() => {
  const m = new Map<string, number>();
  for (const i of items.value) {
    m.set(i.path, (m.get(i.path) || 0) + 1);
  }
  return m;
});

/** 含任意视频 → 输出视频；全图 → 图片 */
const outputKind = computed<"image" | "video">(() =>
  items.value.some((i) => i.media_kind === "video") ? "video" : "image",
);

const outputKindLabel = computed(() =>
  items.value.length === 0
    ? "待添加"
    : outputKind.value === "video"
      ? "视频 (mp4)"
      : "图片 (png)",
);

const canUseTransparent = computed(() => outputKind.value === "image");

const canExport = computed(
  () =>
    items.value.length > 0 &&
    props.outputDir &&
    canvasWidth.value >= 2 &&
    canvasHeight.value >= 2 &&
    !isExporting.value,
);

const mediaPreviewVisible = ref(false);
const mediaPreviewFile = ref<FileInfo | null>(null);

function openMediaPreview(file: FileInfo, e: Event) {
  e.stopPropagation();
  mediaPreviewFile.value = file;
  mediaPreviewVisible.value = true;
}

function closeMediaPreview() {
  mediaPreviewVisible.value = false;
  mediaPreviewFile.value = null;
}

function evenDim(v: number) {
  const n = Math.max(2, Math.floor(v));
  return n % 2 === 0 ? n : n - 1 || 2;
}

function uid() {
  return `j_${Date.now().toString(36)}_${Math.random().toString(36).slice(2, 7)}`;
}

function fitCanvasView() {
  const el = canvasHostRef.value;
  if (!el) return;
  const pad = 32;
  const aw = Math.max(80, el.clientWidth - pad);
  const ah = Math.max(80, el.clientHeight - pad);
  const sx = aw / canvasWidth.value;
  const sy = ah / canvasHeight.value;
  viewScale.value = Math.max(0.05, Math.min(sx, sy, 1));
}

watch([canvasWidth, canvasHeight], () => {
  nextTick(fitCanvasView);
});

watch(outputKind, (kind) => {
  if (kind === "video" && background.value === "transparent") {
    background.value = "#000000";
  }
});

function applyPreset(id: string) {
  canvasPresetId.value = id;
  const p = CANVAS_PRESETS.find((x) => x.id === id);
  if (!p || p.w <= 0) return;
  canvasWidth.value = p.w;
  canvasHeight.value = p.h;
}

function onCanvasSizeInput() {
  canvasPresetId.value = "custom";
  canvasWidth.value = evenDim(canvasWidth.value || 2);
  canvasHeight.value = evenDim(canvasHeight.value || 2);
}

async function loadPreview(path: string, kind: "image" | "video") {
  if (kind === "video") {
    return invoke<string>("extract_video_frame", { videoPath: path });
  }
  return invoke<string>("load_image_preview", { imagePath: path });
}

async function loadNativeSize(path: string, kind: "image" | "video") {
  if (kind === "video") {
    return invoke<[number, number]>("get_video_dimensions", {
      videoPath: path,
    });
  }
  return invoke<[number, number]>("get_image_dimensions", { imagePath: path });
}

async function addFile(file: FileInfo) {
  if (isExporting.value) return;
  const kind = file.file_type === "video" ? "video" : "image";
  if (file.file_type !== "image" && file.file_type !== "video") {
    statusMessage.value = "仅支持图片或视频";
    return;
  }
  if (items.value.length >= 6) {
    statusMessage.value = "最多添加 6 个素材";
    return;
  }

  statusMessage.value = "";
  try {
    const [nw, nh] = await loadNativeSize(file.path, kind);
    const nativeW = Math.max(1, nw);
    const nativeH = Math.max(1, nh);
    const maxW = Math.floor(canvasWidth.value * 0.45);
    const scale = Math.min(1, maxW / nativeW);
    let w = evenDim(nativeW * scale);
    let h = evenDim(nativeH * scale);
    w = Math.max(2, Math.min(w, canvasWidth.value));
    h = Math.max(2, Math.min(h, canvasHeight.value));
    const x = evenDim(
      Math.max(0, (canvasWidth.value - w) / 2 + items.value.length * 24),
    );
    const y = evenDim(
      Math.max(0, (canvasHeight.value - h) / 2 + items.value.length * 24),
    );
    const maxZ = items.value.reduce((m, i) => Math.max(m, i.z), 0);
    const item: JoinItem = {
      id: uid(),
      path: file.path,
      name: file.name,
      media_kind: kind,
      x,
      y,
      width: w,
      height: h,
      z: maxZ + 1,
      fit: "cover",
      blur: false,
      blur_sigma: 20,
    };
    items.value.push(item);
    selectedId.value = item.id;
    itemNatives[item.id] = { w: nativeW, h: nativeH };
    itemPreviews[item.id] = await loadPreview(file.path, kind);
  } catch (e) {
    statusMessage.value = `添加失败: ${e}`;
  }
}

type SnapCandidate = { pos: number; guide: number };

function bestSnap(
  value: number,
  candidates: SnapCandidate[],
  threshold: number,
): { value: number; guide: number | null } {
  let best = value;
  let guide: number | null = null;
  let bestDist = threshold + 1;
  for (const c of candidates) {
    const d = Math.abs(value - c.pos);
    if (d <= threshold && d < bestDist) {
      bestDist = d;
      best = c.pos;
      guide = c.guide;
    }
  }
  return { value: best, guide };
}

/** 其它图层的垂直参考线（x） */
function otherVerticalEdges(excludeId: string): number[] {
  const edges: number[] = [];
  for (const o of items.value) {
    if (o.id === excludeId) continue;
    edges.push(o.x, o.x + o.width / 2, o.x + o.width);
  }
  return edges;
}

/** 其它图层的水平参考线（y） */
function otherHorizontalEdges(excludeId: string): number[] {
  const edges: number[] = [];
  for (const o of items.value) {
    if (o.id === excludeId) continue;
    edges.push(o.y, o.y + o.height / 2, o.y + o.height);
  }
  return edges;
}

/**
 * 将「当前边」吸附到目标参考线：候选为 (应落到的坐标 pos, 辅助线位置 guide)
 * edge 为 left/right/centerX 或 top/bottom/centerY
 */
function snapEdgeToRefs(
  edgeValue: number,
  edgeKind: "left" | "right" | "cx",
  itemSize: number,
  canvasSize: number,
  otherEdges: number[],
  thr: number,
): { delta: number; guide: number | null } {
  const candidates: SnapCandidate[] = [];
  // 画布
  if (edgeKind === "left") {
    candidates.push({ pos: 0, guide: 0 });
    candidates.push({
      pos: (canvasSize - itemSize) / 2,
      guide: canvasSize / 2,
    });
  } else if (edgeKind === "right") {
    candidates.push({ pos: canvasSize, guide: canvasSize });
    candidates.push({
      pos: (canvasSize + itemSize) / 2,
      guide: canvasSize / 2,
    });
  } else {
    candidates.push({ pos: canvasSize / 2, guide: canvasSize / 2 });
    candidates.push({ pos: itemSize / 2, guide: 0 });
    candidates.push({ pos: canvasSize - itemSize / 2, guide: canvasSize });
  }
  // 其它素材边
  for (const e of otherEdges) {
    if (edgeKind === "left") {
      candidates.push({ pos: e, guide: e });
      candidates.push({ pos: e - itemSize, guide: e }); // 右对齐到 e → left = e - w
    } else if (edgeKind === "right") {
      candidates.push({ pos: e, guide: e });
      candidates.push({ pos: e + itemSize, guide: e }); // 左对齐到 e → right = e + w
    } else {
      candidates.push({ pos: e, guide: e });
    }
  }
  // 对 left：edgeValue 是 left，pos 是目标 left
  // 对 right：edgeValue 是 right，pos 是目标 right
  // 对 cx：edgeValue 是 center，pos 是目标 center
  const hit = bestSnap(edgeValue, candidates, thr);
  if (hit.guide == null) return { delta: 0, guide: null };
  return { delta: hit.value - edgeValue, guide: hit.guide };
}

/** 移动：画布边 + 素材边/中线 吸附 */
function applyMoveSnap(item: JoinItem, x: number, y: number) {
  if (!snapEnabled.value) {
    snapGuides.value = { v: null, h: null };
    item.x = Math.floor(x);
    item.y = Math.floor(y);
    return;
  }
  const thr = SNAP_THRESHOLD;
  const cw = canvasWidth.value;
  const ch = canvasHeight.value;
  const w = item.width;
  const h = item.height;
  const vEdges = otherVerticalEdges(item.id);
  const hEdges = otherHorizontalEdges(item.id);

  // X：左 / 右 / 中 三者取最近
  type AxisHit = { delta: number; guide: number | null; dist: number };
  const xHits: AxisHit[] = [];
  for (const kind of ["left", "right", "cx"] as const) {
    const edge = kind === "left" ? x : kind === "right" ? x + w : x + w / 2;
    const r = snapEdgeToRefs(edge, kind, w, cw, vEdges, thr);
    if (r.guide != null) {
      xHits.push({ delta: r.delta, guide: r.guide, dist: Math.abs(r.delta) });
    }
  }
  xHits.sort((a, b) => a.dist - b.dist);
  const xBest = xHits[0];
  const nx = xBest ? x + xBest.delta : x;
  const gv = xBest?.guide ?? null;

  const yHits: AxisHit[] = [];
  for (const kind of ["left", "right", "cx"] as const) {
    // 复用逻辑：left→top, right→bottom, cx→cy
    const edgeKind =
      kind === "left" ? "left" : kind === "right" ? "right" : "cx";
    const edge = kind === "left" ? y : kind === "right" ? y + h : y + h / 2;
    const r = snapEdgeToRefs(edge, edgeKind, h, ch, hEdges, thr);
    if (r.guide != null) {
      yHits.push({ delta: r.delta, guide: r.guide, dist: Math.abs(r.delta) });
    }
  }
  yHits.sort((a, b) => a.dist - b.dist);
  const yBest = yHits[0];
  const ny = yBest ? y + yBest.delta : y;
  const gh = yBest?.guide ?? null;

  item.x = Math.floor(nx);
  item.y = Math.floor(ny);
  snapGuides.value = { v: gv, h: gh };
}

/** 缩放：画布边 + 素材边 吸附 */
function applyResizeSnap(
  item: JoinItem,
  x: number,
  y: number,
  w: number,
  h: number,
  handle: string,
) {
  if (!snapEnabled.value) {
    snapGuides.value = { v: null, h: null };
    item.x = Math.floor(x);
    item.y = Math.floor(y);
    item.width = evenDim(w);
    item.height = evenDim(h);
    return;
  }
  const thr = SNAP_THRESHOLD;
  const cw = canvasWidth.value;
  const ch = canvasHeight.value;
  const vEdges = otherVerticalEdges(item.id);
  const hEdges = otherHorizontalEdges(item.id);
  let nx = x;
  let ny = y;
  let nw = w;
  let nh = h;
  let gv: number | null = null;
  let gh: number | null = null;

  if (handle.includes("e")) {
    const right = x + w;
    const targets: SnapCandidate[] = [
      { pos: cw, guide: cw },
      ...vEdges.map((e) => ({ pos: e, guide: e })),
    ];
    const s = bestSnap(right, targets, thr);
    if (s.guide != null) {
      nw = Math.max(2, s.value - x);
      gv = s.guide;
    }
  }
  if (handle.includes("w")) {
    const targets: SnapCandidate[] = [
      { pos: 0, guide: 0 },
      ...vEdges.map((e) => ({ pos: e, guide: e })),
    ];
    const s = bestSnap(x, targets, thr);
    if (s.guide != null) {
      const right = x + w;
      nx = s.value;
      nw = Math.max(2, right - nx);
      gv = s.guide;
    }
  }
  if (handle.includes("s")) {
    const bottom = y + h;
    const targets: SnapCandidate[] = [
      { pos: ch, guide: ch },
      ...hEdges.map((e) => ({ pos: e, guide: e })),
    ];
    const s = bestSnap(bottom, targets, thr);
    if (s.guide != null) {
      nh = Math.max(2, s.value - y);
      gh = s.guide;
    }
  }
  if (handle.includes("n")) {
    const targets: SnapCandidate[] = [
      { pos: 0, guide: 0 },
      ...hEdges.map((e) => ({ pos: e, guide: e })),
    ];
    const s = bestSnap(y, targets, thr);
    if (s.guide != null) {
      const bottom = y + h;
      ny = s.value;
      nh = Math.max(2, bottom - ny);
      gh = s.guide;
    }
  }

  item.x = Math.floor(nx);
  item.y = Math.floor(ny);
  item.width = evenDim(nw);
  item.height = evenDim(nh);
  snapGuides.value = { v: gv, h: gh };
}

function removeSelected() {
  if (!selectedId.value) return;
  const id = selectedId.value;
  items.value = items.value.filter((i) => i.id !== id);
  delete itemPreviews[id];
  delete itemNatives[id];
  selectedId.value = items.value[items.value.length - 1]?.id ?? null;
}

function bringForward() {
  const item = selectedItem.value;
  if (!item) return;
  const higher = items.value
    .filter((i) => i.z > item.z)
    .sort((a, b) => a.z - b.z)[0];
  if (!higher) return;
  const tz = item.z;
  item.z = higher.z;
  higher.z = tz;
}

function sendBackward() {
  const item = selectedItem.value;
  if (!item) return;
  const lower = items.value
    .filter((i) => i.z < item.z)
    .sort((a, b) => b.z - a.z)[0];
  if (!lower) return;
  const tz = item.z;
  item.z = lower.z;
  lower.z = tz;
}

function updateSelectedNumber(
  key: "x" | "y" | "width" | "height",
  raw: string,
) {
  const item = selectedItem.value;
  if (!item) return;
  const n = Number(raw);
  if (!Number.isFinite(n)) return;
  if (key === "width" || key === "height") {
    item[key] = evenDim(Math.max(2, n));
  } else {
    item[key] = Math.floor(n);
  }
}

function setFit(fit: JoinFit) {
  const item = selectedItem.value;
  if (item) item.fit = fit;
}

function setBlur(enabled: boolean) {
  const item = selectedItem.value;
  if (!item) return;
  item.blur = enabled;
  if (item.blur_sigma == null || item.blur_sigma <= 0) {
    item.blur_sigma = 20;
  }
}

function setBlurSigma(raw: string) {
  const item = selectedItem.value;
  if (!item) return;
  const n = Number(raw);
  if (!Number.isFinite(n)) return;
  item.blur_sigma = Math.max(1, Math.min(50, Math.round(n)));
  if (item.blur_sigma > 0) item.blur = true;
}

/** 画布预览用 CSS blur，与导出 sigma 大致对应 */
function previewBlurPx(sigma: number | undefined) {
  const s = sigma != null && Number.isFinite(sigma) ? sigma : 20;
  return Math.max(0, Math.min(40, s * 0.55));
}

// --- drag / resize ---
type DragMode =
  | {
      kind: "move";
      id: string;
      startX: number;
      startY: number;
      origX: number;
      origY: number;
    }
  | {
      kind: "resize";
      id: string;
      handle: string;
      startX: number;
      startY: number;
      orig: { x: number; y: number; w: number; h: number };
    };

const drag = ref<DragMode | null>(null);

function onItemPointerDown(e: PointerEvent, id: string) {
  if (isExporting.value) return;
  e.preventDefault();
  e.stopPropagation();
  selectedId.value = id;
  const item = items.value.find((i) => i.id === id);
  if (!item) return;
  drag.value = {
    kind: "move",
    id,
    startX: e.clientX,
    startY: e.clientY,
    origX: item.x,
    origY: item.y,
  };
  (e.target as HTMLElement).setPointerCapture?.(e.pointerId);
}

function onResizePointerDown(e: PointerEvent, id: string, handle: string) {
  if (isExporting.value) return;
  e.preventDefault();
  e.stopPropagation();
  selectedId.value = id;
  const item = items.value.find((i) => i.id === id);
  if (!item) return;
  drag.value = {
    kind: "resize",
    id,
    handle,
    startX: e.clientX,
    startY: e.clientY,
    orig: { x: item.x, y: item.y, w: item.width, h: item.height },
  };
  (e.target as HTMLElement).setPointerCapture?.(e.pointerId);
}

function onPointerMove(e: PointerEvent) {
  const d = drag.value;
  if (!d) return;
  const item = items.value.find((i) => i.id === d.id);
  if (!item) return;
  const dx = (e.clientX - d.startX) / viewScale.value;
  const dy = (e.clientY - d.startY) / viewScale.value;

  if (d.kind === "move") {
    applyMoveSnap(item, d.origX + dx, d.origY + dy);
    return;
  }

  const o = d.orig;
  let x = o.x;
  let y = o.y;
  let w = o.w;
  let h = o.h;
  const hdl = d.handle;

  if (hdl.includes("e")) w = Math.max(2, o.w + dx);
  if (hdl.includes("s")) h = Math.max(2, o.h + dy);
  if (hdl.includes("w")) {
    w = Math.max(2, o.w - dx);
    x = o.x + (o.w - w);
  }
  if (hdl.includes("n")) {
    h = Math.max(2, o.h - dy);
    y = o.y + (o.h - h);
  }

  applyResizeSnap(item, x, y, w, h, hdl);
}

function onPointerUp() {
  drag.value = null;
  snapGuides.value = { v: null, h: null };
}

function onCanvasBgClick() {
  selectedId.value = null;
}

function buildOutputName() {
  const parts: string[] = [];
  if (props.naming.use_original_name) parts.push("join");
  if (props.naming.use_timestamp) parts.push(String(Date.now()));
  if (props.naming.use_datetime) {
    const d = new Date();
    const pad = (n: number) => n.toString().padStart(2, "0");
    parts.push(
      `${d.getFullYear()}${pad(d.getMonth() + 1)}${pad(d.getDate())}_${pad(d.getHours())}${pad(d.getMinutes())}${pad(d.getSeconds())}`,
    );
  }
  if (props.naming.custom_text.trim())
    parts.push(props.naming.custom_text.trim());
  if (parts.length === 0) parts.push(`join_${Date.now()}`);
  const ext = outputKind.value === "image" ? "png" : "mp4";
  return `${parts.join("-")}.${ext}`;
}

async function startExport() {
  if (!canExport.value) return;
  isExporting.value = true;
  statusMessage.value = "正在导出…";

  const bg =
    !canUseTransparent.value && background.value === "transparent"
      ? "#000000"
      : background.value;

  const outputFileName = buildOutputName();
  const sep = props.outputDir.includes("\\") ? "\\" : "/";
  const outputPath = `${props.outputDir.replace(/[\\/]$/, "")}${sep}${outputFileName}`;

  const options: JoinOptions = {
    media_kind: outputKind.value,
    canvas_width: evenDim(canvasWidth.value),
    canvas_height: evenDim(canvasHeight.value),
    background: bg,
    items: items.value.map((i) => ({
      ...i,
      x: Math.floor(i.x),
      y: Math.floor(i.y),
      width: evenDim(i.width),
      height: evenDim(i.height),
      blur: !!i.blur,
      blur_sigma:
        i.blur_sigma != null && Number.isFinite(i.blur_sigma)
          ? Math.max(1, Math.min(50, i.blur_sigma))
          : 20,
    })),
    output_path: outputPath,
  };

  if (outputKind.value === "video") {
    options.output_fps = outputFps.value;
    options.set_level = setLevel.value;
    if (setLevel.value) {
      options.video_level = videoLevel.value;
      options.video_profile = videoProfile.value;
    }
  }

  emit("started");
  try {
    const result = await invoke<string>("join_media", { options });
    statusMessage.value = result;
    emit("completed", result);
  } catch (e) {
    statusMessage.value = `导出失败: ${e}`;
  } finally {
    isExporting.value = false;
    emit("finished");
  }
}

function onKeyDown(e: KeyboardEvent) {
  if (!selectedId.value || isExporting.value) return;
  const tag = (e.target as HTMLElement)?.tagName;
  if (tag === "INPUT" || tag === "SELECT" || tag === "TEXTAREA") return;
  if (e.key === "Delete" || e.key === "Backspace") {
    e.preventDefault();
    removeSelected();
  }
}

function onWindowResize() {
  fitCanvasView();
}

onMounted(() => {
  nextTick(fitCanvasView);
  window.addEventListener("resize", onWindowResize);
  window.addEventListener("keydown", onKeyDown);
});

onBeforeUnmount(() => {
  window.removeEventListener("resize", onWindowResize);
  window.removeEventListener("keydown", onKeyDown);
});
</script>

<template>
  <div class="flex-1 min-h-0 flex flex-col bg-bg1 overflow-hidden">
    <!-- 顶栏 -->
    <div
      class="shrink-0 px-14px py-10px flex items-center justify-between gap-12px border-b border-border flex-wrap"
    >
      <div class="flex items-center gap-16px flex-wrap min-w-0">
        <div class="min-w-140px">
          <div class="text-13px font-600">自定义拼接</div>
          <div class="text-11px color-t3 mt-2px">图/视频可混排 · 最多 6 层</div>
        </div>

        <div
          class="flex h-36px flex-wrap items-center gap-12px px-12px bg-bg0 rounded-8px border border-border"
        >
          <span class="text-12px color-t3">画布</span>
          <select
            class="field w-auto! min-w-140px h-28px! px-8px! text-12px!"
            :value="canvasPresetId"
            :disabled="isExporting"
            @change="applyPreset(($event.target as HTMLSelectElement).value)"
          >
            <option v-for="p in CANVAS_PRESETS" :key="p.id" :value="p.id">
              {{ p.label }}
            </option>
          </select>
          <label class="text-12px color-t3">宽</label>
          <input
            class="field w-80px! h-26px"
            type="number"
            min="2"
            v-model.number="canvasWidth"
            :disabled="isExporting"
            @change="onCanvasSizeInput"
          />
          <label class="text-12px color-t3">高</label>
          <input
            class="field w-80px! h-26px"
            type="number"
            min="2"
            v-model.number="canvasHeight"
            :disabled="isExporting"
            @change="onCanvasSizeInput"
          />
          <span class="text-12px color-t3">背景</span>
          <select
            class="field w-auto! min-w-88px h-28px! px-8px! text-12px!"
            v-model="background"
            :disabled="isExporting"
          >
            <option value="#000000">黑色</option>
            <option value="#ffffff">白色</option>
            <option value="transparent" :disabled="!canUseTransparent">
              透明{{ !canUseTransparent ? "（仅纯图）" : "" }}
            </option>
          </select>
          <div class="w-1px h-16px bg-border"></div>
          <label
            class="flex h-28px items-center gap-6px cursor-pointer text-12px select-none"
            title="拖到画布边缘时自动吸附"
          >
            <input
              type="checkbox"
              v-model="snapEnabled"
              :disabled="isExporting"
              class="accent-secondary"
            />
            边缘吸附
            <span class="text-10px color-t3">（画布+素材）</span>
          </label>
        </div>

        <div
          class="bg-bg0 px-10px h-36px flex items-center rounded-6px text-12px color-t2 gap-8px"
        >
          <span>图层 {{ items.length }}/6</span>
          <span class="color-t3">·</span>
          <span>{{ canvasWidth }}×{{ canvasHeight }}</span>
          <span class="color-t3">·</span>
          <span class="color-secondary">输出 {{ outputKindLabel }}</span>
        </div>
      </div>

      <button
        class="tb-btn tb-btn-success"
        type="button"
        :disabled="!canExport"
        @click="startExport"
      >
        {{ isExporting ? "导出中…" : "导出拼接" }}
      </button>
    </div>

    <div class="flex-1 min-h-0 flex overflow-hidden">
      <!-- 左侧素材 -->
      <div
        class="w-260px shrink-0 min-h-0 border-r border-border flex flex-col bg-bg0 overflow-hidden"
      >
        <div
          class="shrink-0 px-10px py-6px flex flex-col gap-6px text-11px font-500 color-t3 border-b border-border bg-bg2"
        >
          <div class="flex items-center justify-between">
            <span>素材 ({{ pickerFiles.length }})</span>
          </div>
          <div class="flex gap-4px">
            <button
              v-for="f in [
                { id: 'all', label: '全部' },
                { id: 'image', label: '图' },
                { id: 'video', label: '视频' },
              ]"
              :key="f.id"
              type="button"
              class="h-22px px-8px rounded-4px text-10px border-none cursor-pointer"
              :class="
                listFilter === f.id
                  ? 'bg-secondary color-white'
                  : 'bg-bg0 color-t3 hover:color-t1'
              "
              :disabled="isExporting"
              @click="listFilter = f.id as 'all' | 'image' | 'video'"
            >
              {{ f.label }}
            </button>
          </div>
        </div>
        <div class="flex-1 min-h-0 overflow-y-auto">
          <button
            v-for="f in pickerFiles"
            :key="f.path"
            type="button"
            class="w-full text-left px-10px py-8px border-none border-b border-border/60 cursor-pointer text-12px flex gap-10px items-center bg-transparent color-t2 hover:bg-bg2"
            :disabled="isExporting"
            @click="addFile(f)"
          >
            <div
              class="w-44px h-44px rounded-4px overflow-hidden bg-bg1 border border-border shrink-0 cursor-zoom-in hover:border-secondary"
              title="预览"
              @click="openMediaPreview(f, $event)"
            >
              <img
                v-if="thumbs[f.path]"
                :src="thumbs[f.path]"
                class="w-full h-full object-cover pointer-events-none"
                draggable="false"
              />
              <div
                v-else
                class="w-full h-full grid place-items-center text-10px color-t3 pointer-events-none"
              >
                …
              </div>
            </div>
            <div class="min-w-0 flex-1">
              <div class="truncate font-500" :title="f.name">{{ f.name }}</div>
              <div class="text-10px color-t3 mt-2px">
                {{ f.file_type === "video" ? "视频" : "图片" }} ·
                {{ formatFileSizeMb(f.size_bytes) }}
                <span
                  v-if="(pathUseCount.get(f.path) || 0) > 0"
                  class="ml-4px color-secondary"
                >
                  · 已用 {{ pathUseCount.get(f.path) }}
                </span>
              </div>
            </div>
          </button>
          <div
            v-if="pickerFiles.length === 0"
            class="p-16px text-11px color-t3 text-center"
          >
            输入目录无{{
              listFilter === "video"
                ? "视频"
                : listFilter === "image"
                  ? "图片"
                  : "素材"
            }}
          </div>
        </div>
      </div>

      <!-- 画布 -->
      <div
        ref="canvasHostRef"
        class="flex-1 min-w-0 min-h-0 overflow-hidden flex items-center justify-center bg-[linear-gradient(45deg,#2a2a2e_25%,transparent_25%),linear-gradient(-45deg,#2a2a2e_25%,transparent_25%),linear-gradient(45deg,transparent_75%,#2a2a2e_75%),linear-gradient(-45deg,transparent_75%,#2a2a2e_75%)] bg-[length:16px_16px] bg-[position:0_0,0_8px,8px_-8px,-8px_0] bg-bg0"
        @pointermove="onPointerMove"
        @pointerup="onPointerUp"
        @pointercancel="onPointerUp"
        @click="onCanvasBgClick"
      >
        <div
          class="relative shadow-lg shrink-0 box-border"
          :style="{
            width: canvasWidth * viewScale + 'px',
            height: canvasHeight * viewScale + 'px',
            background:
              canUseTransparent && background === 'transparent'
                ? 'transparent'
                : background === 'transparent'
                  ? '#000000'
                  : background,
            outline: '1px dashed red',
            outlineOffset: '0px',
            boxShadow: '0 0 0 1px rgba(0,0,0,0.5), 0 8px 24px rgba(0,0,0,0.35)',
          }"
          @click.stop
        >
          <!-- 吸附辅助线 -->
          <div
            v-if="snapGuides.v != null"
            class="absolute top-0 bottom-0 w-0 pointer-events-none z-100"
            :style="{
              left: snapGuides.v * viewScale + 'px',
              borderLeft: '1px dashed #5b9cff',
            }"
          ></div>
          <div
            v-if="snapGuides.h != null"
            class="absolute left-0 right-0 h-0 pointer-events-none z-100"
            :style="{
              top: snapGuides.h * viewScale + 'px',
              borderTop: '1px dashed #5b9cff',
            }"
          ></div>
          <div
            v-for="item in sortedItems"
            :key="item.id"
            class="absolute box-border select-none overflow-hidden"
            :class="
              selectedId === item.id
                ? 'outline outline-1 outline-secondary z-50'
                : 'outline outline-1 outline-white/30'
            "
            :style="{
              left: item.x * viewScale + 'px',
              top: item.y * viewScale + 'px',
              width: item.width * viewScale + 'px',
              height: item.height * viewScale + 'px',
              zIndex: item.z,
              cursor:
                drag?.kind === 'move' && drag.id === item.id
                  ? 'grabbing'
                  : 'grab',
            }"
            @pointerdown="onItemPointerDown($event, item.id)"
            @click.stop="selectedId = item.id"
          >
            <img
              v-if="itemPreviews[item.id]"
              :src="itemPreviews[item.id]"
              class="w-full h-full pointer-events-none"
              :class="{
                'object-cover': item.fit === 'cover',
                'object-contain': item.fit === 'contain',
                'object-fill': item.fit === 'fill',
              }"
              :style="
                item.blur
                  ? {
                      filter: `blur(${previewBlurPx(item.blur_sigma)}px)`,
                      transform: 'scale(1.06)',
                    }
                  : undefined
              "
              draggable="false"
            />
            <div
              v-else
              class="w-full h-full grid place-items-center text-10px color-t3 bg-bg2"
            >
              …
            </div>
            <div
              class="absolute left-0 right-0 bottom-0 px-4px py-2px text-10px color-white bg-black/50 truncate pointer-events-none flex items-center gap-4px"
            >
              <span
                class="shrink-0 px-3px rounded-2px text-9px"
                :class="
                  item.media_kind === 'video'
                    ? 'bg-secondary/90'
                    : 'bg-white/25'
                "
              >
                {{ item.media_kind === "video" ? "视频" : "图" }}
              </span>
              <span
                v-if="item.blur"
                class="shrink-0 px-3px rounded-2px text-9px bg-amber-500/90"
              >
                模糊
              </span>
              <span class="truncate">{{ item.name }}</span>
            </div>

            <template v-if="selectedId === item.id">
              <div
                v-for="h in ['nw', 'ne', 'sw', 'se', 'n', 's', 'e', 'w']"
                :key="h"
                class="absolute w-8px h-8px bg-secondary border border-white rounded-1px z-10"
                :class="{
                  'left-0 top-0 -translate-x-1/2 -translate-y-1/2 cursor-nwse-resize':
                    h === 'nw',
                  'right-0 top-0 translate-x-1/2 -translate-y-1/2 cursor-nesw-resize':
                    h === 'ne',
                  'left-0 bottom-0 -translate-x-1/2 translate-y-1/2 cursor-nesw-resize':
                    h === 'sw',
                  'right-0 bottom-0 translate-x-1/2 translate-y-1/2 cursor-nwse-resize':
                    h === 'se',
                  'left-1/2 top-0 -translate-x-1/2 -translate-y-1/2 cursor-ns-resize':
                    h === 'n',
                  'left-1/2 bottom-0 -translate-x-1/2 translate-y-1/2 cursor-ns-resize':
                    h === 's',
                  'right-0 top-1/2 translate-x-1/2 -translate-y-1/2 cursor-ew-resize':
                    h === 'e',
                  'left-0 top-1/2 -translate-x-1/2 -translate-y-1/2 cursor-ew-resize':
                    h === 'w',
                }"
                @pointerdown="onResizePointerDown($event, item.id, h)"
              ></div>
            </template>
          </div>
        </div>
      </div>

      <!-- 右侧属性 -->
      <div
        class="w-240px shrink-0 min-h-0 border-l border-border flex flex-col bg-bg0 overflow-hidden"
      >
        <div
          class="shrink-0 h-32px px-10px flex items-center text-11px font-500 color-t3 border-b border-border bg-bg2"
        >
          输出设置(仅视频有效)
        </div>
        <div class="px-12px py-4px flex flex-col gap-6px">
          <div class="flex h-28px items-center gap-6px">
            <template v-if="items.length === 0 || outputKind === 'video'">
              <span class="text-12px color-t3">帧率</span>
              <label class="flex items-center gap-4px cursor-pointer text-12px">
                <input
                  type="radio"
                  :value="30"
                  v-model.number="outputFps"
                  :disabled="isExporting"
                  class="accent-secondary"
                />
                30
              </label>
              <label
                class="flex items-center gap-4px cursor-pointer text-12px"
                title="源不足 60fps 时自动补帧"
              >
                <input
                  type="radio"
                  :value="60"
                  v-model.number="outputFps"
                  :disabled="isExporting"
                  class="accent-secondary"
                />
                60
              </label>
            </template>
          </div>
          <div class="flex items-center h-28px">
            <label class="flex items-center gap-6px cursor-pointer text-12px">
              Level
            </label>
            <div>
              <select
                class="field w-auto! min-w-72px h-28px! px-6px! text-11px!"
                v-model="videoProfile"
                :disabled="isExporting"
              >
                <option value="baseline">Baseline</option>
                <option value="main">Main</option>
                <option value="high">High</option>
              </select>
              <select
                class="field w-auto! min-w-72px h-28px! px-6px! text-11px!"
                v-model="videoLevel"
                :disabled="isExporting"
              >
                <option value="3.0">3.0</option>
                <option value="3.1">3.1</option>
                <option value="4.0">4.0</option>
                <option value="4.1">4.1</option>
                <option value="4.2">4.2</option>
                <option value="5.0">5.0</option>
                <option value="5.1">5.1</option>
                <option value="5.2">5.2</option>
              </select>
            </div>
          </div>
        </div>
        <div
          class="shrink-0 h-32px px-10px flex items-center text-11px font-500 color-t3 border-b border-border bg-bg2"
        >
          素材属性
        </div>
        <div
          class="flex-1 min-h-0 overflow-y-auto p-12px flex flex-col gap-10px"
        >
          <template v-if="selectedItem">
            <div class="text-12px font-500 truncate" :title="selectedItem.name">
              <span class="color-t3 font-400 mr-4px">
                {{ selectedItem.media_kind === "video" ? "视频" : "图片" }}
              </span>
              {{ selectedItem.name }}
            </div>
            <div class="grid grid-cols-2 gap-8px">
              <label class="text-11px color-t3 flex flex-col gap-4px">
                X
                <input
                  class="field h-28px"
                  type="number"
                  :value="selectedItem.x"
                  :disabled="isExporting"
                  @input="
                    updateSelectedNumber(
                      'x',
                      ($event.target as HTMLInputElement).value,
                    )
                  "
                />
              </label>
              <label class="text-11px color-t3 flex flex-col gap-4px">
                Y
                <input
                  class="field h-28px"
                  type="number"
                  :value="selectedItem.y"
                  :disabled="isExporting"
                  @input="
                    updateSelectedNumber(
                      'y',
                      ($event.target as HTMLInputElement).value,
                    )
                  "
                />
              </label>
              <label class="text-11px color-t3 flex flex-col gap-4px">
                宽
                <input
                  class="field h-28px"
                  type="number"
                  min="2"
                  :value="selectedItem.width"
                  :disabled="isExporting"
                  @input="
                    updateSelectedNumber(
                      'width',
                      ($event.target as HTMLInputElement).value,
                    )
                  "
                />
              </label>
              <label class="text-11px color-t3 flex flex-col gap-4px">
                高
                <input
                  class="field h-28px"
                  type="number"
                  min="2"
                  :value="selectedItem.height"
                  :disabled="isExporting"
                  @input="
                    updateSelectedNumber(
                      'height',
                      ($event.target as HTMLInputElement).value,
                    )
                  "
                />
              </label>
            </div>

            <label class="text-11px color-t3 flex flex-col gap-4px">
              填充
              <select
                class="field h-28px"
                :value="selectedItem.fit"
                :disabled="isExporting"
                @change="
                  setFit(($event.target as HTMLSelectElement).value as JoinFit)
                "
              >
                <option v-for="f in FIT_OPTIONS" :key="f.id" :value="f.id">
                  {{ f.label }}
                </option>
              </select>
            </label>

            <div class="flex flex-col gap-6px">
              <label
                class="text-11px color-t3 flex items-center gap-6px cursor-pointer"
              >
                <input
                  type="checkbox"
                  class="accent-secondary"
                  :checked="!!selectedItem.blur"
                  :disabled="isExporting"
                  @change="
                    setBlur(($event.target as HTMLInputElement).checked)
                  "
                />
                区域模糊
              </label>
              <label
                v-if="selectedItem.blur"
                class="text-11px color-t3 flex flex-col gap-4px"
              >
                模糊强度 (1–50)
                <input
                  class="field h-28px"
                  type="number"
                  min="1"
                  max="50"
                  :value="selectedItem.blur_sigma ?? 20"
                  :disabled="isExporting"
                  @input="
                    setBlurSigma(($event.target as HTMLInputElement).value)
                  "
                />
              </label>
              <p class="text-10px color-t3 leading-relaxed">
                模糊作用于当前图层矩形：拖拽/缩放即可自选位置与大小。上下虚化时：底层铺满并开模糊，顶层清晰居中。
              </p>
            </div>

            <div class="flex gap-6px flex-wrap">
              <button
                class="tb-btn h-28px! px-8px! text-11px!"
                type="button"
                :disabled="isExporting"
                @click="bringForward"
              >
                上移一层
              </button>
              <button
                class="tb-btn h-28px! px-8px! text-11px!"
                type="button"
                :disabled="isExporting"
                @click="sendBackward"
              >
                下移一层
              </button>
              <button
                class="tb-btn h-28px! px-8px! text-11px!"
                type="button"
                :disabled="isExporting"
                @click="removeSelected"
              >
                删除
              </button>
            </div>
            <p class="text-10px color-t3">Delete 删除 · 拖拽移动 · 角点缩放</p>
          </template>
          <div v-else class="text-11px color-t3 leading-relaxed">
            从左侧点击图片/视频加入画布（可混排、可重复）。全图导出
            png，含视频导出 mp4。选中后可拖拽、缩放、开关区域模糊。
          </div>

          <div
            v-if="statusMessage"
            class="mt-auto text-11px color-t2 whitespace-pre-wrap break-all border border-border rounded-6px p-8px bg-bg1"
          >
            {{ statusMessage }}
          </div>
        </div>
      </div>
    </div>

    <MediaPreviewModal
      :visible="mediaPreviewVisible"
      :file="mediaPreviewFile"
      @close="closeMediaPreview"
    />
  </div>
</template>

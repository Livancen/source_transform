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
import type { FileInfo, JoinFit, JoinItem, JoinOptions, NamingOptions } from "../types";
import { formatFileSizeMb } from "../types";
import { useFileThumbs } from "../composables/useFileThumbs";
import MediaPreviewModal from "./MediaPreviewModal.vue";

const props = defineProps<{
  outputDir: string;
  inputFiles: FileInfo[];
  naming: NamingOptions;
}>();

const emit = defineEmits<{
  completed: [message: string];
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

const mediaKind = ref<"video" | "image">("image");
const canvasPresetId = ref("9:16");
const canvasWidth = ref(1080);
const canvasHeight = ref(1920);
const background = ref<"#000000" | "#ffffff" | "transparent">("#000000");
const items = ref<JoinItem[]>([]);
const selectedId = ref<string | null>(null);
const statusMessage = ref("");
const isExporting = ref(false);
const itemPreviews = reactive<Record<string, string>>({});
const itemNatives = reactive<Record<string, { w: number; h: number }>>({});

const canvasHostRef = ref<HTMLElement | null>(null);
const viewScale = ref(0.25);

const pickerFiles = computed(() =>
  props.inputFiles.filter((f) => f.file_type === mediaKind.value),
);
const pickerFilesRef = computed(() => pickerFiles.value);
const { thumbs } = useFileThumbs(pickerFilesRef);

const selectedItem = computed(
  () => items.value.find((i) => i.id === selectedId.value) || null,
);

const sortedItems = computed(() =>
  [...items.value].sort((a, b) => a.z - b.z),
);

const usedPaths = computed(() => new Set(items.value.map((i) => i.path)));

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

watch(mediaKind, () => {
  items.value = [];
  selectedId.value = null;
  statusMessage.value = "";
  if (mediaKind.value === "video" && background.value === "transparent") {
    background.value = "#000000";
  }
  for (const k of Object.keys(itemPreviews)) delete itemPreviews[k];
  for (const k of Object.keys(itemNatives)) delete itemNatives[k];
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
    return invoke<[number, number]>("get_video_dimensions", { videoPath: path });
  }
  return invoke<[number, number]>("get_image_dimensions", { imagePath: path });
}

async function addFile(file: FileInfo) {
  if (isExporting.value) return;
  if (file.file_type !== mediaKind.value) {
    statusMessage.value =
      mediaKind.value === "video" ? "请选择视频文件" : "请选择图片文件";
    return;
  }
  if (items.value.length >= 6) {
    statusMessage.value = "最多添加 6 个素材";
    return;
  }
  if (usedPaths.value.has(file.path)) {
    const existing = items.value.find((i) => i.path === file.path);
    if (existing) selectedId.value = existing.id;
    return;
  }

  statusMessage.value = "";
  try {
    const [nw, nh] = await loadNativeSize(file.path, mediaKind.value);
    const nativeW = Math.max(1, nw);
    const nativeH = Math.max(1, nh);
    const maxW = Math.floor(canvasWidth.value * 0.45);
    const scale = Math.min(1, maxW / nativeW);
    let w = evenDim(nativeW * scale);
    let h = evenDim(nativeH * scale);
    w = Math.max(2, Math.min(w, canvasWidth.value));
    h = Math.max(2, Math.min(h, canvasHeight.value));
    const x = evenDim(Math.max(0, (canvasWidth.value - w) / 2 + items.value.length * 24));
    const y = evenDim(Math.max(0, (canvasHeight.value - h) / 2 + items.value.length * 24));
    const maxZ = items.value.reduce((m, i) => Math.max(m, i.z), 0);
    const item: JoinItem = {
      id: uid(),
      path: file.path,
      name: file.name,
      x,
      y,
      width: w,
      height: h,
      z: maxZ + 1,
      fit: "cover",
    };
    items.value.push(item);
    selectedId.value = item.id;
    itemNatives[item.id] = { w: nativeW, h: nativeH };
    itemPreviews[item.id] = await loadPreview(file.path, mediaKind.value);
  } catch (e) {
    statusMessage.value = `添加失败: ${e}`;
  }
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
  const higher = items.value.filter((i) => i.z > item.z).sort((a, b) => a.z - b.z)[0];
  if (!higher) return;
  const tz = item.z;
  item.z = higher.z;
  higher.z = tz;
}

function sendBackward() {
  const item = selectedItem.value;
  if (!item) return;
  const lower = items.value.filter((i) => i.z < item.z).sort((a, b) => b.z - a.z)[0];
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

// --- drag / resize ---
type DragMode =
  | { kind: "move"; id: string; startX: number; startY: number; origX: number; origY: number }
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
    item.x = Math.floor(d.origX + dx);
    item.y = Math.floor(d.origY + dy);
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

  item.x = Math.floor(x);
  item.y = Math.floor(y);
  item.width = evenDim(w);
  item.height = evenDim(h);
}

function onPointerUp() {
  drag.value = null;
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
  if (props.naming.custom_text.trim()) parts.push(props.naming.custom_text.trim());
  if (parts.length === 0) parts.push(`join_${Date.now()}`);
  const ext = mediaKind.value === "image" ? "png" : "mp4";
  return `${parts.join("-")}.${ext}`;
}

async function startExport() {
  if (!canExport.value) return;
  isExporting.value = true;
  statusMessage.value = "正在导出…";

  const outputFileName = buildOutputName();
  const sep = props.outputDir.includes("\\") ? "\\" : "/";
  const outputPath = `${props.outputDir.replace(/[\\/]$/, "")}${sep}${outputFileName}`;

  const options: JoinOptions = {
    media_kind: mediaKind.value,
    canvas_width: evenDim(canvasWidth.value),
    canvas_height: evenDim(canvasHeight.value),
    background: background.value,
    items: items.value.map((i) => ({
      ...i,
      x: Math.floor(i.x),
      y: Math.floor(i.y),
      width: evenDim(i.width),
      height: evenDim(i.height),
    })),
    output_path: outputPath,
  };

  try {
    const result = await invoke<string>("join_media", { options });
    statusMessage.value = result;
    emit("completed", result);
  } catch (e) {
    statusMessage.value = `导出失败: ${e}`;
  } finally {
    isExporting.value = false;
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
          <div class="text-11px color-t3 mt-2px">画布自由布局 · 最多 6 层</div>
        </div>

        <div
          class="flex flex-wrap items-center gap-12px py-8px px-12px bg-bg0 rounded-8px border border-border"
        >
          <label class="flex items-center gap-6px cursor-pointer text-12px">
            <input
              type="radio"
              value="image"
              v-model="mediaKind"
              :disabled="isExporting"
              class="accent-secondary"
            />
            图片
          </label>
          <label class="flex items-center gap-6px cursor-pointer text-12px">
            <input
              type="radio"
              value="video"
              v-model="mediaKind"
              :disabled="isExporting"
              class="accent-secondary"
            />
            视频
          </label>
          <div class="w-1px h-16px bg-border"></div>
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
            <option value="transparent" :disabled="mediaKind === 'video'">
              透明{{ mediaKind === "video" ? "（仅图）" : "" }}
            </option>
          </select>
        </div>

        <div
          class="bg-bg0 px-10px h-36px flex items-center rounded-6px text-12px color-t2"
        >
          图层 {{ items.length }}/6 · {{ canvasWidth }}×{{ canvasHeight }}
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
          class="shrink-0 h-32px px-10px flex items-center justify-between text-11px font-500 color-t3 border-b border-border bg-bg2"
        >
          <span>素材 ({{ pickerFiles.length }})</span>
          <span class="color-secondary">点击加入</span>
        </div>
        <div class="flex-1 min-h-0 overflow-y-auto">
          <button
            v-for="f in pickerFiles"
            :key="f.path"
            type="button"
            class="w-full text-left px-10px py-8px border-none border-b border-border/60 cursor-pointer text-12px flex gap-10px items-center"
            :class="
              usedPaths.has(f.path)
                ? 'bg-secondary-soft color-secondary'
                : 'bg-transparent color-t2 hover:bg-bg2'
            "
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
              </div>
            </div>
          </button>
          <div
            v-if="pickerFiles.length === 0"
            class="p-16px text-11px color-t3 text-center"
          >
            输入目录无{{ mediaKind === "video" ? "视频" : "图片" }}
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
          class="relative shadow-lg shrink-0"
          :style="{
            width: canvasWidth * viewScale + 'px',
            height: canvasHeight * viewScale + 'px',
            background:
              background === 'transparent'
                ? 'transparent'
                : background,
          }"
          @click.stop
        >
          <div
            v-for="item in sortedItems"
            :key="item.id"
            class="absolute box-border select-none"
            :class="
              selectedId === item.id
                ? 'outline outline-2 outline-secondary z-50'
                : 'outline outline-1 outline-white/30'
            "
            :style="{
              left: item.x * viewScale + 'px',
              top: item.y * viewScale + 'px',
              width: item.width * viewScale + 'px',
              height: item.height * viewScale + 'px',
              zIndex: item.z,
              cursor: drag?.kind === 'move' && drag.id === item.id ? 'grabbing' : 'grab',
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
              draggable="false"
            />
            <div
              v-else
              class="w-full h-full grid place-items-center text-10px color-t3 bg-bg2"
            >
              …
            </div>
            <div
              class="absolute left-0 right-0 bottom-0 px-4px py-2px text-10px color-white bg-black/50 truncate pointer-events-none"
            >
              {{ item.name }}
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
          属性
        </div>
        <div class="flex-1 min-h-0 overflow-y-auto p-12px flex flex-col gap-10px">
          <template v-if="selectedItem">
            <div class="text-12px font-500 truncate" :title="selectedItem.name">
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
            从左侧点击素材加入画布，选中后可拖拽、缩放并调整属性。
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

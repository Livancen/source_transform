<script setup lang="ts">
import { computed, reactive, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import type {
  FileInfo,
  NamingOptions,
  VideoMergeLayout,
  VideoMergeOptions,
} from "../types";
import { formatFileSizeMb } from "../types";
import { useFileThumbs } from "../composables/useFileThumbs";
import MediaPreviewModal from "./MediaPreviewModal.vue";

type MergeSlotState = {
  path: string;
  name: string;
  fileType: "image" | "video" | "";
  /** 素材原始尺寸 */
  nativeWidth: number | null;
  nativeHeight: number | null;
  /** 槽位输出尺寸（fill 目标） */
  width: number | null;
  height: number | null;
  previewImage: string;
  isLoadingPreview: boolean;
};

type SlotRatioPreset = {
  id: string;
  label: string;
  /** 0 表示原比例 */
  w: number;
  h: number;
};

const SLOT_RATIO_PRESETS: SlotRatioPreset[] = [
  { id: "9:16", label: "9:16", w: 9, h: 16 },
  { id: "16:9", label: "16:9", w: 16, h: 9 },
  { id: "1:1", label: "1:1", w: 1, h: 1 },
  { id: "4:3", label: "4:3", w: 4, h: 3 },
  { id: "3:4", label: "3:4", w: 3, h: 4 },
  { id: "original", label: "原比例", w: 0, h: 0 },
];

const props = defineProps<{
  outputDir: string;
  inputFiles: FileInfo[];
  naming: NamingOptions;
}>();

const emit = defineEmits<{
  completed: [message: string];
}>();

const layout = ref<VideoMergeLayout>("vertical");
const mediaKind = ref<"video" | "image">("video");
/** 槽位比例预设 */
const slotRatioId = ref("9:16");
/** 当前从左侧列表填入的目标槽位 */
const activeSlot = ref<0 | 1>(0);
const slots = reactive<[MergeSlotState, MergeSlotState]>([
  {
    path: "",
    name: "",
    fileType: "",
    nativeWidth: null,
    nativeHeight: null,
    width: null,
    height: null,
    previewImage: "",
    isLoadingPreview: false,
  },
  {
    path: "",
    name: "",
    fileType: "",
    nativeWidth: null,
    nativeHeight: null,
    width: null,
    height: null,
    previewImage: "",
    isLoadingPreview: false,
  },
]);
const outputWidth = ref<number | null>(null);
const outputHeight = ref<number | null>(null);
const statusMessage = ref("");
const isMerging = ref(false);

const currentRatioPreset = computed(
  () =>
    SLOT_RATIO_PRESETS.find((p) => p.id === slotRatioId.value) ||
    SLOT_RATIO_PRESETS[0],
);

const slotAspectStyle = computed(() => {
  const p = currentRatioPreset.value;
  if (p.w > 0 && p.h > 0) {
    return { aspectRatio: `${p.w} / ${p.h}` };
  }
  return layout.value === "vertical"
    ? { aspectRatio: "9 / 16" }
    : { aspectRatio: "16 / 9" };
});

const slotRatioLabel = computed(() => currentRatioPreset.value.label);

const naturalWidth = computed(() => {
  if (slots.some((slot) => slot.width == null || slot.height == null))
    return null;
  return layout.value === "vertical"
    ? slots[0].width!
    : slots[0].width! + slots[1].width!;
});
const naturalHeight = computed(() => {
  if (slots.some((slot) => slot.width == null || slot.height == null))
    return null;
  return layout.value === "vertical"
    ? slots[0].height! + slots[1].height!
    : slots[0].height!;
});
const naturalSizeText = computed(() =>
  naturalWidth.value == null || naturalHeight.value == null
    ? "待选择文件"
    : `${naturalWidth.value} x ${naturalHeight.value}`,
);

const canMerge = computed(() => {
  const slotsReady = slots.every(
    (slot) =>
      slot.path &&
      slot.fileType === mediaKind.value &&
      slot.width != null &&
      slot.height != null &&
      slot.width > 0 &&
      slot.height > 0,
  );
  const outputEmpty = outputWidth.value == null && outputHeight.value == null;
  const outputReady =
    outputWidth.value != null &&
    outputHeight.value != null &&
    outputWidth.value > 0 &&
    outputHeight.value > 0;
  return (
    slotsReady &&
    props.outputDir &&
    (outputEmpty || outputReady) &&
    !isMerging.value
  );
});

const pickerFiles = computed(() =>
  props.inputFiles.filter((f) => f.file_type === mediaKind.value),
);

const pickerFilesRef = computed(() => pickerFiles.value);
const { thumbs } = useFileThumbs(pickerFilesRef);

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

const slotPaths = computed(() => new Set(slots.map((s) => s.path).filter(Boolean)));

watch(layout, () => {
  // 非原比例时按布局切换常用默认比例（会触发 reapply）
  if (slotRatioId.value !== "original") {
    const next = layout.value === "vertical" ? "9:16" : "16:9";
    if (slotRatioId.value !== next) {
      slotRatioId.value = next;
      return;
    }
  }
  reapplyAllSlotSizes();
});

watch(slotRatioId, () => {
  reapplyAllSlotSizes();
});

watch(mediaKind, () => {
  clearSlot(0);
  clearSlot(1);
  activeSlot.value = 0;
  statusMessage.value = "";
});

function evenDim(v: number) {
  const n = Math.max(2, Math.floor(v));
  return n % 2 === 0 ? n : n - 1 || 2;
}

/** 按当前比例预设，从素材原始尺寸算出槽位 fill 目标尺寸 */
function computeSlotSize(
  nativeW: number,
  nativeH: number,
): { width: number; height: number } {
  const preset = currentRatioPreset.value;
  if (preset.w <= 0 || preset.h <= 0) {
    return { width: evenDim(nativeW), height: evenDim(nativeH) };
  }
  const ratio = preset.w / preset.h;
  // 以素材较长边为基准，生成目标比例矩形，素材将 stretch fill 填满
  let width: number;
  let height: number;
  if (nativeW / nativeH >= ratio) {
    width = evenDim(nativeW);
    height = evenDim(width / ratio);
  } else {
    height = evenDim(nativeH);
    width = evenDim(height * ratio);
  }
  return { width: Math.max(2, width), height: Math.max(2, height) };
}

function applySizeToSlot(index: number) {
  const slot = slots[index];
  if (slot.nativeWidth == null || slot.nativeHeight == null) return;
  const size = computeSlotSize(slot.nativeWidth, slot.nativeHeight);
  slot.width = size.width;
  slot.height = size.height;
}

function reapplyAllSlotSizes() {
  for (let i = 0; i < 2; i++) {
    if (slots[i].path) applySizeToSlot(i);
  }
  // 布局约束：上下同宽 / 左右同高；预设比例时另一维由比例推导
  syncLayoutConstraints();
}

function syncLayoutConstraints() {
  const filled = [0, 1].filter((i) => slots[i].path && slots[i].width && slots[i].height);
  if (filled.length === 0) return;

  const preset = currentRatioPreset.value;
  const hasRatio = preset.w > 0 && preset.h > 0;
  const ratio = hasRatio ? preset.w / preset.h : 0;

  if (layout.value === "vertical") {
    const baseW = Math.max(
      ...filled.map((i) => slots[i].width || 0),
    );
    for (const i of filled) {
      slots[i].width = evenDim(baseW);
      if (hasRatio) {
        slots[i].height = evenDim(slots[i].width! / ratio);
      }
    }
  } else {
    const baseH = Math.max(
      ...filled.map((i) => slots[i].height || 0),
    );
    for (const i of filled) {
      slots[i].height = evenDim(baseH);
      if (hasRatio) {
        slots[i].width = evenDim(slots[i].height! * ratio);
      }
    }
  }
}

function syncRequiredDimension(changedIndex: number) {
  const otherIndex = changedIndex === 0 ? 1 : 0;
  if (!slots[otherIndex].path) return;
  const preset = currentRatioPreset.value;
  const hasRatio = preset.w > 0 && preset.h > 0;
  const ratio = hasRatio ? preset.w / preset.h : 0;

  if (layout.value === "vertical") {
    if (slots[changedIndex].width != null) {
      slots[otherIndex].width = slots[changedIndex].width;
      if (hasRatio && slots[otherIndex].width != null) {
        slots[otherIndex].height = evenDim(slots[otherIndex].width! / ratio);
      }
    }
  } else if (slots[changedIndex].height != null) {
    slots[otherIndex].height = slots[changedIndex].height;
    if (hasRatio && slots[otherIndex].height != null) {
      slots[otherIndex].width = evenDim(slots[otherIndex].height! * ratio);
    }
  }
}

function slotIndexOf(path: string): number {
  return slots.findIndex((s) => s.path === path);
}

async function assignFile(index: number, file: FileInfo) {
  if (file.file_type !== mediaKind.value) {
    statusMessage.value =
      mediaKind.value === "video" ? "请选择视频文件" : "请选择图片文件";
    return;
  }

  // 若文件已在另一槽位，先清空该槽
  const existing = slotIndexOf(file.path);
  if (existing >= 0 && existing !== index) {
    clearSlot(existing);
  }

  slots[index].path = file.path;
  slots[index].name = file.name;
  slots[index].fileType = file.file_type as "image" | "video";
  slots[index].previewImage = "";
  slots[index].nativeWidth = null;
  slots[index].nativeHeight = null;
  slots[index].width = null;
  slots[index].height = null;
  slots[index].isLoadingPreview = true;
  statusMessage.value = "";

  try {
    if (file.file_type === "video") {
      const dimensions = await invoke<[number, number]>("get_video_dimensions", {
        videoPath: file.path,
      });
      slots[index].nativeWidth = normalizeSize(dimensions[0]);
      slots[index].nativeHeight = normalizeSize(dimensions[1]);
      slots[index].previewImage = await invoke<string>("extract_video_frame", {
        videoPath: file.path,
      });
    } else {
      const dimensions = await invoke<[number, number]>("get_image_dimensions", {
        imagePath: file.path,
      });
      slots[index].nativeWidth = normalizeSize(dimensions[0]);
      slots[index].nativeHeight = normalizeSize(dimensions[1]);
      slots[index].previewImage = await invoke<string>("load_image_preview", {
        imagePath: file.path,
      });
    }
    applySizeToSlot(index);
    syncLayoutConstraints();

    // 填完当前槽后，若另一槽为空则自动切过去
    const other = index === 0 ? 1 : 0;
    if (!slots[other].path) {
      activeSlot.value = other as 0 | 1;
    }
  } catch (e) {
    statusMessage.value = `槽位 ${index + 1} 加载失败: ${e}`;
  } finally {
    slots[index].isLoadingPreview = false;
  }
}

/** 左侧列表点击：填入当前目标槽位 */
async function onPickFromList(file: FileInfo) {
  if (isMerging.value) return;
  await assignFile(activeSlot.value, file);
}

function selectActiveSlot(index: 0 | 1) {
  activeSlot.value = index;
}

async function pickFile(index: number) {
  activeSlot.value = index as 0 | 1;
  const filters =
    mediaKind.value === "video"
      ? [
          {
            name: "Video",
            extensions: ["mp4", "avi", "mov", "mkv", "wmv", "flv", "webm", "m4v"],
          },
        ]
      : [
          {
            name: "Image",
            extensions: ["jpg", "jpeg", "png", "gif", "bmp", "webp", "tiff"],
          },
        ];

  const selected = await open({
    multiple: false,
    filters,
    title: `选择${mediaKind.value === "video" ? "视频" : "图片"} ${index + 1}`,
  });
  if (!selected || Array.isArray(selected)) return;

  const path = selected as string;
  const name = path.split(/[\\/]/).pop() || path;
  const ext = name.split(".").pop()?.toLowerCase() || "";
  const imageExts = ["jpg", "jpeg", "png", "gif", "bmp", "webp", "tiff"];
  const fileType = imageExts.includes(ext) ? "image" : "video";
  await assignFile(index, { path, name, file_type: fileType, size_bytes: 0 });
}

function clearSlot(index: number) {
  slots[index].path = "";
  slots[index].name = "";
  slots[index].fileType = "";
  slots[index].nativeWidth = null;
  slots[index].nativeHeight = null;
  slots[index].width = null;
  slots[index].height = null;
  slots[index].previewImage = "";
  slots[index].isLoadingPreview = false;
}

function updateSlotWidth(index: number, value: string) {
  const w = parseOptionalSize(value);
  slots[index].width = w == null ? null : evenDim(w);
  const preset = currentRatioPreset.value;
  if (preset.w > 0 && preset.h > 0 && slots[index].width != null) {
    slots[index].height = evenDim(slots[index].width! / (preset.w / preset.h));
  }
  if (layout.value === "vertical") syncRequiredDimension(index);
  else if (preset.w > 0) syncLayoutConstraints();
}

function updateSlotHeight(index: number, value: string) {
  const h = parseOptionalSize(value);
  slots[index].height = h == null ? null : evenDim(h);
  const preset = currentRatioPreset.value;
  if (preset.w > 0 && preset.h > 0 && slots[index].height != null) {
    slots[index].width = evenDim(slots[index].height! * (preset.w / preset.h));
  }
  if (layout.value === "horizontal") syncRequiredDimension(index);
  else if (preset.w > 0) syncLayoutConstraints();
}

function normalizeSize(value: number) {
  return Number.isFinite(value) && value > 0 ? Math.floor(value) : 1;
}

function parseOptionalSize(value: string) {
  const trimmed = value.trim();
  if (!trimmed) return null;
  const parsed = Number(trimmed);
  return Number.isFinite(parsed) && parsed > 0 ? Math.floor(parsed) : null;
}

function clearOutputSize() {
  outputWidth.value = null;
  outputHeight.value = null;
}

function buildOutputName() {
  const parts: string[] = [];
  if (props.naming.use_original_name) parts.push("merge");
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
  if (parts.length === 0) parts.push(`merge_${Date.now()}`);
  const ext = mediaKind.value === "image" ? "png" : "mp4";
  return `${parts.join("-")}.${ext}`;
}

async function startMerge() {
  if (!canMerge.value) return;
  isMerging.value = true;
  statusMessage.value = "正在拼接...";

  const outputFileName = buildOutputName();
  const sep = props.outputDir.includes("\\") ? "\\" : "/";
  const outputPath = `${props.outputDir.replace(/[\\/]$/, "")}${sep}${outputFileName}`;

  const options: VideoMergeOptions = {
    layout: layout.value,
    slots: [
      {
        path: slots[0].path,
        name: slots[0].name,
        width: normalizeSize(slots[0].width!),
        height: normalizeSize(slots[0].height!),
      },
      {
        path: slots[1].path,
        name: slots[1].name,
        width: normalizeSize(slots[1].width!),
        height: normalizeSize(slots[1].height!),
      },
    ],
    output_path: outputPath,
    media_kind: mediaKind.value,
  };

  if (outputWidth.value != null && outputHeight.value != null) {
    options.output_width = normalizeSize(outputWidth.value);
    options.output_height = normalizeSize(outputHeight.value);
  }

  try {
    const result = await invoke<string>("merge_videos", { options });
    statusMessage.value = result;
    emit("completed", result);
  } catch (e) {
    statusMessage.value = `拼接失败: ${e}`;
  } finally {
    isMerging.value = false;
  }
}
</script>

<template>
  <div class="flex-1 min-h-0 flex flex-col bg-bg1 overflow-hidden">
    <!-- 顶栏 -->
    <div
      class="shrink-0 px-14px py-10px flex items-center justify-between gap-12px border-b border-border flex-wrap"
    >
      <div class="flex items-center gap-16px flex-wrap min-w-0">
        <div>
          <div class="text-13px font-600">拼接</div>
          <div class="text-11px color-t3 mt-2px">
            左侧选素材 · 右侧填槽位 · 素材 fill 填满槽位
          </div>
        </div>
        <div class="flex items-center gap-8px">
          <span class="text-12px color-t3 shrink-0">槽位比例</span>
          <select
            class="field w-auto! min-w-88px h-28px! px-8px! text-12px!"
            v-model="slotRatioId"
            :disabled="isMerging"
          >
            <option
              v-for="p in SLOT_RATIO_PRESETS"
              :key="p.id"
              :value="p.id"
            >
              {{ p.label }}
            </option>
          </select>
        </div>
      </div>
      <button
        class="tb-btn tb-btn-success"
        type="button"
        :disabled="!canMerge"
        @click="startMerge"
      >
        {{ isMerging ? "拼接中…" : "导出拼接" }}
      </button>
    </div>

    <div class="flex-1 min-h-0 flex overflow-hidden">
      <!-- 左侧素材列表 -->
      <div
        class="w-280px shrink-0 min-h-0 border-r border-border flex flex-col bg-bg0 overflow-hidden"
      >
        <div
          class="shrink-0 h-32px px-10px flex items-center justify-between text-11px font-500 color-t3 border-b border-border bg-bg2"
        >
          <span>选择素材 ({{ pickerFiles.length }})</span>
          <span class="color-secondary">→ 槽位 {{ activeSlot + 1 }}</span>
        </div>
        <div class="flex-1 min-h-0 overflow-y-auto overflow-x-hidden">
          <button
            v-for="f in pickerFiles"
            :key="f.path"
            type="button"
            class="w-full text-left px-10px py-8px border-none border-b border-border/60 cursor-pointer text-12px flex gap-10px items-center"
            :class="
              slotPaths.has(f.path)
                ? 'bg-secondary-soft color-secondary'
                : 'bg-transparent color-t2 hover:bg-bg2'
            "
            :disabled="isMerging"
            @click="onPickFromList(f)"
          >
            <div
              class="w-44px h-44px rounded-4px overflow-hidden bg-bg1 border border-border shrink-0 cursor-zoom-in hover:border-secondary"
              title="点击预览"
              @click="openMediaPreview(f, $event)"
            >
              <img
                v-if="thumbs[f.path]"
                :src="thumbs[f.path]"
                class="w-full h-full object-cover pointer-events-none"
                draggable="false"
              />
              <div v-else class="w-full h-full grid place-items-center text-10px color-t3 pointer-events-none">…</div>
            </div>
            <div class="min-w-0 flex-1">
              <div class="truncate font-500" :title="f.name">{{ f.name }}</div>
              <div class="text-10px color-t3 mt-2px flex items-center gap-6px flex-wrap">
                <span>{{ f.file_type === "video" ? "视频" : "图片" }}</span>
                <span>{{ formatFileSizeMb(f.size_bytes) }}</span>
                <span v-if="slotIndexOf(f.path) >= 0" class="color-secondary">
                  槽位 {{ slotIndexOf(f.path) + 1 }}
                </span>
              </div>
            </div>
          </button>
          <div
            v-if="pickerFiles.length === 0"
            class="p-16px text-11px color-t3 text-center"
          >
            输入目录无{{ mediaKind === "video" ? "视频" : "图片" }}文件
          </div>
        </div>
      </div>

      <!-- 右侧拼接配置 -->
      <div class="flex-1 min-w-0 min-h-0 overflow-y-auto p-14px flex flex-col gap-12px">
        <div
          class="flex flex-wrap items-center gap-16px py-10px px-12px bg-bg0 rounded-8px border border-border"
        >
          <div class="flex items-center gap-12px">
            <label class="flex items-center gap-6px cursor-pointer text-12px">
              <input
                type="radio"
                value="video"
                v-model="mediaKind"
                :disabled="isMerging"
                class="accent-secondary"
              />
              双视频
            </label>
            <label class="flex items-center gap-6px cursor-pointer text-12px">
              <input
                type="radio"
                value="image"
                v-model="mediaKind"
                :disabled="isMerging"
                class="accent-secondary"
              />
              双图片
            </label>
          </div>
          <div class="w-1px h-16px bg-border"></div>
          <div class="flex items-center gap-12px">
            <label class="flex items-center gap-6px cursor-pointer text-12px">
              <input
                type="radio"
                value="vertical"
                v-model="layout"
                :disabled="isMerging"
                class="accent-secondary"
              />
              上下拼接
            </label>
            <label class="flex items-center gap-6px cursor-pointer text-12px">
              <input
                type="radio"
                value="horizontal"
                v-model="layout"
                :disabled="isMerging"
                class="accent-secondary"
              />
              左右拼接
            </label>
          </div>
        </div>

        <p class="text-11px color-t3 -mt-4px">
          点击左侧文件填入高亮槽位；也可点击槽位卡片切换目标，或点预览区从本地选文件。
        </p>

        <div
          class="gap-12px grid w-full"
          :class="
            layout === 'vertical'
              ? 'grid-cols-1 max-w-360px'
              : 'grid-cols-1 sm:grid-cols-2 max-w-920px'
          "
        >
          <div
            v-for="(slot, index) in slots"
            :key="index"
            class="border rounded-8px p-10px bg-bg0 transition-colors cursor-pointer min-w-0"
            :class="
              activeSlot === index
                ? 'border-secondary ring-1 ring-secondary/30'
                : 'border-border'
            "
            @click="selectActiveSlot(index as 0 | 1)"
          >
            <div class="flex items-center justify-between mb-8px">
              <div class="text-11px font-500" :class="activeSlot === index ? 'color-secondary' : 'color-t3'">
                槽位 {{ index + 1 }}
                <span v-if="activeSlot === index" class="font-400">（当前填入）</span>
                <span class="font-400 color-t3 ml-4px">
                  {{ slotRatioLabel }} · fill
                </span>
              </div>
            </div>
            <button
              type="button"
              class="relative w-full bg-bg2 border border-border rounded-6px flex flex-col items-center justify-center text-center p-0 overflow-hidden color-t2 cursor-pointer hover:not-disabled:bg-bg-hover disabled:opacity-45 max-h-420px"
              :style="slotAspectStyle"
              :disabled="isMerging"
              @click.stop="pickFile(index)"
            >
              <img
                v-if="slot.previewImage"
                :src="slot.previewImage"
                class="absolute inset-0 w-full h-full object-fill"
                draggable="false"
              />
              <div
                v-if="slot.previewImage"
                class="absolute inset-0 bg-black/28"
              ></div>
              <div class="relative z-1 px-10px max-w-full text-13px">
                <span class="block truncate max-w-full">
                  {{
                    slot.isLoadingPreview
                      ? "加载中..."
                      : slot.name ||
                        `左侧选择或点击浏览${mediaKind === "video" ? "视频" : "图片"}`
                  }}
                </span>
              </div>
            </button>

            <div class="flex gap-8px mt-10px flex-wrap items-center">
              <label class="text-12px color-t3">宽</label>
              <input
                class="field w-90px!"
                type="number"
                min="1"
                :value="slot.width ?? ''"
                :disabled="isMerging"
                @click.stop
                @input="
                  updateSlotWidth(
                    index,
                    ($event.target as HTMLInputElement).value,
                  )
                "
              />
              <label class="text-12px color-t3">高</label>
              <input
                class="field w-90px!"
                type="number"
                min="1"
                :value="slot.height ?? ''"
                :disabled="isMerging"
                @click.stop
                @input="
                  updateSlotHeight(
                    index,
                    ($event.target as HTMLInputElement).value,
                  )
                "
              />
              <button
                class="tb-btn h-28px! px-8px! text-11px!"
                type="button"
                :disabled="isMerging || !slot.path"
                @click.stop="clearSlot(index)"
              >
                清除
              </button>
            </div>
          </div>
        </div>

        <div class="py-10px px-12px bg-bg0 rounded-8px border border-border max-w-560px">
          <div class="text-12px mb-10px color-t3">
            拼接后自然分辨率: {{ naturalSizeText }}
          </div>
          <div class="flex gap-8px flex-wrap items-center">
            <label class="text-12px color-t3">输出宽</label>
            <input
              class="field w-100px!"
              type="number"
              min="1"
              v-model.number="outputWidth"
              :disabled="isMerging"
              placeholder="可选"
            />
            <label class="text-12px color-t3">输出高</label>
            <input
              class="field w-100px!"
              type="number"
              min="1"
              v-model.number="outputHeight"
              :disabled="isMerging"
              placeholder="可选"
            />
            <button
              class="tb-btn h-28px! px-8px! text-11px!"
              type="button"
              :disabled="isMerging"
              @click="clearOutputSize"
            >
              清空
            </button>
          </div>
        </div>

        <div
          v-if="statusMessage"
          class="bg-bg0 p-10px rounded-6px whitespace-pre-wrap text-12px border border-border color-t2 max-w-560px"
        >
          {{ statusMessage }}
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

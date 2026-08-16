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

type MergeSlotState = {
  path: string;
  name: string;
  fileType: "image" | "video" | "";
  width: number | null;
  height: number | null;
  previewImage: string;
  isLoadingPreview: boolean;
};

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
const slots = reactive<[MergeSlotState, MergeSlotState]>([
  { path: "", name: "", fileType: "", width: null, height: null, previewImage: "", isLoadingPreview: false },
  { path: "", name: "", fileType: "", width: null, height: null, previewImage: "", isLoadingPreview: false },
]);
const outputWidth = ref<number | null>(null);
const outputHeight = ref<number | null>(null);
const statusMessage = ref("");
const isMerging = ref(false);

const naturalWidth = computed(() => {
  if (slots.some((slot) => slot.width == null || slot.height == null)) return null;
  return layout.value === "vertical" ? slots[0].width! : slots[0].width! + slots[1].width!;
});
const naturalHeight = computed(() => {
  if (slots.some((slot) => slot.width == null || slot.height == null)) return null;
  return layout.value === "vertical" ? slots[0].height! + slots[1].height! : slots[0].height!;
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
  return slotsReady && props.outputDir && (outputEmpty || outputReady) && !isMerging.value;
});

const pickerFiles = computed(() =>
  props.inputFiles.filter((f) => f.file_type === mediaKind.value),
);

watch(layout, () => {
  syncRequiredDimension(slots[0].path ? 0 : 1);
});

watch(mediaKind, () => {
  clearSlot(0);
  clearSlot(1);
  statusMessage.value = "";
});

function syncRequiredDimension(changedIndex: number) {
  const otherIndex = changedIndex === 0 ? 1 : 0;
  if (!slots[otherIndex].path) return;
  if (layout.value === "vertical") {
    if (slots[changedIndex].width != null) {
      slots[otherIndex].width = slots[changedIndex].width;
    }
  } else if (slots[changedIndex].height != null) {
    slots[otherIndex].height = slots[changedIndex].height;
  }
}

async function assignFile(index: number, file: FileInfo) {
  if (file.file_type !== mediaKind.value) {
    statusMessage.value = mediaKind.value === "video" ? "请选择视频文件" : "请选择图片文件";
    return;
  }
  slots[index].path = file.path;
  slots[index].name = file.name;
  slots[index].fileType = file.file_type as "image" | "video";
  slots[index].previewImage = "";
  slots[index].width = null;
  slots[index].height = null;
  slots[index].isLoadingPreview = true;
  statusMessage.value = "";

  try {
    if (file.file_type === "video") {
      const dimensions = await invoke<[number, number]>("get_video_dimensions", {
        videoPath: file.path,
      });
      slots[index].width = normalizeSize(dimensions[0]);
      slots[index].height = normalizeSize(dimensions[1]);
      slots[index].previewImage = await invoke<string>("extract_video_frame", {
        videoPath: file.path,
      });
    } else {
      const dimensions = await invoke<[number, number]>("get_image_dimensions", {
        imagePath: file.path,
      });
      slots[index].width = normalizeSize(dimensions[0]);
      slots[index].height = normalizeSize(dimensions[1]);
      slots[index].previewImage = await invoke<string>("load_image_preview", {
        imagePath: file.path,
      });
    }
    syncRequiredDimension(index);
  } catch (e) {
    statusMessage.value = `槽位 ${index + 1} 加载失败: ${e}`;
  } finally {
    slots[index].isLoadingPreview = false;
  }
}

async function pickFile(index: number) {
  const filters =
    mediaKind.value === "video"
      ? [{ name: "Video", extensions: ["mp4", "avi", "mov", "mkv", "wmv", "flv", "webm", "m4v"] }]
      : [{ name: "Image", extensions: ["jpg", "jpeg", "png", "gif", "bmp", "webp", "tiff"] }];

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
  await assignFile(index, { path, name, file_type: fileType });
}

function clearSlot(index: number) {
  slots[index].path = "";
  slots[index].name = "";
  slots[index].fileType = "";
  slots[index].width = null;
  slots[index].height = null;
  slots[index].previewImage = "";
  slots[index].isLoadingPreview = false;
}

function updateSlotWidth(index: number, value: string) {
  slots[index].width = parseOptionalSize(value);
  if (layout.value === "vertical") syncRequiredDimension(index);
}

function updateSlotHeight(index: number, value: string) {
  slots[index].height = parseOptionalSize(value);
  if (layout.value === "horizontal") syncRequiredDimension(index);
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
  if (props.naming.custom_text.trim()) parts.push(props.naming.custom_text.trim());
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
  <div class="shrink-0 border-b border-border bg-bg1">
    <div class="px-14px py-12px flex flex-col gap-12px max-h-480px overflow-auto">
      <div class="flex items-center justify-between gap-12px flex-wrap">
        <div>
          <div class="text-13px font-600">拼接</div>
          <div class="text-11px color-t3 mt-2px">双视频或双图片 · 不混排 · 上下/左右</div>
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

      <div class="flex flex-wrap items-center gap-16px py-10px px-12px bg-bg0 rounded-8px border border-border">
        <div class="flex items-center gap-12px">
          <label class="flex items-center gap-6px cursor-pointer text-12px">
            <input type="radio" value="video" v-model="mediaKind" :disabled="isMerging" class="accent-secondary" />
            双视频
          </label>
          <label class="flex items-center gap-6px cursor-pointer text-12px">
            <input type="radio" value="image" v-model="mediaKind" :disabled="isMerging" class="accent-secondary" />
            双图片
          </label>
        </div>
        <div class="w-1px h-16px bg-border"></div>
        <div class="flex items-center gap-12px">
          <label class="flex items-center gap-6px cursor-pointer text-12px">
            <input type="radio" value="vertical" v-model="layout" :disabled="isMerging" class="accent-secondary" />
            上下拼接
          </label>
          <label class="flex items-center gap-6px cursor-pointer text-12px">
            <input type="radio" value="horizontal" v-model="layout" :disabled="isMerging" class="accent-secondary" />
            左右拼接
          </label>
        </div>
      </div>

      <div class="gap-10px grid" :class="layout === 'vertical' ? 'grid-cols-1 max-w-520px' : 'grid-cols-1 md:grid-cols-2'">
        <div
          v-for="(slot, index) in slots"
          :key="index"
          class="border border-border rounded-8px p-10px bg-bg0"
        >
          <div class="text-11px color-t3 mb-8px">槽位 {{ index + 1 }}</div>
          <button
            type="button"
            class="relative w-full h-140px bg-bg2 border border-border rounded-6px flex flex-col items-center justify-center text-center p-0 overflow-hidden color-t2 cursor-pointer hover:not-disabled:bg-bg-hover disabled:opacity-45"
            :disabled="isMerging"
            @click="pickFile(index)"
          >
            <img
              v-if="slot.previewImage"
              :src="slot.previewImage"
              class="absolute inset-0 w-full h-full object-cover"
              draggable="false"
            />
            <div v-if="slot.previewImage" class="absolute inset-0 bg-black/28"></div>
            <div class="relative z-1 px-10px max-w-full text-13px">
              <span class="block truncate max-w-full">
                {{ slot.isLoadingPreview ? "加载中..." : slot.name || `点击选择${mediaKind === "video" ? "视频" : "图片"}` }}
              </span>
            </div>
          </button>

          <div v-if="pickerFiles.length" class="mt-8px flex flex-wrap gap-4px max-h-64px overflow-auto">
            <button
              v-for="f in pickerFiles.slice(0, 12)"
              :key="f.path"
              type="button"
              class="text-10px px-6px py-2px rounded-4px border border-border bg-bg1 color-t2 cursor-pointer hover:border-secondary hover:color-secondary"
              :disabled="isMerging"
              @click="assignFile(index, f)"
            >{{ f.name }}</button>
          </div>

          <div class="flex gap-8px mt-10px flex-wrap items-center">
            <label class="text-12px color-t3">宽</label>
            <input
              class="field w-90px!"
              type="number"
              min="1"
              :value="slot.width ?? ''"
              :disabled="isMerging"
              @input="updateSlotWidth(index, ($event.target as HTMLInputElement).value)"
            />
            <label class="text-12px color-t3">高</label>
            <input
              class="field w-90px!"
              type="number"
              min="1"
              :value="slot.height ?? ''"
              :disabled="isMerging"
              @input="updateSlotHeight(index, ($event.target as HTMLInputElement).value)"
            />
            <button
              class="tb-btn h-28px! px-8px! text-11px!"
              type="button"
              :disabled="isMerging || !slot.path"
              @click="clearSlot(index)"
            >清除</button>
          </div>
        </div>
      </div>

      <div class="py-10px px-12px bg-bg0 rounded-8px border border-border">
        <div class="text-12px mb-10px color-t3">拼接后自然分辨率: {{ naturalSizeText }}</div>
        <div class="flex gap-8px flex-wrap items-center">
          <label class="text-12px color-t3">输出宽</label>
          <input class="field w-100px!" type="number" min="1" v-model.number="outputWidth" :disabled="isMerging" placeholder="可选" />
          <label class="text-12px color-t3">输出高</label>
          <input class="field w-100px!" type="number" min="1" v-model.number="outputHeight" :disabled="isMerging" placeholder="可选" />
          <button class="tb-btn h-28px! px-8px! text-11px!" type="button" :disabled="isMerging" @click="clearOutputSize">清空</button>
        </div>
      </div>

      <div
        v-if="statusMessage"
        class="bg-bg0 p-10px rounded-6px whitespace-pre-wrap text-12px border border-border color-t2"
      >
        {{ statusMessage }}
      </div>
    </div>
  </div>
</template>

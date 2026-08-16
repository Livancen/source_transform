<script setup lang="ts">
import { computed, reactive, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import type { VideoMergeLayout, VideoMergeOptions } from "../types";

type MergeSlotState = {
  path: string;
  name: string;
  width: number | null;
  height: number | null;
  previewImage: string;
  isLoadingPreview: boolean;
};

const props = defineProps<{
  visible: boolean;
  outputDir: string;
}>();

const emit = defineEmits<{
  close: [];
  completed: [message: string];
}>();

const videoFilters = [
  { name: "Video", extensions: ["mp4", "avi", "mov", "mkv", "wmv", "flv", "webm", "m4v"] },
];

const layout = ref<VideoMergeLayout>("vertical");
const slots = reactive<[MergeSlotState, MergeSlotState]>([
  { path: "", name: "", width: null, height: null, previewImage: "", isLoadingPreview: false },
  { path: "", name: "", width: null, height: null, previewImage: "", isLoadingPreview: false },
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
    ? "待选择视频"
    : `${naturalWidth.value} x ${naturalHeight.value}`,
);
const canMerge = computed(() => {
  const slotsReady = slots.every(
    (slot) => slot.path && slot.width != null && slot.height != null && slot.width > 0 && slot.height > 0,
  );
  const outputEmpty = outputWidth.value == null && outputHeight.value == null;
  const outputReady =
    outputWidth.value != null &&
    outputHeight.value != null &&
    outputWidth.value > 0 &&
    outputHeight.value > 0;
  return slotsReady && props.outputDir && (outputEmpty || outputReady) && !isMerging.value;
});

watch(layout, () => {
  syncRequiredDimension(slots[0].path ? 0 : 1);
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

async function selectVideo(index: number) {
  const selected = await open({
    multiple: false,
    filters: videoFilters,
    title: `选择视频 ${index + 1}`,
  });
  if (!selected || Array.isArray(selected)) return;

  slots[index].path = selected;
  slots[index].name = selected.split(/[\\/]/).pop() || selected;
  slots[index].previewImage = "";
  slots[index].width = null;
  slots[index].height = null;
  slots[index].isLoadingPreview = true;
  statusMessage.value = "";

  try {
    const dimensions = await invoke<[number, number]>("get_video_dimensions", {
      videoPath: selected,
    });
    slots[index].width = normalizeSize(dimensions[0]);
    slots[index].height = normalizeSize(dimensions[1]);
    syncRequiredDimension(index);
  } catch (e) {
    statusMessage.value = `视频 ${index + 1} 分辨率读取失败: ${e}`;
    slots[index].isLoadingPreview = false;
    return;
  }

  try {
    slots[index].previewImage = await invoke<string>("extract_video_frame", {
      videoPath: selected,
    });
  } catch (e) {
    statusMessage.value = `视频 ${index + 1} 预览生成失败: ${e}`;
  } finally {
    slots[index].isLoadingPreview = false;
  }
}

function clearVideo(index: number) {
  slots[index].path = "";
  slots[index].name = "";
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

function closeModal() {
  if (isMerging.value) return;
  statusMessage.value = "";
  emit("close");
}

async function startMerge() {
  if (!canMerge.value) return;
  isMerging.value = true;
  statusMessage.value = "正在拼接...";

  const outputFileName = `merge_${formatTimestamp(new Date())}.mp4`;
  const outputPath = `${props.outputDir.replace(/[\\/]$/, "")}\\${outputFileName}`;
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

function formatTimestamp(date: Date) {
  const pad = (value: number) => value.toString().padStart(2, "0");
  return `${date.getFullYear()}${pad(date.getMonth() + 1)}${pad(date.getDate())}_${pad(date.getHours())}${pad(date.getMinutes())}${pad(date.getSeconds())}`;
}
</script>

<template>
  <div v-if="visible" class="modal-mask">
    <div class="modal-panel p-20px w-760px">
      <div class="flex justify-between items-center mb-14px">
        <h3 class="text-16px font-600">视频拼接</h3>
        <button class="modal-btn" type="button" :disabled="isMerging" @click="closeModal">关闭</button>
      </div>

      <div class="mb-14px py-10px px-12px bg-bg2 rounded-8px border border-border">
        <div class="flex items-center gap-16px">
          <label class="flex items-center gap-6px cursor-pointer text-12px">
            <input type="radio" value="vertical" v-model="layout" :disabled="isMerging" class="accent-accent" />
            上下拼接
          </label>
          <label class="flex items-center gap-6px cursor-pointer text-12px">
            <input type="radio" value="horizontal" v-model="layout" :disabled="isMerging" class="accent-accent" />
            左右拼接
          </label>
        </div>
      </div>

      <div
        class="gap-10px mb-14px grid"
        :class="layout === 'vertical' ? 'grid-cols-1' : 'grid-cols-2'"
      >
        <div
          v-for="(slot, index) in slots"
          :key="index"
          class="border border-border rounded-8px p-10px bg-bg2"
        >
          <button
            type="button"
            class="relative w-full h-150px bg-#111 border border-border rounded-6px flex flex-col items-center justify-center text-center p-0 overflow-hidden color-t2 cursor-pointer hover:not-disabled:bg-#1a1a1a disabled:opacity-45 disabled:cursor-not-allowed"
            :disabled="isMerging"
            @click="selectVideo(index)"
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
                {{ slot.isLoadingPreview ? "正在生成预览..." : slot.name || "点击添加视频" }}
              </span>
              <span
                v-if="slot.path && !slot.previewImage"
                class="block text-11px color-t3 mt-6px break-all whitespace-normal"
              >{{ slot.path }}</span>
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
              class="modal-btn"
              type="button"
              :disabled="isMerging || !slot.path"
              @click="clearVideo(index)"
            >清除</button>
          </div>
        </div>
      </div>

      <div class="py-10px px-12px bg-bg2 rounded-8px mb-14px border border-border">
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
          <button class="modal-btn" type="button" :disabled="isMerging" @click="clearOutputSize">清空</button>
        </div>
        <div class="text-11px color-t3 mt-8px">
          不填写输出分辨率时使用自然分辨率，输出文件不包含音轨。
        </div>
      </div>

      <div
        v-if="statusMessage"
        class="mb-14px bg-bg0 p-10px rounded-6px whitespace-pre-wrap text-12px border border-border color-t2"
      >
        {{ statusMessage }}
      </div>

      <div class="flex justify-end gap-8px">
        <button class="modal-btn" type="button" :disabled="isMerging" @click="closeModal">取消</button>
        <button class="modal-btn modal-btn-primary" type="button" :disabled="!canMerge" @click="startMerge">
          {{ isMerging ? "拼接中..." : "开始拼接" }}
        </button>
      </div>
    </div>
  </div>
</template>

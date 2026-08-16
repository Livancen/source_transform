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
  } else {
    if (slots[changedIndex].height != null) {
      slots[otherIndex].height = slots[changedIndex].height;
    }
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
    <div class="modal-panel" style="padding: 20px; width: 760px">
      <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 14px">
        <h3>视频拼接</h3>
        <button class="modal-btn ghost" type="button" :disabled="isMerging" @click="closeModal">关闭</button>
      </div>

      <div style="margin-bottom: 14px; padding: 10px 12px; background: var(--bg-2); border-radius: 8px; border: 1px solid var(--border)">
        <div style="display: flex; align-items: center; gap: 16px">
          <label style="display: flex; align-items: center; gap: 6px; cursor: pointer; font-size: 12px">
            <input type="radio" value="vertical" v-model="layout" :disabled="isMerging" style="accent-color: var(--accent)" />
            上下拼接
          </label>
          <label style="display: flex; align-items: center; gap: 6px; cursor: pointer; font-size: 12px">
            <input type="radio" value="horizontal" v-model="layout" :disabled="isMerging" style="accent-color: var(--accent)" />
            左右拼接
          </label>
        </div>
      </div>

      <div
        style="gap: 10px; margin-bottom: 14px; display: grid"
        :style="layout === 'vertical' ? { gridTemplateColumns: '1fr' } : { gridTemplateColumns: '1fr 1fr' }"
      >
        <div
          v-for="(slot, index) in slots"
          :key="index"
          style="border: 1px solid var(--border); border-radius: 8px; padding: 10px; background: var(--bg-2)"
        >
          <button
            type="button"
            class="slot-preview"
            :disabled="isMerging"
            @click="selectVideo(index)"
          >
            <img
              v-if="slot.previewImage"
              :src="slot.previewImage"
              class="slot-img"
              draggable="false"
            />
            <div v-if="slot.previewImage" class="slot-overlay"></div>
            <div class="slot-label">
              <span>
                {{ slot.isLoadingPreview ? "正在生成预览..." : slot.name || "点击添加视频" }}
              </span>
              <span v-if="slot.path && !slot.previewImage" class="slot-path">{{ slot.path }}</span>
            </div>
          </button>

          <div style="display: flex; gap: 8px; margin-top: 10px; flex-wrap: wrap; align-items: center">
            <label style="font-size: 12px; color: var(--text-3)">宽</label>
            <input
              class="field"
              type="number"
              min="1"
              :value="slot.width ?? ''"
              :disabled="isMerging"
              style="width: 90px"
              @input="updateSlotWidth(index, ($event.target as HTMLInputElement).value)"
            />
            <label style="font-size: 12px; color: var(--text-3)">高</label>
            <input
              class="field"
              type="number"
              min="1"
              :value="slot.height ?? ''"
              :disabled="isMerging"
              style="width: 90px"
              @input="updateSlotHeight(index, ($event.target as HTMLInputElement).value)"
            />
            <button class="modal-btn" type="button" :disabled="isMerging || !slot.path" @click="clearVideo(index)">清除</button>
          </div>
        </div>
      </div>

      <div style="padding: 10px 12px; background: var(--bg-2); border-radius: 8px; margin-bottom: 14px; border: 1px solid var(--border)">
        <div style="font-size: 12px; margin-bottom: 10px; color: var(--text-3)">
          拼接后自然分辨率: {{ naturalSizeText }}
        </div>
        <div style="display: flex; gap: 8px; flex-wrap: wrap; align-items: center">
          <label style="font-size: 12px; color: var(--text-3)">输出宽</label>
          <input
            class="field"
            type="number"
            min="1"
            v-model.number="outputWidth"
            :disabled="isMerging"
            placeholder="可选"
            style="width: 100px"
          />
          <label style="font-size: 12px; color: var(--text-3)">输出高</label>
          <input
            class="field"
            type="number"
            min="1"
            v-model.number="outputHeight"
            :disabled="isMerging"
            placeholder="可选"
            style="width: 100px"
          />
          <button class="modal-btn" type="button" :disabled="isMerging" @click="clearOutputSize">清空</button>
        </div>
        <div style="font-size: 11px; color: var(--text-3); margin-top: 8px">
          不填写输出分辨率时使用自然分辨率，输出文件不包含音轨。
        </div>
      </div>

      <div
        v-if="statusMessage"
        style="margin-bottom: 14px; background: var(--bg-0); padding: 10px; border-radius: 6px; white-space: pre-wrap; font-size: 12px; border: 1px solid var(--border); color: var(--text-2)"
      >
        {{ statusMessage }}
      </div>

      <div style="display: flex; justify-content: flex-end; gap: 8px">
        <button class="modal-btn" type="button" :disabled="isMerging" @click="closeModal">取消</button>
        <button class="modal-btn primary" type="button" :disabled="!canMerge" @click="startMerge">
          {{ isMerging ? "拼接中..." : "开始拼接" }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.slot-preview {
  position: relative;
  width: 100%;
  height: 150px;
  background: #111;
  border: 1px solid var(--border);
  border-radius: 6px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  text-align: center;
  padding: 0;
  overflow: hidden;
  color: var(--text-2);
}

.slot-preview:hover:not(:disabled) {
  background: #1a1a1a;
}

.slot-img {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.slot-overlay {
  position: absolute;
  inset: 0;
  background: rgba(0, 0, 0, 0.28);
}

.slot-label {
  position: relative;
  z-index: 1;
  padding: 0 10px;
  max-width: 100%;
  font-size: 13px;
}

.slot-label span {
  display: block;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.slot-path {
  font-size: 11px;
  color: var(--text-3);
  margin-top: 6px;
  white-space: normal !important;
  word-break: break-all;
}
</style>

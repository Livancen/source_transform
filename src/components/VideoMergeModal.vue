<script setup lang="ts">
import { computed, reactive, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import type { VideoMergeLayout, VideoMergeOptions, VideoMergeSlot } from "../types";

type MergeSlotState = VideoMergeSlot & {
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
  { path: "", name: "", width: 1080, height: 960, previewImage: "", isLoadingPreview: false },
  { path: "", name: "", width: 1080, height: 960, previewImage: "", isLoadingPreview: false },
]);
const outputWidth = ref<number | null>(null);
const outputHeight = ref<number | null>(null);
const statusMessage = ref("");
const isMerging = ref(false);

const naturalWidth = computed(() =>
  layout.value === "vertical" ? slots[0].width : slots[0].width + slots[1].width,
);
const naturalHeight = computed(() =>
  layout.value === "vertical" ? slots[0].height + slots[1].height : slots[0].height,
);
const canMerge = computed(() => {
  const slotsReady = slots.every((slot) => slot.path && slot.width > 0 && slot.height > 0);
  const outputEmpty = outputWidth.value == null && outputHeight.value == null;
  const outputReady =
    outputWidth.value != null &&
    outputHeight.value != null &&
    outputWidth.value > 0 &&
    outputHeight.value > 0;
  return slotsReady && props.outputDir && (outputEmpty || outputReady) && !isMerging.value;
});

watch(layout, () => {
  syncRequiredDimension(0);
});

function syncRequiredDimension(changedIndex: number) {
  const otherIndex = changedIndex === 0 ? 1 : 0;
  if (layout.value === "vertical") {
    slots[otherIndex].width = slots[changedIndex].width;
  } else {
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
  slots[index].isLoadingPreview = true;

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
  slots[index].previewImage = "";
  slots[index].isLoadingPreview = false;
}

function updateSlotWidth(index: number, value: number) {
  slots[index].width = normalizeSize(value);
  if (layout.value === "vertical") syncRequiredDimension(index);
}

function updateSlotHeight(index: number, value: number) {
  slots[index].height = normalizeSize(value);
  if (layout.value === "horizontal") syncRequiredDimension(index);
}

function normalizeSize(value: number) {
  return Number.isFinite(value) && value > 0 ? Math.floor(value) : 1;
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
        width: normalizeSize(slots[0].width),
        height: normalizeSize(slots[0].height),
      },
      {
        path: slots[1].path,
        name: slots[1].name,
        width: normalizeSize(slots[1].width),
        height: normalizeSize(slots[1].height),
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
  <div v-if="visible" class="fixed inset-0 bg-black/70 flex items-center justify-center z-1000">
    <div class="bg-white rounded-8px p-20px w-760px max-w-94% max-h-92% overflow-auto dark:bg-#2d2d2d dark:color-#f6f6f6">
      <div class="flex justify-between items-center mb-15px">
        <h3 class="text-18px">视频拼接</h3>
        <button class="p-4px-10px bg-#6c757d" :disabled="isMerging" @click="closeModal">关闭</button>
      </div>

      <div class="mb-15px p-10px bg-#f9f9f9 rounded-4px dark:bg-#333">
        <div class="flex items-center gap-12px">
          <label class="flex items-center gap-6px cursor-pointer">
            <input type="radio" value="vertical" v-model="layout" :disabled="isMerging" />
            上下拼接
          </label>
          <label class="flex items-center gap-6px cursor-pointer">
            <input type="radio" value="horizontal" v-model="layout" :disabled="isMerging" />
            左右拼接
          </label>
        </div>
      </div>

      <div
        class="gap-10px mb-15px"
        :class="layout === 'vertical' ? 'flex flex-col' : 'grid grid-cols-2'"
      >
        <div
          v-for="(slot, index) in slots"
          :key="index"
          class="border border-#ddd rounded-6px p-10px bg-#f8f9fa dark:bg-#333 dark:border-#444"
        >
          <button
            class="relative w-full h-150px bg-#222 hover:not-disabled:bg-#333 flex flex-col items-center justify-center text-center p-0 overflow-hidden"
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
            <div class="relative z-1 px-10px max-w-full">
              <span class="text-14px block truncate max-w-full">
                {{ slot.isLoadingPreview ? '正在生成预览...' : slot.name || '点击添加视频' }}
              </span>
              <span v-if="slot.path && !slot.previewImage" class="text-11px color-#ccc mt-6px break-all block">{{ slot.path }}</span>
            </div>
          </button>

          <div class="flex gap-8px mt-10px flex-wrap items-center">
            <label class="text-12px color-#666 dark:color-#aaa">宽</label>
            <input
              type="number"
              min="1"
              :value="slot.width"
              :disabled="isMerging"
              class="w-90px p-5px border border-#ddd rounded-4px dark:bg-#444 dark:border-#555 dark:color-#f6f6f6"
              @input="updateSlotWidth(index, Number(($event.target as HTMLInputElement).value))"
            />
            <label class="text-12px color-#666 dark:color-#aaa">高</label>
            <input
              type="number"
              min="1"
              :value="slot.height"
              :disabled="isMerging"
              class="w-90px p-5px border border-#ddd rounded-4px dark:bg-#444 dark:border-#555 dark:color-#f6f6f6"
              @input="updateSlotHeight(index, Number(($event.target as HTMLInputElement).value))"
            />
            <button class="p-4px-10px text-12px bg-#6c757d" :disabled="isMerging || !slot.path" @click="clearVideo(index)">清除</button>
          </div>
        </div>
      </div>

      <div class="p-10px bg-#f9f9f9 rounded-4px mb-15px dark:bg-#333">
        <div class="text-13px mb-10px color-#666 dark:color-#aaa">
          拼接后自然分辨率: {{ naturalWidth }} x {{ naturalHeight }}
        </div>
        <div class="flex gap-8px flex-wrap items-center">
          <label class="text-12px color-#666 dark:color-#aaa">输出宽</label>
          <input
            type="number"
            min="1"
            v-model.number="outputWidth"
            :disabled="isMerging"
            placeholder="可选"
            class="w-100px p-5px border border-#ddd rounded-4px dark:bg-#444 dark:border-#555 dark:color-#f6f6f6"
          />
          <label class="text-12px color-#666 dark:color-#aaa">输出高</label>
          <input
            type="number"
            min="1"
            v-model.number="outputHeight"
            :disabled="isMerging"
            placeholder="可选"
            class="w-100px p-5px border border-#ddd rounded-4px dark:bg-#444 dark:border-#555 dark:color-#f6f6f6"
          />
          <button class="p-4px-10px text-12px bg-#6c757d" :disabled="isMerging" @click="clearOutputSize">清空</button>
        </div>
        <div class="text-12px color-#999 mt-8px dark:color-#888">
          不填写输出分辨率时使用自然分辨率，输出文件不包含音轨。
        </div>
      </div>

      <div v-if="statusMessage" class="mb-15px bg-#f8f9fa p-10px rounded-4px whitespace-pre-wrap text-12px dark:bg-#333">
        {{ statusMessage }}
      </div>

      <div class="flex justify-end gap-10px">
        <button class="bg-#6c757d" :disabled="isMerging" @click="closeModal">取消</button>
        <button class="bg-#007bff hover:not-disabled:bg-#0056b3" :disabled="!canMerge" @click="startMerge">
          {{ isMerging ? '拼接中...' : '开始拼接' }}
        </button>
      </div>
    </div>
  </div>
</template>

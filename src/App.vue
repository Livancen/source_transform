<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";

import type { FileInfo, ProcessProgress, ProcessOptions } from "./types";
import { useCrop } from "./composables/useCrop";

import AppToolbar from "./components/AppToolbar.vue";
import OptionsStrip from "./components/OptionsStrip.vue";
import FilePane from "./components/FilePane.vue";
import StatusBar from "./components/StatusBar.vue";
import CropPreviewModal from "./components/CropPreviewModal.vue";
import VideoMergeModal from "./components/VideoMergeModal.vue";

const inputDir = ref("");
const outputDir = ref("");
const files = ref<FileInfo[]>([]);
const outputFiles = ref<FileInfo[]>([]);
const selectedInputPath = ref("");
const selectedOutputPath = ref("");

const options = ref<ProcessOptions>({
  compress: false,
  compress_quality: 80,
  compress_resize: false,
  compress_width: 1280,
  compress_height: 720,
  reduce_resolution: false,
  target_width: 1920,
  target_height: 1080,
  reduce_bitrate: false,
  target_bitrate: "2M",
  reduce_level: false,
  target_level: "4.0",
  target_profile: "high",
  convert_h265_to_h264: false,
  convert_format: false,
  target_format: "mp4",
  crop: false,
  crop_width: 1280,
  crop_height: 720,
  crop_x: 0,
  crop_y: 0,
  rotate: false,
  rotation_degrees: 90,
  mute: false,
  change_framerate: false,
  target_framerate: 30,
});

const isProcessing = ref(false);
const progress = ref<ProcessProgress | null>(null);
const resultMessage = ref("");
const videoMergeVisible = ref(false);
const uploadUrl = ref("");
const optionsOpen = ref(true);

const RATIO_STORAGE_KEY = "aspect-ratio-crop-ratios";
const enableRatioCrop = ref(false);
const ratios = ref<string[]>([]);
const newRatio = ref("");
const ratioError = ref("");

const imageCount = computed(
  () => files.value.filter((f) => f.file_type === "image").length,
);
const videoCount = computed(
  () => files.value.filter((f) => f.file_type === "video").length,
);
const videoFiles = computed(() =>
  files.value.filter((f) => f.file_type === "video"),
);

const statusMessage = computed(() => {
  if (isProcessing.value && progress.value) {
    return `正在处理 ${progress.value.current}/${progress.value.total} · ${progress.value.current_file || progress.value.status || ""}`;
  }
  if (resultMessage.value) {
    const oneLine = resultMessage.value.replace(/\s+/g, " ").trim();
    return oneLine.length > 120 ? oneLine.slice(0, 120) + "…" : oneLine;
  }
  return "就绪";
});

const leftPanePct = ref(50);
const isDraggingSplitter = ref(false);
const panesRef = ref<HTMLElement | null>(null);

function addRatio() {
  ratioError.value = "";
  const val = newRatio.value.trim();
  if (!val) {
    ratioError.value = "请输入比例";
    return;
  }
  const parts = val.split(":");
  if (parts.length !== 2 || !Number(parts[0]) || !Number(parts[1])) {
    ratioError.value = "格式错误，应为 W:H 如 1:1";
    return;
  }
  if (ratios.value.includes(val)) {
    ratioError.value = "该比例已存在";
    return;
  }
  ratios.value.push(val);
  newRatio.value = "";
}

function removeRatio(i: number) {
  ratios.value.splice(i, 1);
}

function openVideoMerge() {
  if (!outputDir.value) {
    resultMessage.value = "请先选择输出目录";
    return;
  }
  videoMergeVisible.value = true;
}

function handleVideoMergeCompleted(message: string) {
  resultMessage.value = message;
  scanOutputFiles();
}

const {
  cropPreviewVisible,
  cropFrameImage,
  cropVideoWidth,
  cropVideoHeight,
  previewScale,
  openCropPreview,
  closeCropPreview,
  confirmCrop,
  startDrag,
  startResize,
  handleMouseMove,
  handleMouseUp,
} = useCrop(options, videoFiles, (msg) => {
  resultMessage.value = msg;
});

onMounted(async () => {
  const saved = localStorage.getItem(RATIO_STORAGE_KEY);
  if (saved) {
    try {
      ratios.value = JSON.parse(saved);
    } catch {
      /* ignore */
    }
  }

  try {
    const dirs = await invoke<[string, string]>("get_custom_dirs");
    inputDir.value = dirs[0];
    outputDir.value = dirs[1];
    await scanAllFiles();

    const url = await invoke<string>("start_upload_server", {
      inputDir: inputDir.value,
    });
    uploadUrl.value = url;
  } catch (e) {
    console.error("初始化失败:", e);
  }

  await listen<ProcessProgress>("process-progress", (event) => {
    progress.value = event.payload;
  });
  await listen<ProcessProgress>("crop-progress", (event) => {
    progress.value = event.payload;
  });

  window.addEventListener("mousemove", onSplitterMove);
  window.addEventListener("mouseup", onSplitterUp);
});

onUnmounted(() => {
  window.removeEventListener("mousemove", onSplitterMove);
  window.removeEventListener("mouseup", onSplitterUp);
});

watch(
  ratios,
  (val) => {
    localStorage.setItem(RATIO_STORAGE_KEY, JSON.stringify(val));
  },
  { deep: true },
);

async function selectInputDir() {
  const selected = await open({
    directory: true,
    title: "选择输入目录",
  });
  if (selected) {
    inputDir.value = selected as string;
    await saveCustomDirs();
    await scanFiles();
  }
}

async function selectOutputDir() {
  const selected = await open({
    directory: true,
    title: "选择输出目录",
  });
  if (selected) {
    outputDir.value = selected as string;
    await saveCustomDirs();
    await scanOutputFiles();
  }
}

async function saveCustomDirs() {
  try {
    await invoke("set_custom_dirs", {
      inputPath: inputDir.value,
      outputPath: outputDir.value,
    });
  } catch (e) {
    console.error("保存目录失败:", e);
  }
}

async function scanFiles() {
  if (!inputDir.value) return;
  try {
    files.value = await invoke<FileInfo[]>("scan_input_files", {
      inputDir: inputDir.value,
    });
  } catch (e) {
    console.error("扫描文件失败:", e);
    files.value = [];
  }
}

async function scanOutputFiles() {
  if (!outputDir.value) {
    outputFiles.value = [];
    return;
  }
  try {
    outputFiles.value = await invoke<FileInfo[]>("scan_input_files", {
      inputDir: outputDir.value,
    });
  } catch (e) {
    console.error("扫描输出文件失败:", e);
    outputFiles.value = [];
  }
}

async function scanAllFiles() {
  await Promise.all([scanFiles(), scanOutputFiles()]);
}

async function startProcess() {
  if (files.value.length === 0) {
    resultMessage.value = "没有可处理的文件";
    return;
  }

  isProcessing.value = true;
  progress.value = null;
  resultMessage.value = "";

  const messages: string[] = [];

  if (enableRatioCrop.value && ratios.value.length > 0) {
    try {
      const cropResult = await invoke<string>("crop_videos_by_ratios", {
        inputDir: inputDir.value,
        outputDir: outputDir.value,
        ratios: ratios.value,
      });
      messages.push(cropResult);
    } catch (e) {
      messages.push(`比例裁剪失败: ${e}`);
    }
  } else {
    try {
      const result = await invoke<string>("process_files", {
        inputDir: inputDir.value,
        outputDir: outputDir.value,
        options: options.value,
      });
      messages.push(result);
    } catch (e) {
      messages.push(`处理失败: ${e}`);
    }
  }

  resultMessage.value = messages.join("");
  isProcessing.value = false;
  progress.value = null;
  await scanOutputFiles();
}

async function openFolder(path: string) {
  try {
    await invoke("open_folder", { path });
  } catch (e) {
    console.error("打开文件夹失败:", e);
  }
}

function onSplitterDown(e: MouseEvent) {
  isDraggingSplitter.value = true;
  e.preventDefault();
}

function onSplitterMove(e: MouseEvent) {
  if (!isDraggingSplitter.value || !panesRef.value) return;
  const rect = panesRef.value.getBoundingClientRect();
  let x = e.clientX - rect.left;
  const min = 220;
  const max = rect.width - 220;
  x = Math.max(min, Math.min(max, x));
  leftPanePct.value = (x / rect.width) * 100;
}

function onSplitterUp() {
  isDraggingSplitter.value = false;
}
</script>

<template>
  <div class="app-shell" @contextmenu.prevent>
    <AppToolbar
      :options-open="optionsOpen"
      :is-processing="isProcessing"
      :can-start="files.length > 0"
      :progress="progress"
      @refresh="scanAllFiles"
      @select-input="selectInputDir"
      @select-output="selectOutputDir"
      @toggle-options="optionsOpen = !optionsOpen"
      @open-crop-preview="openCropPreview"
      @open-video-merge="openVideoMerge"
      @start-process="startProcess"
    />

    <OptionsStrip
      :open="optionsOpen"
      :options="options"
      :enable-ratio-crop="enableRatioCrop"
      :ratios="ratios"
      :new-ratio="newRatio"
      :ratio-error="ratioError"
      :video-files-length="videoFiles.length"
      @open-crop-preview="openCropPreview"
      @update:enable-ratio-crop="enableRatioCrop = $event"
      @update:new-ratio="newRatio = $event"
      @add-ratio="addRatio"
      @remove-ratio="removeRatio"
    />

    <div
      ref="panesRef"
      class="panes"
      :style="{ gridTemplateColumns: `${leftPanePct}% 6px 1fr` }"
    >
      <FilePane
        kind="input"
        :dir="inputDir"
        :files="files"
        :selected-path="selectedInputPath"
        @select-dir="selectInputDir"
        @open-folder="openFolder(inputDir)"
        @select-file="selectedInputPath = $event.path"
      />

      <div
        class="splitter"
        :class="{ dragging: isDraggingSplitter }"
        title="拖动调整宽度"
        @mousedown="onSplitterDown"
      ></div>

      <FilePane
        kind="output"
        :dir="outputDir"
        :files="outputFiles"
        :selected-path="selectedOutputPath"
        @select-dir="selectOutputDir"
        @open-folder="openFolder(outputDir)"
        @select-file="selectedOutputPath = $event.path"
      />
    </div>

    <StatusBar
      :input-count="files.length"
      :image-count="imageCount"
      :video-count="videoCount"
      :output-count="outputFiles.length"
      :message="statusMessage"
      :upload-url="uploadUrl"
    />

    <CropPreviewModal
      :visible="cropPreviewVisible"
      :options="options"
      :crop-frame-image="cropFrameImage"
      :crop-video-width="cropVideoWidth"
      :crop-video-height="cropVideoHeight"
      :preview-scale="previewScale"
      @close="closeCropPreview"
      @confirm="confirmCrop"
      @start-drag="startDrag"
      @start-resize="startResize"
      @mouse-move="handleMouseMove"
      @mouse-up="handleMouseUp"
    />

    <VideoMergeModal
      :visible="videoMergeVisible"
      :output-dir="outputDir"
      @close="videoMergeVisible = false"
      @completed="handleVideoMergeCompleted"
    />
  </div>
</template>

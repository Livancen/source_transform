<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";

import type { FileInfo, ProcessProgress, ProcessOptions } from "./types";
import { useCrop } from "./composables/useCrop";

import DirectorySettings from "./components/DirectorySettings.vue";
import FileStatistics from "./components/FileStatistics.vue";
import ProcessingOptions from "./components/ProcessingOptions.vue";
import ProcessingActions from "./components/ProcessingActions.vue";
import CropPreviewModal from "./components/CropPreviewModal.vue";

// 目录状态
const inputDir = ref("");
const outputDir = ref("");

// 文件列表
const files = ref<FileInfo[]>([]);

// 处理选项
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

// 处理状态
const isProcessing = ref(false);
const progress = ref<ProcessProgress | null>(null);
const resultMessage = ref("");

// 计算属性
const imageCount = computed(
  () => files.value.filter((f) => f.file_type === "image").length
);
const videoCount = computed(
  () => files.value.filter((f) => f.file_type === "video").length
);
const videoFiles = computed(() =>
  files.value.filter((f) => f.file_type === "video")
);

// 裁剪逻辑
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

// 初始化
onMounted(async () => {
  try {
    const dirs = await invoke<[string, string]>("get_custom_dirs");
    inputDir.value = dirs[0];
    outputDir.value = dirs[1];
    await scanFiles();
  } catch (e) {
    console.error("初始化失败:", e);
  }

  await listen<ProcessProgress>("process-progress", (event) => {
    progress.value = event.payload;
  });
});

// 选择输入目录
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

// 选择输出目录
async function selectOutputDir() {
  const selected = await open({
    directory: true,
    title: "选择输出目录",
  });
  if (selected) {
    outputDir.value = selected as string;
    await saveCustomDirs();
  }
}

// 保存自定义目录
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

// 扫描文件
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

// 开始处理
async function startProcess() {
  if (files.value.length === 0) {
    resultMessage.value = "没有可处理的文件";
    return;
  }

  isProcessing.value = true;
  progress.value = null;
  resultMessage.value = "";

  try {
    const result = await invoke<string>("process_files", {
      inputDir: inputDir.value,
      outputDir: outputDir.value,
      options: options.value,
    });
    resultMessage.value = result;
  } catch (e) {
    resultMessage.value = `处理失败: ${e}`;
  } finally {
    isProcessing.value = false;
  }
}

// 打开文件夹
async function openFolder(path: string) {
  try {
    await invoke("open_folder", { path });
  } catch (e) {
    console.error("打开文件夹失败:", e);
  }
}
</script>

<template>
  <main class="container" @contextmenu.prevent>
    <div class="top-row">
      <DirectorySettings
        :input-dir="inputDir"
        :output-dir="outputDir"
        @select-input="selectInputDir"
        @select-output="selectOutputDir"
        @open-folder="openFolder"
      />
      <FileStatistics
        :image-count="imageCount"
        :video-count="videoCount"
        :total-count="files.length"
        :is-processing="isProcessing"
        @refresh="scanFiles"
      />
    </div>

    <ProcessingOptions
      :options="options"
      :video-files="videoFiles"
      @open-crop-preview="openCropPreview"
    />

    <ProcessingActions
      :is-processing="isProcessing"
      :progress="progress"
      :result-message="resultMessage"
      :files-length="files.length"
      @start-process="startProcess"
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
  </main>
</template>

<style>
* {
  margin: 0;
  padding: 0;
}
:root {
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  font-size: 14px;
  line-height: 1.5;
  font-weight: 400;
  color: #0f0f0f;
  background-color: #f6f6f6;
  user-select: none;
  -webkit-user-select: none;
}

.container {
  max-width: 900px;
  margin: 0 auto;
  padding: 10px;
}

.top-row {
  display: flex;
  gap: 15px;
}

.dir-section {
  flex: 1;
  margin-bottom: 0;
}

.stats-section {
  width: 200px;
  flex-shrink: 0;
  margin-bottom: 0;
  display: flex;
  flex-direction: column;
}

.stats {
  display: flex;
  flex-direction: column;
  gap: 8px;
  flex: 1;
}

.stat-item {
  display: flex;
  justify-content: space-between;
  padding: 4px 8px;
  background: #e9ecef;
  border-radius: 4px;
}

.stat-row-last {
  display: flex;
  gap: 8px;
}

.stat-row-last .stat-item {
  flex: 1;
}

.refresh-btn {
  flex: 1;
  padding: 4px 8px;
  font-size: 12px;
}

.stat-label {
  color: #666;
}

.stat-value {
  font-weight: 600;
}

h2 {
  font-size: 16px;
  margin-bottom: 10px;
  color: #555;
  border-bottom: 1px solid #ddd;
  padding-bottom: 5px;
}

.section {
  background: #fff;
  border-radius: 8px;
  padding: 15px;
  margin-bottom: 15px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}

.dir-row {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 10px;
}

.dir-row label {
  width: 40px;
  flex-shrink: 0;
}

.dir-row input {
  flex: 1;
  padding: 8px;
  border: 1px solid #ddd;
  border-radius: 4px;
  background: #f9f9f9;
}

.options-grid {
  display: flex;
  gap: 15px;
}

.options-column {
  flex: 1;
}

.option-group {
  margin-bottom: 10px;
  padding: 10px;
  background: #f9f9f9;
  border-radius: 4px;
}

.option-group:last-child {
  margin-bottom: 0;
}

.checkbox-label {
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  font-weight: 500;
}

.checkbox-label.sub-option {
  font-weight: 400;
  font-size: 13px;
}

.compress-mode {
  margin-top: 10px;
  padding-left: 26px;
}

.radio-label {
  display: flex;
  align-items: center;
  gap: 6px;
  cursor: pointer;
  font-size: 13px;
  margin-bottom: 6px;
}

.radio-label input[type="radio"] {
  width: 14px;
  height: 14px;
}

.compress-size {
  margin-top: 8px;
  padding-left: 20px;
}

.checkbox-label input[type="checkbox"] {
  width: 18px;
  height: 18px;
}

.option-detail {
  margin-top: 10px;
  padding-left: 26px;
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
}

.option-detail label {
  color: #666;
}

.option-detail input[type="number"] {
  width: 100px;
  padding: 5px;
  border: 1px solid #ddd;
  border-radius: 4px;
}

.option-detail input[type="range"] {
  width: 150px;
}

.option-detail select {
  padding: 5px 10px;
  border: 1px solid #ddd;
  border-radius: 4px;
}

button {
  padding: 8px 16px;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  background: #6c757d;
  color: white;
  transition: background 0.2s;
}

button:hover:not(:disabled) {
  background: #5a6268;
}

button:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.primary-btn {
  width: 100%;
  padding: 12px;
  font-size: 16px;
  background: #007bff;
}

.primary-btn:hover:not(:disabled) {
  background: #0056b3;
}

.processing-status {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
}

.processing-text {
  font-size: 14px;
  color: #333;
}

.processing-file {
  font-size: 12px;
  color: #666;
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.result-message {
  background: #f8f9fa;
  padding: 10px;
  border-radius: 4px;
  white-space: pre-wrap;
  font-family: monospace;
  font-size: 12px;
  max-height: 200px;
  overflow-y: auto;
}

.crop-info {
  flex-direction: column;
  align-items: flex-start !important;
  gap: 5px !important;
}

.crop-info span {
  font-size: 12px;
  color: #666;
}

.crop-btn {
  margin-top: 5px;
  padding: 4px 12px !important;
  font-size: 12px;
}

@media (prefers-color-scheme: dark) {
  :root {
    color: #f6f6f6;
    background-color: #1a1a1a;
  }

  .section {
    background: #2d2d2d;
  }

  h2 {
    color: #ccc;
    border-bottom-color: #444;
  }

  .dir-row input {
    background: #333;
    border-color: #444;
    color: #f6f6f6;
  }

  .stat-item {
    background: #444;
  }

  .stat-label {
    color: #aaa;
  }

  .option-group {
    background: #333;
  }

  .option-detail label {
    color: #aaa;
  }

  .option-detail input,
  .option-detail select {
    background: #444;
    border-color: #555;
    color: #f6f6f6;
  }

  .result-message {
    background: #333;
    color: #f6f6f6;
  }

  .crop-info span {
    color: #aaa;
  }
}
</style>

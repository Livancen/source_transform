<script setup lang="ts">
import { ref, onMounted, computed, watch } from "vue";
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

// 比例裁剪
const RATIO_STORAGE_KEY = "aspect-ratio-crop-ratios";
const enableRatioCrop = ref(false);
const ratios = ref<string[]>([]);
const newRatio = ref("");
const ratioError = ref("");

// 计算属性
const imageCount = computed(
  () => files.value.filter((f) => f.file_type === "image").length,
);
const videoCount = computed(
  () => files.value.filter((f) => f.file_type === "video").length,
);
const videoFiles = computed(() =>
  files.value.filter((f) => f.file_type === "video"),
);

// 比例管理
function addRatio() {
  ratioError.value = "";
  const val = newRatio.value.trim();
  if (!val) { ratioError.value = "请输入比例"; return; }
  const parts = val.split(":");
  if (parts.length !== 2 || !Number(parts[0]) || !Number(parts[1])) {
    ratioError.value = "格式错误，应为 W:H 如 1:1"; return;
  }
  if (ratios.value.includes(val)) { ratioError.value = "该比例已存在"; return; }
  ratios.value.push(val);
  newRatio.value = "";
}

function removeRatio(i: number) {
  ratios.value.splice(i, 1);
}

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
  const saved = localStorage.getItem(RATIO_STORAGE_KEY);
  if (saved) { try { ratios.value = JSON.parse(saved); } catch { /* ignore */ } }

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
  await listen<ProcessProgress>("crop-progress", (event) => {
    progress.value = event.payload;
  });
});

// 保存比例到 localStorage
watch(ratios, (val) => {
  localStorage.setItem(RATIO_STORAGE_KEY, JSON.stringify(val));
}, { deep: true });

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

  const messages: string[] = [];

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

  // 如果启用了比例裁剪且配置了比例，自动执行比例裁剪
  if (enableRatioCrop.value && ratios.value.length > 0) {
    try {
      const cropResult = await invoke<string>("crop_videos_by_ratios", {
        inputDir: inputDir.value,
        outputDir: outputDir.value,
        ratios: ratios.value,
      });
      messages.push(`\n比例裁剪:\n${cropResult}`);
    } catch (e) {
      messages.push(`\n比例裁剪失败: ${e}`);
    }
  }

  resultMessage.value = messages.join("");
  isProcessing.value = false;
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
  <main
    class="w-full h-full overflow-auto box-border p-12px font-sans text-14px bg-[#ddd]"
    @contextmenu.prevent
  >
    <div class="flex gap-15px">
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
        :files-length="files.length"
        @refresh="scanFiles"
        @start-process="startProcess"
      />
      <ProcessingActions
        :is-processing="isProcessing"
        :progress="progress"
        :result-message="resultMessage"
      />
    </div>
    <div class="h-15px"></div>
    <ProcessingOptions
      :options="options"
      :video-files="videoFiles"
      :enable-ratio-crop="enableRatioCrop"
      :ratios="ratios"
      :new-ratio="newRatio"
      :ratio-error="ratioError"
      @open-crop-preview="openCropPreview"
      @update:enable-ratio-crop="enableRatioCrop = $event"
      @update:new-ratio="newRatio = $event"
      @add-ratio="addRatio"
      @remove-ratio="removeRatio"
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
  padding: 0;
  margin: 0;
}
html,
body,
#app {
  width: 100%;
  height: 100%;
  user-select: none;
}
button {
  padding: 8px 16px;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  background: #20b42c;
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
</style>

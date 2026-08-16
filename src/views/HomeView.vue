<script setup lang="ts">
import { computed, watch } from "vue";
import { useRouter } from "vue-router";
import { useWorkspace } from "../composables/useWorkspace";
import { useCrop } from "../composables/useCrop";
import { useSplitter } from "../composables/useSplitter";

import AppToolbar from "../components/AppToolbar.vue";
import OptionsStrip from "../components/OptionsStrip.vue";
import FilePane from "../components/FilePane.vue";
import StatusBar from "../components/StatusBar.vue";
import CropWorkspace from "../components/CropWorkspace.vue";
import MergeWorkspace from "../components/MergeWorkspace.vue";
import type { FileInfo } from "../types";

const router = useRouter();

const {
  inputDir,
  outputDir,
  files,
  allInputFiles,
  outputFiles,
  selectedInputPath,
  selectedOutputPath,
  workMode,
  options,
  naming,
  isProcessing,
  progress,
  resultMessage,
  uploadUrl,
  optionsOpen,
  ratios,
  newRatio,
  ratioError,
  imageCount,
  videoCount,
  statusMessage,
  primaryActionLabel,
  canStart,
  addRatio,
  removeRatio,
  scanAllFiles,
  scanOutputFiles,
  selectInputDir,
  selectOutputDir,
  openFolder,
  startProcess,
} = useWorkspace();

const { leftPanePct, isDragging, onSplitterDown } = useSplitter();

const {
  selectedFile,
  cropFrameImage,
  mediaWidth,
  mediaHeight,
  previewScale,
  cropX,
  cropY,
  cropWidth,
  cropHeight,
  isExporting,
  isLoading,
  loadFile,
  clearFile,
  startDrag,
  startResize,
  handleMouseMove,
  handleMouseUp,
  exportCrop,
} = useCrop((msg) => {
  resultMessage.value = msg;
});

const showStart = computed(
  () =>
    workMode.value === "image" ||
    workMode.value === "video" ||
    workMode.value === "ratio",
);

const listFiles = computed(() => {
  if (workMode.value === "crop" || workMode.value === "merge") {
    return allInputFiles.value;
  }
  return files.value;
});

watch(workMode, () => {
  clearFile();
});

function goSettings() {
  router.push({ name: "settings" });
}

async function onCropSelect(file: FileInfo) {
  selectedInputPath.value = file.path;
  await loadFile(file);
}

async function onExportCrop() {
  await exportCrop(outputDir.value, naming.value);
  await scanOutputFiles();
}

function handleMergeCompleted(message: string) {
  resultMessage.value = message;
  scanOutputFiles();
}
</script>

<template>
  <div class="app-shell" @contextmenu.prevent>
    <AppToolbar
      :work-mode="workMode"
      :is-processing="isProcessing"
      :can-start="canStart"
      :progress="progress"
      :primary-label="primaryActionLabel"
      :show-start="showStart"
      @refresh="scanAllFiles"
      @update:work-mode="workMode = $event"
      @start-process="startProcess"
      @open-settings="goSettings"
    />

    <OptionsStrip
      :open="optionsOpen || workMode === 'ratio'"
      :work-mode="workMode"
      :options="options"
      :ratios="ratios"
      :new-ratio="newRatio"
      :ratio-error="ratioError"
      @update:new-ratio="newRatio = $event"
      @add-ratio="addRatio"
      @remove-ratio="removeRatio"
    />

    <CropWorkspace
      class="flex-1"
      v-if="workMode === 'crop'"
      :files="allInputFiles"
      :selected-path="selectedFile?.path || ''"
      :crop-frame-image="cropFrameImage"
      :media-width="mediaWidth"
      :media-height="mediaHeight"
      :preview-scale="previewScale"
      :crop-x="cropX"
      :crop-y="cropY"
      :crop-width="cropWidth"
      :crop-height="cropHeight"
      :is-loading="isLoading"
      :is-exporting="isExporting"
      :has-selection="!!selectedFile"
      @select-file="onCropSelect"
      @clear-file="clearFile"
      @export-crop="onExportCrop"
      @start-drag="startDrag"
      @start-resize="startResize"
      @mouse-move="handleMouseMove"
      @mouse-up="handleMouseUp"
      @update:crop-x="cropX = $event"
      @update:crop-y="cropY = $event"
      @update:crop-width="cropWidth = $event"
      @update:crop-height="cropHeight = $event"
    />

    <MergeWorkspace
      class="flex-1"
      v-if="workMode === 'merge'"
      :output-dir="outputDir"
      :input-files="allInputFiles"
      :naming="naming"
      @completed="handleMergeCompleted"
    />

    <div
      v-if="workMode == 'image' || workMode == 'video' || workMode == 'ratio'"
      ref="panesRef"
      class="flex-1 min-h-0 grid bg-bg0"
      :style="{ gridTemplateColumns: `${leftPanePct}% 6px 1fr` }"
    >
      <FilePane
        kind="input"
        :dir="inputDir"
        :files="listFiles"
        :selected-path="selectedInputPath"
        @select-dir="selectInputDir"
        @open-folder="openFolder(inputDir)"
        @select-file="selectedInputPath = $event.path"
      />

      <div
        class="splitter-bar relative z-5 bg-bg0 border-l border-r border-border cursor-col-resize transition-colors duration-150 hover:bg-secondary"
        :class="{ 'is-dragging! bg-secondary': isDragging }"
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
      :input-count="listFiles.length"
      :image-count="imageCount"
      :video-count="videoCount"
      :output-count="outputFiles.length"
      :message="statusMessage"
      :upload-url="uploadUrl"
    />
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { useRouter } from "vue-router";
import { useWorkspace } from "../composables/useWorkspace";
import { useCrop } from "../composables/useCrop";
import { useSplitter } from "../composables/useSplitter";

import AppToolbar from "../components/AppToolbar.vue";
import OptionsStrip from "../components/OptionsStrip.vue";
import FilePane from "../components/FilePane.vue";
import StatusBar from "../components/StatusBar.vue";
import CropPreviewModal from "../components/CropPreviewModal.vue";
import VideoMergeModal from "../components/VideoMergeModal.vue";

const router = useRouter();
const videoMergeVisible = ref(false);

const {
  inputDir,
  outputDir,
  files,
  outputFiles,
  selectedInputPath,
  selectedOutputPath,
  options,
  isProcessing,
  progress,
  resultMessage,
  uploadUrl,
  optionsOpen,
  enableRatioCrop,
  ratios,
  newRatio,
  ratioError,
  imageCount,
  videoCount,
  videoFiles,
  statusMessage,
  addRatio,
  removeRatio,
  scanAllFiles,
  scanOutputFiles,
  selectInputDir,
  selectOutputDir,
  openFolder,
  startProcess,
} = useWorkspace();

const { leftPanePct, isDragging, panesRef, onSplitterDown } = useSplitter();

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

function goSettings() {
  router.push({ name: "settings" });
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
      @open-settings="goSettings"
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
      class="flex-1 min-h-0 grid bg-bg0"
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
        class="splitter-bar relative z-5 bg-bg0 border-l border-r border-border cursor-col-resize transition-colors duration-150 hover:bg-accent"
        :class="{ 'is-dragging! bg-accent': isDragging }"
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

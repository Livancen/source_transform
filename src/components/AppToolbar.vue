<script setup lang="ts">
import type { ProcessProgress } from "../types";

const props = defineProps<{
  optionsOpen: boolean;
  isProcessing: boolean;
  canStart: boolean;
  progress: ProcessProgress | null;
}>();

defineEmits<{
  refresh: [];
  selectInput: [];
  selectOutput: [];
  toggleOptions: [];
  openCropPreview: [];
  openVideoMerge: [];
  startProcess: [];
}>();

const progressPercent = () => {
  if (!props.progress || props.progress.total <= 0) return 0;
  return Math.min(
    100,
    Math.round((props.progress.current / props.progress.total) * 100),
  );
};
</script>

<template>
  <header class="toolbar">
    <div class="brand">
      <div class="brand-icon">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M14.5 4h-5L7 7H4a2 2 0 0 0-2 2v9a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2V9a2 2 0 0 0-2-2h-3l-2.5-3z"/>
          <circle cx="12" cy="13" r="3"/>
        </svg>
      </div>
      <div class="brand-text">
        <strong>素材转换</strong>
        <span>v5.2.0</span>
      </div>
    </div>

    <div class="tb-group">
      <button class="tb-btn" title="刷新文件列表" :disabled="isProcessing" @click="$emit('refresh')">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="23 4 23 10 17 10"/><path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"/></svg>
        刷新
      </button>
      <button class="tb-btn" title="选择输入目录" :disabled="isProcessing" @click="$emit('selectInput')">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>
        输入目录
      </button>
      <button class="tb-btn" title="选择输出目录" :disabled="isProcessing" @click="$emit('selectOutput')">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/><path d="M12 11v6M9 14h6"/></svg>
        输出目录
      </button>
    </div>

    <div class="tb-sep"></div>

    <div class="tb-group">
      <button
        class="tb-btn"
        :class="{ active: optionsOpen }"
        title="处理选项"
        @click="$emit('toggleOptions')"
      >
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/></svg>
        处理选项
      </button>
      <button class="tb-btn" title="裁剪预览" :disabled="isProcessing" @click="$emit('openCropPreview')">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M6 2v14a2 2 0 0 0 2 2h14"/><path d="M18 22V8a2 2 0 0 0-2-2H2"/></svg>
        裁剪预览
      </button>
      <button class="tb-btn" title="视频拼接" :disabled="isProcessing" @click="$emit('openVideoMerge')">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="2" y="7" width="8" height="10" rx="1"/><rect x="14" y="7" width="8" height="10" rx="1"/><path d="M10 12h4"/></svg>
        视频拼接
      </button>
    </div>

    <div class="tb-sep"></div>

    <button
      class="tb-btn success"
      :disabled="isProcessing || !canStart"
      @click="$emit('startProcess')"
    >
      <svg v-if="!isProcessing" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><polygon points="5 3 19 12 5 21 5 3"/></svg>
      <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg>
      {{ isProcessing ? "处理中…" : "开始处理" }}
    </button>

    <div class="tb-spacer"></div>

    <div v-if="isProcessing && progress" class="tb-progress">
      <div class="bar"><i :style="{ width: progressPercent() + '%' }"></i></div>
      <span class="pct">{{ progressPercent() }}%</span>
    </div>

    <div class="tb-status">
      <span class="dot"></span>
      FFmpeg 就绪
    </div>
  </header>
</template>

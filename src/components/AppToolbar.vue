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
  openSettings: [];
}>();

function progressPercent() {
  if (!props.progress || props.progress.total <= 0) return 0;
  return Math.min(
    100,
    Math.round((props.progress.current / props.progress.total) * 100),
  );
}
</script>

<template>
  <header
    class="shrink-0 h-52px px-12px flex items-center gap-6px bg-gradient-to-b from-#1a2030 to-bg1 border-b border-border z-20"
  >
    <div class="flex items-center gap-10px pr-12px mr-4px border-r border-border shrink-0">
      <div
        class="w-30px h-30px rounded-8px bg-gradient-to-br from-#5b8cff to-#8b5cf6 grid place-items-center shadow-[0_2px_10px_rgba(91,140,255,0.35)] color-white"
      >
        <svg class="w-16px h-16px" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M14.5 4h-5L7 7H4a2 2 0 0 0-2 2v9a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2V9a2 2 0 0 0-2-2h-3l-2.5-3z"/>
          <circle cx="12" cy="13" r="3"/>
        </svg>
      </div>
      <div class="flex flex-col leading-tight">
        <strong class="text-13px font-700 tracking-tight">素材转换</strong>
        <span class="text-10px color-t3">v5.2.0</span>
      </div>
    </div>

    <div class="flex items-center gap-4px shrink-0">
      <button class="tb-btn" title="刷新文件列表" :disabled="isProcessing" @click="$emit('refresh')">
        <svg class="w-15px h-15px shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="23 4 23 10 17 10"/><path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"/></svg>
        刷新
      </button>
      <button class="tb-btn" title="选择输入目录" :disabled="isProcessing" @click="$emit('selectInput')">
        <svg class="w-15px h-15px shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>
        输入目录
      </button>
      <button class="tb-btn" title="选择输出目录" :disabled="isProcessing" @click="$emit('selectOutput')">
        <svg class="w-15px h-15px shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/><path d="M12 11v6M9 14h6"/></svg>
        输出目录
      </button>
    </div>

    <div class="w-1px h-28px bg-border mx-4px shrink-0"></div>

    <div class="flex items-center gap-4px shrink-0">
      <button
        class="tb-btn"
        :class="{ 'tb-btn-active': optionsOpen }"
        title="处理选项"
        @click="$emit('toggleOptions')"
      >
        <svg class="w-15px h-15px shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/></svg>
        处理选项
      </button>
      <button class="tb-btn" title="裁剪预览" :disabled="isProcessing" @click="$emit('openCropPreview')">
        <svg class="w-15px h-15px shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M6 2v14a2 2 0 0 0 2 2h14"/><path d="M18 22V8a2 2 0 0 0-2-2H2"/></svg>
        裁剪预览
      </button>
      <button class="tb-btn" title="视频拼接" :disabled="isProcessing" @click="$emit('openVideoMerge')">
        <svg class="w-15px h-15px shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="2" y="7" width="8" height="10" rx="1"/><rect x="14" y="7" width="8" height="10" rx="1"/><path d="M10 12h4"/></svg>
        视频拼接
      </button>
    </div>

    <div class="w-1px h-28px bg-border mx-4px shrink-0"></div>

    <button
      class="tb-btn tb-btn-success"
      :disabled="isProcessing || !canStart"
      @click="$emit('startProcess')"
    >
      <svg v-if="!isProcessing" class="w-15px h-15px shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><polygon points="5 3 19 12 5 21 5 3"/></svg>
      <svg v-else class="w-15px h-15px shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg>
      {{ isProcessing ? "处理中…" : "开始处理" }}
    </button>

    <div class="flex-1 min-w-8px"></div>

    <div
      v-if="isProcessing && progress"
      class="flex items-center gap-8px min-w-160px px-10px h-34px rounded-6px bg-bg0 border border-border"
    >
      <div class="flex-1 h-4px rounded-full bg-bg3 overflow-hidden">
        <i
          class="block h-full rounded-full bg-gradient-to-r from-#5b8cff to-#8b5cf6 transition-all duration-300"
          :style="{ width: progressPercent() + '%' }"
        ></i>
      </div>
      <span class="font-mono text-11px color-t2 min-w-36px text-right">{{ progressPercent() }}%</span>
    </div>

    <button class="tb-btn" title="设置" @click="$emit('openSettings')">
      <svg class="w-15px h-15px shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="3"/>
        <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/>
      </svg>
      设置
    </button>

    <div class="flex items-center gap-8px text-11px color-t3 shrink-0">
      <span class="w-6px h-6px rounded-full bg-success shadow-[0_0_8px_#34d399]"></span>
      FFmpeg 就绪
    </div>
  </header>
</template>

<script setup lang="ts">
import type { ProcessProgress, WorkMode } from "../types";

const props = defineProps<{
  workMode: WorkMode;
  isProcessing: boolean;
  canStart: boolean;
  progress: ProcessProgress | null;
  primaryLabel: string;
  showStart: boolean;
}>();

defineEmits<{
  refresh: [];
  "update:workMode": [mode: WorkMode];
  startProcess: [];
  openSettings: [];
}>();

const modes: { id: WorkMode; label: string }[] = [
  { id: "image", label: "图片" },
  { id: "video", label: "视频" },
  { id: "ratio", label: "比例裁剪" },
  { id: "crop", label: "自定义裁剪" },
  { id: "merge", label: "拼接" },
];

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
    class="shrink-0 h-52px px-12px flex items-center gap-6px bg-bg1 border-b border-border z-20"
  >
    <div class="flex items-center gap-4px shrink-0">
      <button
        class="tb-btn"
        title="刷新文件列表"
        :disabled="isProcessing"
        @click="$emit('refresh')"
      >
        <svg
          class="w-15px h-15px shrink-0"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
        >
          <polyline points="23 4 23 10 17 10" />
          <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10" />
        </svg>
      </button>
    </div>

    <div class="w-1px h-28px bg-border mx-4px shrink-0"></div>

    <div class="flex items-center gap-2px shrink-0 p-2px rounded-8px bg-bg0 border border-border">
      <button
        v-for="m in modes"
        :key="m.id"
        type="button"
        class="h-30px px-12px rounded-6px text-12px font-500 border-none cursor-pointer transition-all duration-150"
        :class="
          workMode === m.id
            ? 'bg-secondary color-white'
            : 'bg-transparent color-t2 hover:color-t1 hover:bg-bg2'
        "
        :disabled="isProcessing"
        @click="$emit('update:workMode', m.id)"
      >
        {{ m.label }}
      </button>
    </div>

    <div class="w-1px h-28px bg-border mx-4px shrink-0"></div>

    <button
      v-if="showStart"
      class="tb-btn tb-btn-success"
      :disabled="isProcessing || !canStart"
      @click="$emit('startProcess')"
    >
      <svg
        v-if="!isProcessing"
        class="w-15px h-15px shrink-0"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2.5"
      >
        <polygon points="5 3 19 12 5 21 5 3" />
      </svg>
      <svg
        v-else
        class="w-15px h-15px shrink-0"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
      >
        <circle cx="12" cy="12" r="10" />
        <polyline points="12 6 12 12 16 14" />
      </svg>
      {{ primaryLabel }}
    </button>

    <div class="flex-1 min-w-8px"></div>

    <div
      v-if="isProcessing && progress"
      class="flex items-center gap-8px min-w-160px px-10px h-34px rounded-6px bg-bg0 border border-border"
    >
      <div class="flex-1 h-4px rounded-full bg-bg3 overflow-hidden">
        <i
          class="block h-full rounded-full bg-primary transition-all duration-300"
          :style="{ width: progressPercent() + '%' }"
        ></i>
      </div>
      <span class="font-mono text-11px color-t2 min-w-36px text-right"
        >{{ progressPercent() }}%</span
      >
    </div>

    <button class="tb-btn" title="设置" @click="$emit('openSettings')">
      <svg
        class="w-15px h-15px shrink-0"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
      >
        <circle cx="12" cy="12" r="3" />
        <path
          d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"
        />
      </svg>
      设置
    </button>

    <div class="flex items-center gap-8px text-11px color-t3 shrink-0">
      <span class="w-6px h-6px rounded-full bg-primary"></span>
      FFmpeg 就绪
    </div>
  </header>
</template>

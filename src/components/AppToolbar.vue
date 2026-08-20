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
  { id: "join", label: "自定义拼接" },
];

function progressPercent() {
  if (!props.progress || props.progress.total <= 0) return 0;
  if (typeof props.progress.percent === "number" && Number.isFinite(props.progress.percent)) {
    return Math.min(100, Math.max(0, Math.round(props.progress.percent)));
  }
  // 兼容旧事件：按「已完成文件」估算，当前文件计为进行中
  const done = Math.max(0, props.progress.current - 1);
  return Math.min(
    100,
    Math.round((done / props.progress.total) * 100),
  );
}
</script>

<template>
  <header
    class="shrink-0 h-48px px-12px flex items-center gap-10px bg-bg1 border-b border-border z-20"
  >
    <nav class="flex items-stretch gap-0 shrink-0 h-full" aria-label="工作模式">
      <button
        v-for="m in modes"
        :key="m.id"
        type="button"
        class="fluent-pivot-item"
        :class="{ 'is-active': workMode === m.id }"
        :disabled="isProcessing"
        @click="($emit('update:workMode', m.id), $emit('refresh'))"
      >
        {{ m.label }}
      </button>
    </nav>

    <button
      v-if="showStart"
      class="tb-btn tb-btn-success"
      :disabled="isProcessing || !canStart"
      @click="$emit('startProcess')"
    >
      <svg
        v-if="!isProcessing"
        class="w-14px h-14px shrink-0"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2.5"
      >
        <polygon points="5 3 19 12 5 21 5 3" />
      </svg>
      <svg
        v-else
        class="w-14px h-14px shrink-0"
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
      class="flex items-center gap-8px min-w-160px px-10px h-32px rounded-4px bg-bg0 border border-border"
    >
      <div class="flex-1 h-3px rounded-full bg-bg3 overflow-hidden">
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
        class="w-14px h-14px shrink-0"
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
  </header>
</template>

<script setup lang="ts">
import { computed } from "vue";
import type { FileInfo } from "../types";

const props = defineProps<{
  kind: "input" | "output";
  dir: string;
  files: FileInfo[];
  selectedPath?: string;
}>();

const emit = defineEmits<{
  selectDir: [];
  openFolder: [];
  selectFile: [file: FileInfo];
}>();

const title = computed(() => (props.kind === "input" ? "输入" : "输出"));

function fileExt(name: string) {
  const i = name.lastIndexOf(".");
  return i >= 0 ? name.slice(i + 1) : "";
}
</script>

<template>
  <section class="flex flex-col min-w-0 min-h-0 bg-bg1 relative">
    <div class="shrink-0 h-40px flex items-center gap-6px px-8px bg-bg2 border-b border-border">
      <span
        class="inline-flex items-center gap-5px h-24px px-8px rounded-5px text-11px font-600 shrink-0"
        :class="kind === 'input'
          ? 'bg-[rgba(56,189,248,0.12)] color-video'
          : 'bg-[rgba(52,211,153,0.12)] color-success'"
      >
        <svg class="w-12px h-12px" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
          <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
          <path v-if="kind === 'output'" d="M12 11v6M9 14h6"/>
        </svg>
        {{ title }}
      </span>
      <input
        class="flex-1 min-w-0 h-28px px-10px rounded-6px border border-border bg-bg0 color-t2 font-mono text-11px outline-none focus:border-accent focus:color-t1"
        readonly
        :value="dir"
        :title="dir"
      />
      <button
        class="w-28px h-28px grid place-items-center border border-transparent rounded-6px bg-transparent color-t3 cursor-pointer transition-all duration-150 hover:not-disabled:bg-bg3 hover:not-disabled:color-t1 hover:not-disabled:border-border disabled:opacity-45"
        title="选择目录"
        type="button"
        @click="emit('selectDir')"
      >
        <svg class="w-14px h-14px" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M3 7v10a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V9a2 2 0 0 0-2-2h-6l-2-2H5a2 2 0 0 0-2 2z"/>
        </svg>
      </button>
      <button
        class="w-28px h-28px grid place-items-center border border-transparent rounded-6px bg-transparent color-t3 cursor-pointer transition-all duration-150 hover:not-disabled:bg-bg3 hover:not-disabled:color-t1 hover:not-disabled:border-border disabled:opacity-45"
        title="在资源管理器中打开"
        type="button"
        :disabled="!dir"
        @click="emit('openFolder')"
      >
        <svg class="w-14px h-14px" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/>
          <polyline points="15 3 21 3 21 9"/>
          <line x1="10" y1="14" x2="21" y2="3"/>
        </svg>
      </button>
    </div>

    <div
      class="shrink-0 grid grid-cols-[minmax(0,1fr)_72px_88px] h-28px items-center px-12px bg-bg2 border-b border-border text-11px font-500 color-t3"
    >
      <span class="px-6px truncate">名称</span>
      <span class="px-6px truncate">类型</span>
      <span class="px-6px truncate">扩展名</span>
    </div>

    <div class="file-list-scroll flex-1 min-h-0 overflow-y-auto overflow-x-hidden py-2px">
      <template v-if="files.length > 0">
        <div
          v-for="file in files"
          :key="file.path"
          class="grid grid-cols-[minmax(0,1fr)_72px_88px] items-center h-32px px-12px mx-4px rounded-6px color-t1 transition-colors duration-100 cursor-default hover:bg-bg3"
          :class="{
            'bg-bg-selected outline outline-1 outline-[rgba(91,140,255,0.35)]': selectedPath === file.path,
          }"
          @click="emit('selectFile', file)"
        >
          <div class="flex items-center gap-8px min-w-0 px-6px">
            <span
              class="w-18px h-18px shrink-0 grid place-items-center"
              :class="file.file_type === 'image' ? 'color-image' : 'color-video'"
            >
              <svg v-if="file.file_type === 'image'" class="w-16px h-16px" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <rect x="3" y="3" width="18" height="18" rx="2"/>
                <circle cx="8.5" cy="8.5" r="1.5"/>
                <path d="M21 15l-5-5L5 21"/>
              </svg>
              <svg v-else class="w-16px h-16px" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <rect x="2" y="4" width="15" height="16" rx="2"/>
                <path d="M17 10l5-3v10l-5-3z"/>
              </svg>
            </span>
            <span class="truncate text-13px" :title="file.path">{{ file.name }}</span>
          </div>
          <div class="px-6px">
            <span
              class="inline-flex items-center h-18px px-6px rounded-4px text-10px font-600 tracking-wide"
              :class="file.file_type === 'image'
                ? 'bg-[rgba(167,139,250,0.15)] color-image'
                : 'bg-[rgba(56,189,248,0.15)] color-video'"
            >
              {{ file.file_type === "image" ? "图片" : "视频" }}
            </span>
          </div>
          <div class="px-6px font-mono text-11px color-t3 uppercase truncate">
            {{ fileExt(file.name) }}
          </div>
        </div>
      </template>
      <div
        v-else
        class="h-full flex flex-col items-center justify-center gap-10px color-t3 p-32px text-center"
      >
        <svg class="w-48px h-48px opacity-35" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
        </svg>
        <p class="text-13px">{{ kind === "input" ? "输入目录为空" : "输出目录为空" }}</p>
        <span class="text-11px opacity-80">
          {{ kind === "input" ? "放入图片或视频后点击刷新" : "处理完成后文件将显示在这里" }}
        </span>
      </div>
    </div>
  </section>
</template>

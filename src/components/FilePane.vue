<script setup lang="ts">
import { computed, toRef } from "vue";
import type { FileInfo } from "../types";
import { formatFileSizeMb } from "../types";
import { useFileThumbs } from "../composables/useFileThumbs";

const props = defineProps<{
  kind: "input" | "output";
  dir: string;
  files: FileInfo[];
  selectedPath?: string;
  /** 输入列表是否显示勾选 */
  selectable?: boolean;
  checkedPaths?: Set<string>;
  allChecked?: boolean;
}>();

const emit = defineEmits<{
  selectDir: [];
  openFolder: [];
  selectFile: [file: FileInfo];
  toggleCheck: [path: string];
  toggleCheckAll: [];
}>();

const title = computed(() => (props.kind === "input" ? "输入" : "输出"));
const filesRef = toRef(props, "files");
const { thumbs } = useFileThumbs(filesRef);

function fileExt(name: string) {
  const i = name.lastIndexOf(".");
  return i >= 0 ? name.slice(i + 1) : "";
}

function isRowChecked(path: string) {
  return props.checkedPaths?.has(path) ?? false;
}
</script>

<template>
  <section class="flex flex-col min-w-0 min-h-0 bg-bg1 relative">
    <div class="shrink-0 h-40px flex items-center gap-6px px-8px bg-bg2 border-b border-border">
      <span
        class="inline-flex items-center gap-5px h-24px px-8px rounded-5px text-11px font-600 shrink-0"
        :class="kind === 'input'
          ? 'bg-secondary-soft color-video'
          : 'bg-success-soft color-success'"
      >
        <svg class="w-12px h-12px" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
          <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
          <path v-if="kind === 'output'" d="M12 11v6M9 14h6"/>
        </svg>
        {{ title }}
      </span>
      <input
        class="flex-1 min-w-0 h-28px px-10px rounded-6px border border-border bg-bg0 color-t2 font-mono text-11px outline-none focus:border-secondary focus:bg-bg1 focus:color-t1"
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
      class="shrink-0 grid h-28px items-center px-10px bg-bg2 border-b border-border text-11px font-500 color-t3"
      :class="selectable
        ? 'grid-cols-[28px_48px_minmax(0,1fr)_56px_64px]'
        : 'grid-cols-[48px_minmax(0,1fr)_56px_64px]'"
    >
      <label v-if="selectable" class="flex items-center justify-center" @click.stop>
        <input
          type="checkbox"
          class="accent-secondary w-14px h-14px cursor-pointer"
          :checked="allChecked"
          :indeterminate="!allChecked && files.some(f => isRowChecked(f.path))"
          @change="emit('toggleCheckAll')"
        />
      </label>
      <span class="px-4px truncate">预览</span>
      <span class="px-6px truncate">名称</span>
      <span class="px-4px truncate">大小</span>
      <span class="px-4px truncate">类型</span>
    </div>

    <div class="file-list-scroll flex-1 min-h-0 overflow-y-auto overflow-x-hidden py-2px">
      <template v-if="files.length > 0">
        <div
          v-for="file in files"
          :key="file.path"
          class="grid items-center min-h-52px px-10px mx-4px my-1px rounded-6px color-t1 transition-colors duration-100 cursor-default hover:bg-bg3"
          :class="[
            selectable
              ? 'grid-cols-[28px_48px_minmax(0,1fr)_56px_64px]'
              : 'grid-cols-[48px_minmax(0,1fr)_56px_64px]',
            { 'bg-bg-selected outline outline-1 outline-secondary/25': selectedPath === file.path },
          ]"
          @click="emit('selectFile', file)"
        >
          <label
            v-if="selectable"
            class="flex items-center justify-center h-full"
            @click.stop
          >
            <input
              type="checkbox"
              class="accent-secondary w-14px h-14px cursor-pointer"
              :checked="isRowChecked(file.path)"
              @change="emit('toggleCheck', file.path)"
            />
          </label>

          <div class="w-40px h-40px rounded-4px overflow-hidden bg-bg0 border border-border shrink-0">
            <img
              v-if="thumbs[file.path]"
              :src="thumbs[file.path]"
              class="w-full h-full object-cover"
              draggable="false"
            />
            <div
              v-else
              class="w-full h-full grid place-items-center"
              :class="file.file_type === 'image' ? 'color-image' : 'color-video'"
            >
              <svg v-if="file.file_type === 'image'" class="w-16px h-16px opacity-50" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <rect x="3" y="3" width="18" height="18" rx="2"/>
                <circle cx="8.5" cy="8.5" r="1.5"/>
                <path d="M21 15l-5-5L5 21"/>
              </svg>
              <svg v-else class="w-16px h-16px opacity-50" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <rect x="2" y="4" width="15" height="16" rx="2"/>
                <path d="M17 10l5-3v10l-5-3z"/>
              </svg>
            </div>
          </div>

          <div class="min-w-0 px-6px">
            <div class="truncate text-13px" :title="file.path">{{ file.name }}</div>
            <div class="text-10px color-t3 font-mono uppercase truncate">{{ fileExt(file.name) }}</div>
          </div>

          <div class="px-4px text-11px color-t3 whitespace-nowrap">
            {{ formatFileSizeMb(file.size_bytes) }}
          </div>

          <div class="px-4px">
            <span
              class="inline-flex items-center h-18px px-6px rounded-4px text-10px font-600 tracking-wide"
              :class="file.file_type === 'image'
                ? 'bg-[rgba(114,46,209,0.08)] color-image'
                : 'bg-secondary-soft color-video'"
            >
              {{ file.file_type === "image" ? "图片" : "视频" }}
            </span>
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

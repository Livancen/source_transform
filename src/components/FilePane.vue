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
  <section class="pane">
    <div class="pathbar">
      <span class="pane-tag" :class="kind">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
          <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
          <path v-if="kind === 'output'" d="M12 11v6M9 14h6"/>
        </svg>
        {{ title }}
      </span>
      <input class="path-input" readonly :value="dir" :title="dir" />
      <button class="icon-btn" title="选择目录" type="button" @click="emit('selectDir')">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M3 7v10a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V9a2 2 0 0 0-2-2h-6l-2-2H5a2 2 0 0 0-2 2z"/>
        </svg>
      </button>
      <button class="icon-btn" title="在资源管理器中打开" type="button" :disabled="!dir" @click="emit('openFolder')">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/>
          <polyline points="15 3 21 3 21 9"/>
          <line x1="10" y1="14" x2="21" y2="3"/>
        </svg>
      </button>
    </div>

    <div class="file-header">
      <span>名称</span>
      <span>类型</span>
      <span>扩展名</span>
    </div>

    <div class="file-list">
      <template v-if="files.length > 0">
        <div
          v-for="file in files"
          :key="file.path"
          class="file-row"
          :class="{ selected: selectedPath === file.path }"
          @click="emit('selectFile', file)"
        >
          <div class="name">
            <span class="icon" :class="file.file_type">
              <svg v-if="file.file_type === 'image'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <rect x="3" y="3" width="18" height="18" rx="2"/>
                <circle cx="8.5" cy="8.5" r="1.5"/>
                <path d="M21 15l-5-5L5 21"/>
              </svg>
              <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <rect x="2" y="4" width="15" height="16" rx="2"/>
                <path d="M17 10l5-3v10l-5-3z"/>
              </svg>
            </span>
            <span class="text" :title="file.path">{{ file.name }}</span>
          </div>
          <div class="col">
            <span class="type-pill" :class="file.file_type">
              {{ file.file_type === "image" ? "图片" : "视频" }}
            </span>
          </div>
          <div class="col ext">{{ fileExt(file.name) }}</div>
        </div>
      </template>
      <div v-else class="empty-state">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
        </svg>
        <p>{{ kind === "input" ? "输入目录为空" : "输出目录为空" }}</p>
        <span>{{ kind === "input" ? "放入图片或视频后点击刷新" : "处理完成后文件将显示在这里" }}</span>
      </div>
    </div>
  </section>
</template>

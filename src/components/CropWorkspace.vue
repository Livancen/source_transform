<script setup lang="ts">
import { ref, watch, onMounted, onBeforeUnmount, nextTick, toRef } from "vue";
import type { FileInfo } from "../types";
import { formatFileSizeMb } from "../types";
import { useFileThumbs } from "../composables/useFileThumbs";

const props = defineProps<{
  files: FileInfo[];
  selectedPath: string;
  cropFrameImage: string;
  mediaWidth: number;
  mediaHeight: number;
  previewScale: number;
  cropX: number;
  cropY: number;
  cropWidth: number;
  cropHeight: number;
  isLoading: boolean;
  isExporting: boolean;
  hasSelection: boolean;
}>();

const emit = defineEmits<{
  selectFile: [file: FileInfo];
  clearFile: [];
  exportCrop: [];
  startDrag: [e: MouseEvent];
  startResize: [e: MouseEvent, handle: string];
  mouseMove: [e: MouseEvent];
  mouseUp: [];
  "update:cropX": [v: number];
  "update:cropY": [v: number];
  "update:cropWidth": [v: number];
  "update:cropHeight": [v: number];
  fitPreview: [width: number, height: number];
}>();

const filesRef = toRef(props, "files");
const { thumbs } = useFileThumbs(filesRef);

const previewHostRef = ref<HTMLElement | null>(null);
let resizeObserver: ResizeObserver | null = null;

function reportFit() {
  const el = previewHostRef.value;
  if (!el) return;
  const rect = el.getBoundingClientRect();
  emit("fitPreview", rect.width, rect.height);
}

function setupObserver() {
  resizeObserver?.disconnect();
  const el = previewHostRef.value;
  if (!el) return;
  resizeObserver = new ResizeObserver(() => reportFit());
  resizeObserver.observe(el);
  reportFit();
}

onMounted(() => {
  nextTick(setupObserver);
});

onBeforeUnmount(() => {
  resizeObserver?.disconnect();
  resizeObserver = null;
});

watch(
  () => props.cropFrameImage,
  async () => {
    await nextTick();
    setupObserver();
  },
);

watch(
  () => [props.mediaWidth, props.mediaHeight],
  () => {
    reportFit();
  },
);
</script>

<template>
  <div
    class="flex-1 min-h-0 flex flex-col bg-bg1 overflow-hidden"
    @mousemove="emit('mouseMove', $event)"
    @mouseup="emit('mouseUp')"
    @mouseleave="emit('mouseUp')"
  >
    <div class="shrink-0 px-14px py-10px flex items-center justify-between gap-12px border-b border-border">
      <div>
        <div class="text-13px font-600">自定义裁剪</div>
        <div class="text-11px color-t3 mt-2px">
          每次 1 个文件 · 像素坐标 · 预览自适应窗口
        </div>
      </div>
      <button
        class="tb-btn tb-btn-success"
        type="button"
        :disabled="!hasSelection || isExporting || isLoading"
        @click="emit('exportCrop')"
      >
        {{ isExporting ? "导出中…" : "导出裁剪" }}
      </button>
    </div>

    <div class="flex-1 min-h-0 flex gap-0 overflow-hidden">
      <!-- 文件列表：固定宽 + 内部滚动 -->
      <div
        class="w-280px shrink-0 min-h-0 border-r border-border flex flex-col bg-bg0 overflow-hidden"
      >
        <div
          class="shrink-0 h-32px px-10px flex items-center text-11px font-500 color-t3 border-b border-border bg-bg2"
        >
          选择文件 ({{ files.length }})
        </div>
        <div class="flex-1 min-h-0 overflow-y-auto overflow-x-hidden">
          <button
            v-for="f in files"
            :key="f.path"
            type="button"
            class="w-full text-left px-10px py-8px border-none border-b border-border/60 cursor-pointer text-12px flex gap-10px items-center"
            :class="
              selectedPath === f.path
                ? 'bg-secondary-soft color-secondary'
                : 'bg-transparent color-t2 hover:bg-bg2'
            "
            @click="emit('selectFile', f)"
          >
            <div class="w-44px h-44px rounded-4px overflow-hidden bg-bg1 border border-border shrink-0">
              <img
                v-if="thumbs[f.path]"
                :src="thumbs[f.path]"
                class="w-full h-full object-cover"
                draggable="false"
              />
              <div v-else class="w-full h-full grid place-items-center text-10px color-t3">…</div>
            </div>
            <div class="min-w-0 flex-1">
              <div class="truncate font-500" :title="f.name">{{ f.name }}</div>
              <div class="text-10px color-t3 mt-2px flex gap-8px">
                <span>{{ f.file_type === "video" ? "视频" : "图片" }}</span>
                <span>{{ formatFileSizeMb(f.size_bytes) }}</span>
              </div>
            </div>
          </button>
          <div
            v-if="files.length === 0"
            class="p-16px text-11px color-t3 text-center"
          >
            输入目录无媒体文件
          </div>
        </div>
      </div>

      <!-- 预览区 -->
      <div class="flex-1 min-w-0 min-h-0 flex flex-col overflow-hidden">
        <div
          ref="previewHostRef"
          class="flex-1 min-h-0 overflow-hidden flex items-center justify-center bg-#1a1a1a p-8px"
        >
          <div
            v-if="isLoading"
            class="text-12px color-t3"
          >
            加载预览中…
          </div>
          <div
            v-else-if="!cropFrameImage"
            class="text-12px color-t3 text-center px-20px"
          >
            请从左侧选择一个文件
          </div>
          <div
            v-else
            class="relative bg-black shrink-0"
            :style="{
              width: mediaWidth * previewScale + 'px',
              height: mediaHeight * previewScale + 'px',
            }"
          >
            <img
              :src="cropFrameImage"
              class="block select-none w-full h-full object-fill"
              draggable="false"
            />
            <div class="absolute inset-0 w-full h-full pointer-events-none">
              <div
                class="absolute top-0 left-0 w-full bg-black/50"
                :style="{ height: cropY * previewScale + 'px' }"
              ></div>
              <div
                class="absolute bottom-0 left-0 w-full bg-black/50"
                :style="{
                  height: (mediaHeight - cropY - cropHeight) * previewScale + 'px',
                }"
              ></div>
              <div
                class="absolute left-0 bg-black/50"
                :style="{
                  top: cropY * previewScale + 'px',
                  height: cropHeight * previewScale + 'px',
                  width: cropX * previewScale + 'px',
                }"
              ></div>
              <div
                class="absolute right-0 bg-black/50"
                :style="{
                  top: cropY * previewScale + 'px',
                  height: cropHeight * previewScale + 'px',
                  width: (mediaWidth - cropX - cropWidth) * previewScale + 'px',
                }"
              ></div>
            </div>
            <div
              class="crop-area-box absolute border-2 border-secondary cursor-move box-border"
              :style="{
                left: cropX * previewScale + 'px',
                top: cropY * previewScale + 'px',
                width: cropWidth * previewScale + 'px',
                height: cropHeight * previewScale + 'px',
              }"
              @mousedown="emit('startDrag', $event)"
            >
              <div
                v-for="h in ['n', 's', 'e', 'w', 'ne', 'nw', 'se', 'sw']"
                :key="h"
                class="absolute w-10px h-10px bg-secondary border border-white"
                :class="{
                  'cursor-n-resize -top-5px left-50% -translate-x-50%': h === 'n',
                  'cursor-s-resize -bottom-5px left-50% -translate-x-50%': h === 's',
                  'cursor-e-resize -right-5px top-50% -translate-y-50%': h === 'e',
                  'cursor-w-resize -left-5px top-50% -translate-y-50%': h === 'w',
                  'cursor-ne-resize -top-5px -right-5px': h === 'ne',
                  'cursor-nw-resize -top-5px -left-5px': h === 'nw',
                  'cursor-se-resize -bottom-5px -right-5px': h === 'se',
                  'cursor-sw-resize -bottom-5px -left-5px': h === 'sw',
                }"
                @mousedown.stop="emit('startResize', $event, h)"
              ></div>
              <div
                class="absolute bottom-5px left-50% -translate-x-50% bg-black/70 color-white px-8px py-2px rounded-3px text-12px whitespace-nowrap pointer-events-none"
              >
                {{ cropWidth }} × {{ cropHeight }}
              </div>
            </div>
          </div>
        </div>

        <!-- 参数栏 -->
        <div
          v-if="cropFrameImage && !isLoading"
          class="shrink-0 flex gap-12px flex-wrap items-center px-12px py-10px border-t border-border bg-bg1"
        >
          <div class="flex items-center gap-6px">
            <label class="text-12px color-t3">宽</label>
            <input
              class="field w-80px!"
              type="number"
              :value="cropWidth"
              min="1"
              :max="mediaWidth"
              @input="emit('update:cropWidth', Number(($event.target as HTMLInputElement).value))"
            />
          </div>
          <div class="flex items-center gap-6px">
            <label class="text-12px color-t3">高</label>
            <input
              class="field w-80px!"
              type="number"
              :value="cropHeight"
              min="1"
              :max="mediaHeight"
              @input="emit('update:cropHeight', Number(($event.target as HTMLInputElement).value))"
            />
          </div>
          <div class="flex items-center gap-6px">
            <label class="text-12px color-t3">X</label>
            <input
              class="field w-80px!"
              type="number"
              :value="cropX"
              min="0"
              :max="Math.max(0, mediaWidth - cropWidth)"
              @input="emit('update:cropX', Number(($event.target as HTMLInputElement).value))"
            />
          </div>
          <div class="flex items-center gap-6px">
            <label class="text-12px color-t3">Y</label>
            <input
              class="field w-80px!"
              type="number"
              :value="cropY"
              min="0"
              :max="Math.max(0, mediaHeight - cropHeight)"
              @input="emit('update:cropY', Number(($event.target as HTMLInputElement).value))"
            />
          </div>
          <span class="text-12px color-t3">
            原始 {{ mediaWidth }}×{{ mediaHeight }}
            · 预览 {{ Math.round(previewScale * 100) }}%
          </span>
          <button class="tb-btn" type="button" @click="emit('clearFile')">
            清除选择
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

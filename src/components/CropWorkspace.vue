<script setup lang="ts">
import type { FileInfo } from "../types";

defineProps<{
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
}>();
</script>

<template>
  <div
    class="shrink-0 w-full h-full border-b border-border bg-bg1"
    @mousemove="emit('mouseMove', $event)"
    @mouseup="emit('mouseUp')"
    @mouseleave="emit('mouseUp')"
  >
    <div class="px-14px py-12px flex flex-col gap-12px overflow-auto">
      <div class="flex items-center justify-between gap-12px flex-wrap">
        <div>
          <div class="text-13px font-600">自定义裁剪</div>
          <div class="text-11px color-t3 mt-2px">
            每次处理 1 个文件 · 绝对像素 · 支持图片与视频
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

      <div
        class="flex-1 flex gap-12px min-h-0 flex-wrap lg:flex-nowrap bg-bg-selected"
      >
        <!-- 文件选择 -->
        <div
          class="w-full lg:w-240px shrink-0 border border-border rounded-8px bg-bg0 overflow-hidden flex flex-col max-h-320px"
        >
          <div
            class="h-32px px-10px flex items-center text-11px font-500 color-t3 border-b border-border bg-bg2"
          >
            选择文件
          </div>
          <div class="flex-1 overflow-y-auto">
            <button
              v-for="f in files"
              :key="f.path"
              type="button"
              class="w-full text-left px-10px py-8px border-none border-b border-border/60 cursor-pointer text-12px truncate"
              :class="
                selectedPath === f.path
                  ? 'bg-secondary-soft color-secondary'
                  : 'bg-transparent color-t2 hover:bg-bg2'
              "
              @click="emit('selectFile', f)"
            >
              <span class="font-500">{{ f.name }}</span>
              <span class="ml-6px text-10px color-t3">{{
                f.file_type === "video" ? "视频" : "图片"
              }}</span>
            </button>
            <div
              v-if="files.length === 0"
              class="p-16px text-11px color-t3 text-center"
            >
              输入目录无媒体文件
            </div>
          </div>
        </div>

        <!-- 预览 -->
        <div class="flex-1 min-w-0 flex flex-col items-start gap-10px">
          <div
            v-if="isLoading"
            class="text-12px color-t3 py-40px w-full text-center"
          >
            加载预览中…
          </div>
          <div
            v-else-if="!cropFrameImage"
            class="text-12px color-t3 py-40px w-full text-center border border-dashed border-border rounded-8px"
          >
            请从左侧选择一个文件
          </div>
          <template v-else>
            <div class="relative inline-block bg-black">
              <img
                :src="cropFrameImage"
                class="block select-none"
                :style="{
                  width: mediaWidth * previewScale + 'px',
                  height: mediaHeight * previewScale + 'px',
                }"
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
                    height:
                      (mediaHeight - cropY - cropHeight) * previewScale + 'px',
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
                    width:
                      (mediaWidth - cropX - cropWidth) * previewScale + 'px',
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
                    'cursor-n-resize -top-5px left-50% -translate-x-50%':
                      h === 'n',
                    'cursor-s-resize -bottom-5px left-50% -translate-x-50%':
                      h === 's',
                    'cursor-e-resize -right-5px top-50% -translate-y-50%':
                      h === 'e',
                    'cursor-w-resize -left-5px top-50% -translate-y-50%':
                      h === 'w',
                    'cursor-ne-resize -top-5px -right-5px': h === 'ne',
                    'cursor-nw-resize -top-5px -left-5px': h === 'nw',
                    'cursor-se-resize -bottom-5px -right-5px': h === 'se',
                    'cursor-sw-resize -bottom-5px -left-5px': h === 'sw',
                  }"
                  @mousedown.stop="emit('startResize', $event, h)"
                ></div>
                <div
                  class="absolute bottom-5px left-50% -translate-x-50% bg-black/70 color-white px-8px py-2px rounded-3px text-12px whitespace-nowrap"
                >
                  {{ cropWidth }} × {{ cropHeight }}
                </div>
              </div>
            </div>

            <div class="flex gap-14px flex-wrap items-center">
              <div class="flex items-center gap-6px">
                <label class="text-12px color-t3">宽</label>
                <input
                  class="field w-80px!"
                  type="number"
                  :value="cropWidth"
                  min="100"
                  :max="mediaWidth"
                  @input="
                    emit(
                      'update:cropWidth',
                      Number(($event.target as HTMLInputElement).value),
                    )
                  "
                />
              </div>
              <div class="flex items-center gap-6px">
                <label class="text-12px color-t3">高</label>
                <input
                  class="field w-80px!"
                  type="number"
                  :value="cropHeight"
                  min="100"
                  :max="mediaHeight"
                  @input="
                    emit(
                      'update:cropHeight',
                      Number(($event.target as HTMLInputElement).value),
                    )
                  "
                />
              </div>
              <div class="flex items-center gap-6px">
                <label class="text-12px color-t3">X</label>
                <input
                  class="field w-80px!"
                  type="number"
                  :value="cropX"
                  min="0"
                  :max="mediaWidth - cropWidth"
                  @input="
                    emit(
                      'update:cropX',
                      Number(($event.target as HTMLInputElement).value),
                    )
                  "
                />
              </div>
              <div class="flex items-center gap-6px">
                <label class="text-12px color-t3">Y</label>
                <input
                  class="field w-80px!"
                  type="number"
                  :value="cropY"
                  min="0"
                  :max="mediaHeight - cropHeight"
                  @input="
                    emit(
                      'update:cropY',
                      Number(($event.target as HTMLInputElement).value),
                    )
                  "
                />
              </div>
              <span class="text-12px color-t3"
                >原始: {{ mediaWidth }} × {{ mediaHeight }}</span
              >
              <button class="tb-btn" type="button" @click="emit('clearFile')">
                清除选择
              </button>
            </div>
          </template>
        </div>
      </div>
    </div>
  </div>
</template>

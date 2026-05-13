<script setup lang="ts">
import type { ProcessOptions } from "../types";

defineProps<{
  visible: boolean;
  options: ProcessOptions;
  cropFrameImage: string;
  cropVideoWidth: number;
  cropVideoHeight: number;
  previewScale: number;
}>();

const emit = defineEmits<{
  close: [];
  confirm: [];
  startDrag: [e: MouseEvent];
  startResize: [e: MouseEvent, handle: string];
  mouseMove: [e: MouseEvent];
  mouseUp: [];
}>();
</script>

<template>
  <div
    class="fixed inset-0 bg-black/70 flex items-center justify-center z-1000"
    v-if="visible"
    @mousemove="emit('mouseMove', $event)"
    @mouseup="emit('mouseUp')"
    @mouseleave="emit('mouseUp')"
  >
    <div class="bg-white rounded-8px p-20px max-w-90% max-h-90% overflow-auto dark:bg-#2d2d2d dark:color-#f6f6f6">
      <h3 class="mb-10px text-18px">设置裁剪区域</h3>
      <p class="text-12px color-#666 mb-15px dark:color-#aaa">拖拽移动裁剪框，拖拽边角调整大小</p>

      <div class="relative inline-block bg-black mb-15px">
        <img
          :src="cropFrameImage"
          :style="{
            width: cropVideoWidth * previewScale + 'px',
            height: cropVideoHeight * previewScale + 'px',
          }"
          draggable="false"
          class="block select-none"
        />

        <!-- 裁剪区域遮罩 -->
        <div class="absolute inset-0 w-full h-full pointer-events-none">
          <div
            class="absolute top-0 left-0 w-full bg-black/50"
            :style="{ height: options.crop_y * previewScale + 'px' }"
          ></div>
          <div
            class="absolute bottom-0 left-0 w-full bg-black/50"
            :style="{
              height:
                (cropVideoHeight - options.crop_y - options.crop_height) *
                  previewScale +
                'px',
            }"
          ></div>
          <div
            class="absolute left-0 bg-black/50"
            :style="{
              top: options.crop_y * previewScale + 'px',
              height: options.crop_height * previewScale + 'px',
              width: options.crop_x * previewScale + 'px',
            }"
          ></div>
          <div
            class="absolute right-0 bg-black/50"
            :style="{
              top: options.crop_y * previewScale + 'px',
              height: options.crop_height * previewScale + 'px',
              width:
                (cropVideoWidth - options.crop_x - options.crop_width) *
                  previewScale +
                'px',
            }"
          ></div>
        </div>

        <!-- 裁剪选择框 -->
        <div
          class="crop-area absolute border-2 border-#007bff cursor-move box-border"
          :style="{
            left: options.crop_x * previewScale + 'px',
            top: options.crop_y * previewScale + 'px',
            width: options.crop_width * previewScale + 'px',
            height: options.crop_height * previewScale + 'px',
          }"
          @mousedown="emit('startDrag', $event)"
        >
          <div class="resize-handle absolute w-10px h-10px bg-#007bff border border-white cursor-n-resize -top-5px left-50% -translate-x-50%" @mousedown.stop="emit('startResize', $event, 'n')"></div>
          <div class="resize-handle absolute w-10px h-10px bg-#007bff border border-white cursor-s-resize -bottom-5px left-50% -translate-x-50%" @mousedown.stop="emit('startResize', $event, 's')"></div>
          <div class="resize-handle absolute w-10px h-10px bg-#007bff border border-white cursor-e-resize -right-5px top-50% -translate-y-50%" @mousedown.stop="emit('startResize', $event, 'e')"></div>
          <div class="resize-handle absolute w-10px h-10px bg-#007bff border border-white cursor-w-resize -left-5px top-50% -translate-y-50%" @mousedown.stop="emit('startResize', $event, 'w')"></div>
          <div class="resize-handle absolute w-10px h-10px bg-#007bff border border-white cursor-ne-resize -top-5px -right-5px" @mousedown.stop="emit('startResize', $event, 'ne')"></div>
          <div class="resize-handle absolute w-10px h-10px bg-#007bff border border-white cursor-nw-resize -top-5px -left-5px" @mousedown.stop="emit('startResize', $event, 'nw')"></div>
          <div class="resize-handle absolute w-10px h-10px bg-#007bff border border-white cursor-se-resize -bottom-5px -right-5px" @mousedown.stop="emit('startResize', $event, 'se')"></div>
          <div class="resize-handle absolute w-10px h-10px bg-#007bff border border-white cursor-sw-resize -bottom-5px -left-5px" @mousedown.stop="emit('startResize', $event, 'sw')"></div>

          <div class="absolute bottom-5px left-50% -translate-x-50% bg-black/70 color-white p-2px-8px rounded-3px text-12px whitespace-nowrap">
            {{ options.crop_width }} x {{ options.crop_height }}
          </div>
        </div>
      </div>

      <!-- 精确输入 -->
      <div class="flex gap-15px mb-10px flex-wrap">
        <div class="flex items-center gap-5px">
          <label class="text-12px color-#666 dark:color-#aaa">宽度:</label>
          <input type="number" v-model.number="options.crop_width" min="100" :max="cropVideoWidth" class="w-80px p-5px border border-#ddd rounded-4px dark:bg-#444 dark:border-#555 dark:color-#f6f6f6" />
        </div>
        <div class="flex items-center gap-5px">
          <label class="text-12px color-#666 dark:color-#aaa">高度:</label>
          <input type="number" v-model.number="options.crop_height" min="100" :max="cropVideoHeight" class="w-80px p-5px border border-#ddd rounded-4px dark:bg-#444 dark:border-#555 dark:color-#f6f6f6" />
        </div>
        <div class="flex items-center gap-5px">
          <label class="text-12px color-#666 dark:color-#aaa">X:</label>
          <input type="number" v-model.number="options.crop_x" min="0" :max="cropVideoWidth - options.crop_width" class="w-80px p-5px border border-#ddd rounded-4px dark:bg-#444 dark:border-#555 dark:color-#f6f6f6" />
        </div>
        <div class="flex items-center gap-5px">
          <label class="text-12px color-#666 dark:color-#aaa">Y:</label>
          <input type="number" v-model.number="options.crop_y" min="0" :max="cropVideoHeight - options.crop_height" class="w-80px p-5px border border-#ddd rounded-4px dark:bg-#444 dark:border-#555 dark:color-#f6f6f6" />
        </div>
      </div>

      <div class="text-12px color-#999 mb-15px dark:color-#888">
        原始尺寸: {{ cropVideoWidth }} x {{ cropVideoHeight }}
      </div>

      <div class="flex justify-end gap-10px">
        <button @click="emit('close')">取消</button>
        <button class="w-auto! p-8px-20px! bg-#007bff hover:not-disabled:bg-#0056b3" @click="emit('confirm')">确认</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.crop-area::before {
  content: "";
  position: absolute;
  top: 33.33%;
  left: 0;
  right: 0;
  height: 1px;
  background: rgba(255, 255, 255, 0.5);
}
.crop-area::after {
  content: "";
  position: absolute;
  top: 66.66%;
  left: 0;
  right: 0;
  height: 1px;
  background: rgba(255, 255, 255, 0.5);
}
img {
  -webkit-user-drag: none;
}
</style>

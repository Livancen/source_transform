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
    v-if="visible"
    class="modal-mask"
    @mousemove="emit('mouseMove', $event)"
    @mouseup="emit('mouseUp')"
    @mouseleave="emit('mouseUp')"
  >
    <div class="modal-panel p-20px max-w-90%">
      <h3 class="text-16px font-600 mb-8px">设置裁剪区域</h3>
      <p class="text-12px color-t3 mb-14px">拖拽移动裁剪框，拖拽边角调整大小</p>

      <div class="relative inline-block bg-black mb-14px">
        <img
          :src="cropFrameImage"
          class="block select-none"
          :style="{
            width: cropVideoWidth * previewScale + 'px',
            height: cropVideoHeight * previewScale + 'px',
          }"
          draggable="false"
        />

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

        <div
          class="crop-area-box absolute border-2 border-accent cursor-move box-border"
          :style="{
            left: options.crop_x * previewScale + 'px',
            top: options.crop_y * previewScale + 'px',
            width: options.crop_width * previewScale + 'px',
            height: options.crop_height * previewScale + 'px',
          }"
          @mousedown="emit('startDrag', $event)"
        >
          <div
            class="absolute w-10px h-10px bg-accent border border-white cursor-n-resize -top-5px left-50% -translate-x-50%"
            @mousedown.stop="emit('startResize', $event, 'n')"
          ></div>
          <div
            class="absolute w-10px h-10px bg-accent border border-white cursor-s-resize -bottom-5px left-50% -translate-x-50%"
            @mousedown.stop="emit('startResize', $event, 's')"
          ></div>
          <div
            class="absolute w-10px h-10px bg-accent border border-white cursor-e-resize -right-5px top-50% -translate-y-50%"
            @mousedown.stop="emit('startResize', $event, 'e')"
          ></div>
          <div
            class="absolute w-10px h-10px bg-accent border border-white cursor-w-resize -left-5px top-50% -translate-y-50%"
            @mousedown.stop="emit('startResize', $event, 'w')"
          ></div>
          <div
            class="absolute w-10px h-10px bg-accent border border-white cursor-ne-resize -top-5px -right-5px"
            @mousedown.stop="emit('startResize', $event, 'ne')"
          ></div>
          <div
            class="absolute w-10px h-10px bg-accent border border-white cursor-nw-resize -top-5px -left-5px"
            @mousedown.stop="emit('startResize', $event, 'nw')"
          ></div>
          <div
            class="absolute w-10px h-10px bg-accent border border-white cursor-se-resize -bottom-5px -right-5px"
            @mousedown.stop="emit('startResize', $event, 'se')"
          ></div>
          <div
            class="absolute w-10px h-10px bg-accent border border-white cursor-sw-resize -bottom-5px -left-5px"
            @mousedown.stop="emit('startResize', $event, 'sw')"
          ></div>

          <div
            class="absolute bottom-5px left-50% -translate-x-50% bg-black/70 color-white px-8px py-2px rounded-3px text-12px whitespace-nowrap"
          >
            {{ options.crop_width }} x {{ options.crop_height }}
          </div>
        </div>
      </div>

      <div class="flex gap-14px mb-10px flex-wrap">
        <div class="flex items-center gap-6px">
          <label class="text-12px color-t3">宽度</label>
          <input
            class="field w-80px!"
            type="number"
            v-model.number="options.crop_width"
            min="100"
            :max="cropVideoWidth"
          />
        </div>
        <div class="flex items-center gap-6px">
          <label class="text-12px color-t3">高度</label>
          <input
            class="field w-80px!"
            type="number"
            v-model.number="options.crop_height"
            min="100"
            :max="cropVideoHeight"
          />
        </div>
        <div class="flex items-center gap-6px">
          <label class="text-12px color-t3">X</label>
          <input
            class="field w-80px!"
            type="number"
            v-model.number="options.crop_x"
            min="0"
            :max="cropVideoWidth - options.crop_width"
          />
        </div>
        <div class="flex items-center gap-6px">
          <label class="text-12px color-t3">Y</label>
          <input
            class="field w-80px!"
            type="number"
            v-model.number="options.crop_y"
            min="0"
            :max="cropVideoHeight - options.crop_height"
          />
        </div>
      </div>

      <div class="text-12px color-t3 mb-14px">
        原始尺寸: {{ cropVideoWidth }} x {{ cropVideoHeight }}
      </div>

      <div class="flex justify-end gap-8px">
        <button class="modal-btn" type="button" @click="emit('close')">取消</button>
        <button class="modal-btn modal-btn-primary" type="button" @click="emit('confirm')">确认</button>
      </div>
    </div>
  </div>
</template>

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
    <div class="modal-panel" style="padding: 20px; max-width: 90%">
      <h3 style="margin-bottom: 8px">设置裁剪区域</h3>
      <p style="font-size: 12px; color: var(--text-3); margin-bottom: 14px">
        拖拽移动裁剪框，拖拽边角调整大小
      </p>

      <div style="position: relative; display: inline-block; background: #000; margin-bottom: 14px">
        <img
          :src="cropFrameImage"
          :style="{
            width: cropVideoWidth * previewScale + 'px',
            height: cropVideoHeight * previewScale + 'px',
          }"
          draggable="false"
          style="display: block; user-select: none"
        />

        <div style="position: absolute; inset: 0; width: 100%; height: 100%; pointer-events: none">
          <div
            style="position: absolute; top: 0; left: 0; width: 100%; background: rgba(0,0,0,0.5)"
            :style="{ height: options.crop_y * previewScale + 'px' }"
          ></div>
          <div
            style="position: absolute; bottom: 0; left: 0; width: 100%; background: rgba(0,0,0,0.5)"
            :style="{
              height:
                (cropVideoHeight - options.crop_y - options.crop_height) *
                  previewScale +
                'px',
            }"
          ></div>
          <div
            style="position: absolute; left: 0; background: rgba(0,0,0,0.5)"
            :style="{
              top: options.crop_y * previewScale + 'px',
              height: options.crop_height * previewScale + 'px',
              width: options.crop_x * previewScale + 'px',
            }"
          ></div>
          <div
            style="position: absolute; right: 0; background: rgba(0,0,0,0.5)"
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
          class="crop-area"
          :style="{
            left: options.crop_x * previewScale + 'px',
            top: options.crop_y * previewScale + 'px',
            width: options.crop_width * previewScale + 'px',
            height: options.crop_height * previewScale + 'px',
          }"
          @mousedown="emit('startDrag', $event)"
        >
          <div class="resize-handle n" @mousedown.stop="emit('startResize', $event, 'n')"></div>
          <div class="resize-handle s" @mousedown.stop="emit('startResize', $event, 's')"></div>
          <div class="resize-handle e" @mousedown.stop="emit('startResize', $event, 'e')"></div>
          <div class="resize-handle w" @mousedown.stop="emit('startResize', $event, 'w')"></div>
          <div class="resize-handle ne" @mousedown.stop="emit('startResize', $event, 'ne')"></div>
          <div class="resize-handle nw" @mousedown.stop="emit('startResize', $event, 'nw')"></div>
          <div class="resize-handle se" @mousedown.stop="emit('startResize', $event, 'se')"></div>
          <div class="resize-handle sw" @mousedown.stop="emit('startResize', $event, 'sw')"></div>

          <div class="crop-size-label">
            {{ options.crop_width }} x {{ options.crop_height }}
          </div>
        </div>
      </div>

      <div style="display: flex; gap: 14px; margin-bottom: 10px; flex-wrap: wrap">
        <div style="display: flex; align-items: center; gap: 6px">
          <label style="font-size: 12px; color: var(--text-3)">宽度</label>
          <input class="field" type="number" v-model.number="options.crop_width" min="100" :max="cropVideoWidth" style="width: 80px" />
        </div>
        <div style="display: flex; align-items: center; gap: 6px">
          <label style="font-size: 12px; color: var(--text-3)">高度</label>
          <input class="field" type="number" v-model.number="options.crop_height" min="100" :max="cropVideoHeight" style="width: 80px" />
        </div>
        <div style="display: flex; align-items: center; gap: 6px">
          <label style="font-size: 12px; color: var(--text-3)">X</label>
          <input class="field" type="number" v-model.number="options.crop_x" min="0" :max="cropVideoWidth - options.crop_width" style="width: 80px" />
        </div>
        <div style="display: flex; align-items: center; gap: 6px">
          <label style="font-size: 12px; color: var(--text-3)">Y</label>
          <input class="field" type="number" v-model.number="options.crop_y" min="0" :max="cropVideoHeight - options.crop_height" style="width: 80px" />
        </div>
      </div>

      <div style="font-size: 12px; color: var(--text-3); margin-bottom: 14px">
        原始尺寸: {{ cropVideoWidth }} x {{ cropVideoHeight }}
      </div>

      <div style="display: flex; justify-content: flex-end; gap: 8px">
        <button class="modal-btn" type="button" @click="emit('close')">取消</button>
        <button class="modal-btn primary" type="button" @click="emit('confirm')">确认</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.crop-area {
  position: absolute;
  border: 2px solid var(--accent);
  cursor: move;
  box-sizing: border-box;
}

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

.resize-handle {
  position: absolute;
  width: 10px;
  height: 10px;
  background: var(--accent);
  border: 1px solid #fff;
}

.resize-handle.n {
  top: -5px;
  left: 50%;
  transform: translateX(-50%);
  cursor: n-resize;
}
.resize-handle.s {
  bottom: -5px;
  left: 50%;
  transform: translateX(-50%);
  cursor: s-resize;
}
.resize-handle.e {
  right: -5px;
  top: 50%;
  transform: translateY(-50%);
  cursor: e-resize;
}
.resize-handle.w {
  left: -5px;
  top: 50%;
  transform: translateY(-50%);
  cursor: w-resize;
}
.resize-handle.ne {
  top: -5px;
  right: -5px;
  cursor: ne-resize;
}
.resize-handle.nw {
  top: -5px;
  left: -5px;
  cursor: nw-resize;
}
.resize-handle.se {
  bottom: -5px;
  right: -5px;
  cursor: se-resize;
}
.resize-handle.sw {
  bottom: -5px;
  left: -5px;
  cursor: sw-resize;
}

.crop-size-label {
  position: absolute;
  bottom: 5px;
  left: 50%;
  transform: translateX(-50%);
  background: rgba(0, 0, 0, 0.7);
  color: white;
  padding: 2px 8px;
  border-radius: 3px;
  font-size: 12px;
  white-space: nowrap;
}

img {
  -webkit-user-drag: none;
}
</style>

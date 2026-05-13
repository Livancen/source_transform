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
    class="crop-modal"
    v-if="visible"
    @mousemove="emit('mouseMove', $event)"
    @mouseup="emit('mouseUp')"
    @mouseleave="emit('mouseUp')"
  >
    <div class="crop-modal-content">
      <h3>设置裁剪区域</h3>
      <p class="crop-hint">拖拽移动裁剪框，拖拽边角调整大小</p>

      <div class="crop-preview-container">
        <img
          :src="cropFrameImage"
          :style="{
            width: cropVideoWidth * previewScale + 'px',
            height: cropVideoHeight * previewScale + 'px',
          }"
          draggable="false"
        />

        <!-- 裁剪区域遮罩 -->
        <div class="crop-overlay">
          <div
            class="crop-mask crop-mask-top"
            :style="{ height: options.crop_y * previewScale + 'px' }"
          ></div>
          <div
            class="crop-mask crop-mask-bottom"
            :style="{
              height:
                (cropVideoHeight - options.crop_y - options.crop_height) *
                  previewScale +
                'px',
            }"
          ></div>
          <div
            class="crop-mask crop-mask-left"
            :style="{
              top: options.crop_y * previewScale + 'px',
              height: options.crop_height * previewScale + 'px',
              width: options.crop_x * previewScale + 'px',
            }"
          ></div>
          <div
            class="crop-mask crop-mask-right"
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
          class="crop-area"
          :style="{
            left: options.crop_x * previewScale + 'px',
            top: options.crop_y * previewScale + 'px',
            width: options.crop_width * previewScale + 'px',
            height: options.crop_height * previewScale + 'px',
          }"
          @mousedown="emit('startDrag', $event)"
        >
          <div
            class="resize-handle resize-n"
            @mousedown.stop="emit('startResize', $event, 'n')"
          ></div>
          <div
            class="resize-handle resize-s"
            @mousedown.stop="emit('startResize', $event, 's')"
          ></div>
          <div
            class="resize-handle resize-e"
            @mousedown.stop="emit('startResize', $event, 'e')"
          ></div>
          <div
            class="resize-handle resize-w"
            @mousedown.stop="emit('startResize', $event, 'w')"
          ></div>
          <div
            class="resize-handle resize-ne"
            @mousedown.stop="emit('startResize', $event, 'ne')"
          ></div>
          <div
            class="resize-handle resize-nw"
            @mousedown.stop="emit('startResize', $event, 'nw')"
          ></div>
          <div
            class="resize-handle resize-se"
            @mousedown.stop="emit('startResize', $event, 'se')"
          ></div>
          <div
            class="resize-handle resize-sw"
            @mousedown.stop="emit('startResize', $event, 'sw')"
          ></div>

          <div class="crop-size-label">
            {{ options.crop_width }} x {{ options.crop_height }}
          </div>
        </div>
      </div>

      <!-- 精确输入 -->
      <div class="crop-inputs">
        <div class="crop-input-group">
          <label>宽度:</label>
          <input
            type="number"
            v-model.number="options.crop_width"
            min="100"
            :max="cropVideoWidth"
          />
        </div>
        <div class="crop-input-group">
          <label>高度:</label>
          <input
            type="number"
            v-model.number="options.crop_height"
            min="100"
            :max="cropVideoHeight"
          />
        </div>
        <div class="crop-input-group">
          <label>X:</label>
          <input
            type="number"
            v-model.number="options.crop_x"
            min="0"
            :max="cropVideoWidth - options.crop_width"
          />
        </div>
        <div class="crop-input-group">
          <label>Y:</label>
          <input
            type="number"
            v-model.number="options.crop_y"
            min="0"
            :max="cropVideoHeight - options.crop_height"
          />
        </div>
      </div>

      <div class="crop-video-info">
        原始尺寸: {{ cropVideoWidth }} x {{ cropVideoHeight }}
      </div>

      <div class="crop-modal-actions">
        <button @click="emit('close')">取消</button>
        <button class="primary-btn" @click="emit('confirm')">确认</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.crop-modal {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background: rgba(0, 0, 0, 0.7);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.crop-modal-content {
  background: #fff;
  border-radius: 8px;
  padding: 20px;
  max-width: 90%;
  max-height: 90%;
  overflow: auto;
}

.crop-modal-content h3 {
  margin-bottom: 10px;
  font-size: 18px;
}

.crop-hint {
  font-size: 12px;
  color: #666;
  margin-bottom: 15px;
}

.crop-preview-container {
  position: relative;
  display: inline-block;
  background: #000;
  margin-bottom: 15px;
}

.crop-preview-container img {
  display: block;
  user-select: none;
  -webkit-user-drag: none;
}

.crop-overlay {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  pointer-events: none;
}

.crop-mask {
  position: absolute;
  background: rgba(0, 0, 0, 0.5);
}

.crop-mask-top {
  top: 0;
  left: 0;
  width: 100%;
}

.crop-mask-bottom {
  bottom: 0;
  left: 0;
  width: 100%;
}

.crop-mask-left {
  left: 0;
}

.crop-mask-right {
  right: 0;
}

.crop-area {
  position: absolute;
  border: 2px solid #007bff;
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
  background: #007bff;
  border: 1px solid #fff;
}

.resize-n {
  top: -5px;
  left: 50%;
  transform: translateX(-50%);
  cursor: n-resize;
}

.resize-s {
  bottom: -5px;
  left: 50%;
  transform: translateX(-50%);
  cursor: s-resize;
}

.resize-e {
  right: -5px;
  top: 50%;
  transform: translateY(-50%);
  cursor: e-resize;
}

.resize-w {
  left: -5px;
  top: 50%;
  transform: translateY(-50%);
  cursor: w-resize;
}

.resize-ne {
  top: -5px;
  right: -5px;
  cursor: ne-resize;
}

.resize-nw {
  top: -5px;
  left: -5px;
  cursor: nw-resize;
}

.resize-se {
  bottom: -5px;
  right: -5px;
  cursor: se-resize;
}

.resize-sw {
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
  color: #fff;
  padding: 2px 8px;
  border-radius: 3px;
  font-size: 12px;
  white-space: nowrap;
}

.crop-inputs {
  display: flex;
  gap: 15px;
  margin-bottom: 10px;
  flex-wrap: wrap;
}

.crop-input-group {
  display: flex;
  align-items: center;
  gap: 5px;
}

.crop-input-group label {
  font-size: 12px;
  color: #666;
}

.crop-input-group input {
  width: 80px;
  padding: 5px;
  border: 1px solid #ddd;
  border-radius: 4px;
}

.crop-video-info {
  font-size: 12px;
  color: #999;
  margin-bottom: 15px;
}

.crop-modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}

.crop-modal-actions .primary-btn {
  width: auto;
  padding: 8px 20px;
}

@media (prefers-color-scheme: dark) {
  .crop-modal-content {
    background: #2d2d2d;
    color: #f6f6f6;
  }

  .crop-hint {
    color: #aaa;
  }

  .crop-input-group label {
    color: #aaa;
  }

  .crop-input-group input {
    background: #444;
    border-color: #555;
    color: #f6f6f6;
  }

  .crop-video-info {
    color: #888;
  }
}
</style>

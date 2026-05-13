<script setup lang="ts">
import type { ProcessOptions, FileInfo } from "../types";

defineProps<{
  options: ProcessOptions;
  videoFiles: FileInfo[];
}>();

const emit = defineEmits<{
  openCropPreview: [];
}>();
</script>

<template>
  <section class="section">
    <h2>处理选项</h2>
    <div class="options-grid">
      <!-- 左列 -->
      <div class="options-column">
        <!-- 压缩 -->
        <div class="option-group">
          <label class="checkbox-label">
            <input type="checkbox" v-model="options.compress" />
            启用压缩
          </label>
          <div v-if="options.compress" class="option-detail">
            <label>质量:</label>
            <input
              type="range"
              v-model.number="options.compress_quality"
              min="1"
              max="100"
            />
            <span>{{ options.compress_quality }}</span>
          </div>
          <div v-if="options.compress" class="compress-mode">
            <label class="radio-label">
              <input
                type="radio"
                :value="false"
                v-model="options.compress_resize"
              />
              仅压缩质量（保持原分辨率）
            </label>
            <label class="radio-label">
              <input
                type="radio"
                :value="true"
                v-model="options.compress_resize"
              />
              压缩质量 + 降低分辨率
            </label>
            <div
              v-if="options.compress_resize"
              class="option-detail compress-size"
            >
              <label>宽:</label>
              <input
                type="number"
                v-model.number="options.compress_width"
                min="1"
              />
              <label>高:</label>
              <input
                type="number"
                v-model.number="options.compress_height"
                min="1"
              />
            </div>
          </div>
        </div>

        <!-- 降低分辨率 -->
        <div class="option-group">
          <label class="checkbox-label">
            <input type="checkbox" v-model="options.reduce_resolution" />
            降低分辨率
          </label>
          <div v-if="options.reduce_resolution" class="option-detail">
            <label>宽:</label>
            <input
              type="number"
              v-model.number="options.target_width"
              min="1"
            />
            <label>高:</label>
            <input
              type="number"
              v-model.number="options.target_height"
              min="1"
            />
          </div>
        </div>

        <!-- 旋转 -->
        <div class="option-group">
          <label class="checkbox-label">
            <input type="checkbox" v-model="options.rotate" />
            旋转
          </label>
          <div v-if="options.rotate" class="option-detail">
            <label>角度:</label>
            <select v-model.number="options.rotation_degrees">
              <option :value="90">顺时针 90°</option>
              <option :value="180">180°</option>
              <option :value="270">顺时针 270°</option>
              <option :value="-90">逆时针 90°</option>
            </select>
          </div>
        </div>

        <!-- 视频静音 -->
        <div class="option-group">
          <label class="checkbox-label">
            <input type="checkbox" v-model="options.mute" />
            视频静音（去除音频）
          </label>
        </div>

        <!-- 调整帧率 -->
        <div class="option-group">
          <label class="checkbox-label">
            <input type="checkbox" v-model="options.change_framerate" />
            调整帧率
          </label>
          <div v-if="options.change_framerate" class="option-detail">
            <label>目标帧率:</label>
            <input
              type="number"
              v-model.number="options.target_framerate"
              min="1"
              max="120"
              step="1"
            />
            <span class="unit">fps</span>
          </div>
        </div>

        <!-- 裁剪 (仅视频) -->
        <div class="option-group">
          <label class="checkbox-label">
            <input type="checkbox" v-model="options.crop" />
            裁剪 (视频)
          </label>
          <div v-if="options.crop" class="option-detail crop-info">
            <span>{{ options.crop_width }} x {{ options.crop_height }}</span>
            <span>位置: ({{ options.crop_x }}, {{ options.crop_y }})</span>
            <button
              class="crop-btn"
              @click="emit('openCropPreview')"
              :disabled="videoFiles.length === 0"
            >
              设置裁剪区域
            </button>
          </div>
        </div>
      </div>

      <!-- 右列 -->
      <div class="options-column">
        <!-- 降低码率 (仅视频) -->
        <div class="option-group">
          <label class="checkbox-label">
            <input type="checkbox" v-model="options.reduce_bitrate" />
            降低码率 (视频)
          </label>
          <div v-if="options.reduce_bitrate" class="option-detail">
            <label>码率:</label>
            <select v-model="options.target_bitrate">
              <option value="200k">200 Kbps</option>
              <option value="500k">500 Kbps</option>
              <option value="800k">800 Kbps</option>
              <option value="1M">1 Mbps</option>
              <option value="2M">2 Mbps</option>
              <option value="3M">3 Mbps</option>
              <option value="5M">5 Mbps</option>
              <option value="8M">8 Mbps</option>
              <option value="10M">10 Mbps</option>
              <option value="15M">15 Mbps</option>
              <option value="20M">20 Mbps</option>
            </select>
          </div>
        </div>

        <!-- 降低Level等级 (仅视频) -->
        <div class="option-group">
          <label class="checkbox-label">
            <input type="checkbox" v-model="options.reduce_level" />
            降低Level (视频)
          </label>
          <div v-if="options.reduce_level" class="option-detail">
            <label>Profile:</label>
            <select v-model="options.target_profile">
              <option value="baseline">Baseline</option>
              <option value="main">Main</option>
              <option value="high">High</option>
            </select>
            <label>Level:</label>
            <select v-model="options.target_level">
              <option value="3.0">3.0 (SD 480p)</option>
              <option value="3.1">3.1 (720p@30fps)</option>
              <option value="4.0">4.0 (1080p@30fps)</option>
              <option value="4.1">4.1 (1080p@30fps)</option>
              <option value="4.2">4.2 (1080p@60fps)</option>
              <option value="5.0">5.0 (2K)</option>
              <option value="5.1">5.1 (4K@30fps)</option>
              <option value="5.2">5.2 (4K@60fps)</option>
            </select>
          </div>
        </div>

        <!-- H.265转H.264 (仅视频) -->
        <div class="option-group">
          <label class="checkbox-label">
            <input type="checkbox" v-model="options.convert_h265_to_h264" />
            H.265 转 H.264 (视频)
          </label>
        </div>

        <!-- 视频格式转换 -->
        <div class="option-group">
          <label class="checkbox-label">
            <input type="checkbox" v-model="options.convert_format" />
            格式转换 (视频)
          </label>
          <div v-if="options.convert_format" class="option-detail">
            <label>目标格式:</label>
            <select v-model="options.target_format">
              <option value="mp4">MP4</option>
              <option value="avi">AVI</option>
              <option value="mkv">MKV</option>
              <option value="mov">MOV</option>
              <option value="webm">WebM</option>
              <option value="flv">FLV</option>
            </select>
          </div>
        </div>
      </div>
    </div>
  </section>
</template>

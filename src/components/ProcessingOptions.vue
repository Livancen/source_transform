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
  <section class="bg-white rounded-8px p-15px mb-15px shadow-sm dark:bg-#2d2d2d">
    <h2 class="text-16px mb-10px color-#555 border-b border-b-#ddd border-b-solid pb-5px dark:color-#ccc dark:border-b-#444">处理选项</h2>
    <div class="flex gap-15px">
      <!-- 左列 -->
      <div class="flex-1">
        <!-- 压缩 -->
        <div class="mb-10px last:mb-0 p-10px bg-#f9f9f9 rounded-4px dark:bg-#333">
          <label class="flex items-center gap-8px cursor-pointer font-500">
            <input type="checkbox" v-model="options.compress" class="w-18px h-18px" />
            启用压缩
          </label>
          <div v-if="options.compress" class="mt-10px pl-26px flex items-center gap-10px flex-wrap">
            <label class="color-#666 dark:color-#aaa">质量:</label>
            <input
              type="range"
              v-model.number="options.compress_quality"
              min="1"
              max="100"
              class="w-150px"
            />
            <span>{{ options.compress_quality }}</span>
          </div>
          <div v-if="options.compress" class="mt-10px pl-26px">
            <label class="flex items-center gap-6px cursor-pointer text-13px mb-6px">
              <input
                type="radio"
                :value="false"
                v-model="options.compress_resize"
                class="w-14px h-14px"
              />
              仅压缩质量（保持原分辨率）
            </label>
            <label class="flex items-center gap-6px cursor-pointer text-13px mb-6px">
              <input
                type="radio"
                :value="true"
                v-model="options.compress_resize"
                class="w-14px h-14px"
              />
              压缩质量 + 降低分辨率
            </label>
            <div
              v-if="options.compress_resize"
              class="mt-8px pl-20px flex items-center gap-10px flex-wrap"
            >
              <label class="color-#666 dark:color-#aaa">宽:</label>
              <input
                type="number"
                v-model.number="options.compress_width"
                min="1"
                class="w-100px p-5px border border-#ddd rounded-4px dark:bg-#444 dark:border-#555 dark:color-#f6f6f6"
              />
              <label class="color-#666 dark:color-#aaa">高:</label>
              <input
                type="number"
                v-model.number="options.compress_height"
                min="1"
                class="w-100px p-5px border border-#ddd rounded-4px dark:bg-#444 dark:border-#555 dark:color-#f6f6f6"
              />
            </div>
          </div>
        </div>

        <!-- 降低分辨率 -->
        <div class="mb-10px last:mb-0 p-10px bg-#f9f9f9 rounded-4px dark:bg-#333">
          <label class="flex items-center gap-8px cursor-pointer font-500">
            <input type="checkbox" v-model="options.reduce_resolution" class="w-18px h-18px" />
            降低分辨率
          </label>
          <div v-if="options.reduce_resolution" class="mt-10px pl-26px flex items-center gap-10px flex-wrap">
            <label class="color-#666 dark:color-#aaa">宽:</label>
            <input
              type="number"
              v-model.number="options.target_width"
              min="1"
              class="w-100px p-5px border border-#ddd rounded-4px dark:bg-#444 dark:border-#555 dark:color-#f6f6f6"
            />
            <label class="color-#666 dark:color-#aaa">高:</label>
            <input
              type="number"
              v-model.number="options.target_height"
              min="1"
              class="w-100px p-5px border border-#ddd rounded-4px dark:bg-#444 dark:border-#555 dark:color-#f6f6f6"
            />
          </div>
        </div>

        <!-- 旋转 -->
        <div class="mb-10px last:mb-0 p-10px bg-#f9f9f9 rounded-4px dark:bg-#333">
          <label class="flex items-center gap-8px cursor-pointer font-500">
            <input type="checkbox" v-model="options.rotate" class="w-18px h-18px" />
            旋转
          </label>
          <div v-if="options.rotate" class="mt-10px pl-26px flex items-center gap-10px flex-wrap">
            <label class="color-#666 dark:color-#aaa">角度:</label>
            <select v-model.number="options.rotation_degrees" class="p-5px-10px border border-#ddd rounded-4px dark:bg-#444 dark:border-#555 dark:color-#f6f6f6">
              <option :value="90">顺时针 90°</option>
              <option :value="180">180°</option>
              <option :value="270">顺时针 270°</option>
              <option :value="-90">逆时针 90°</option>
            </select>
          </div>
        </div>

        <!-- 视频静音 -->
        <div class="mb-10px last:mb-0 p-10px bg-#f9f9f9 rounded-4px dark:bg-#333">
          <label class="flex items-center gap-8px cursor-pointer font-500">
            <input type="checkbox" v-model="options.mute" class="w-18px h-18px" />
            视频静音（去除音频）
          </label>
        </div>

        <!-- 调整帧率 -->
        <div class="mb-10px last:mb-0 p-10px bg-#f9f9f9 rounded-4px dark:bg-#333">
          <label class="flex items-center gap-8px cursor-pointer font-500">
            <input type="checkbox" v-model="options.change_framerate" class="w-18px h-18px" />
            调整帧率
          </label>
          <div v-if="options.change_framerate" class="mt-10px pl-26px flex items-center gap-10px flex-wrap">
            <label class="color-#666 dark:color-#aaa">目标帧率:</label>
            <input
              type="number"
              v-model.number="options.target_framerate"
              min="1"
              max="120"
              step="1"
              class="w-100px p-5px border border-#ddd rounded-4px dark:bg-#444 dark:border-#555 dark:color-#f6f6f6"
            />
            <span>fps</span>
          </div>
        </div>

        <!-- 裁剪 (仅视频) -->
        <div class="mb-10px last:mb-0 p-10px bg-#f9f9f9 rounded-4px dark:bg-#333">
          <label class="flex items-center gap-8px cursor-pointer font-500">
            <input type="checkbox" v-model="options.crop" class="w-18px h-18px" />
            裁剪 (视频)
          </label>
          <div v-if="options.crop" class="mt-10px pl-26px flex flex-col items-start gap-5px">
            <span class="text-12px color-#666 dark:color-#aaa">{{ options.crop_width }} x {{ options.crop_height }}</span>
            <span class="text-12px color-#666 dark:color-#aaa">位置: ({{ options.crop_x }}, {{ options.crop_y }})</span>
            <button
              class="mt-5px p-4px-12px text-12px"
              @click="emit('openCropPreview')"
              :disabled="videoFiles.length === 0"
            >
              设置裁剪区域
            </button>
          </div>
        </div>
      </div>

      <!-- 右列 -->
      <div class="flex-1">
        <!-- 降低码率 (仅视频) -->
        <div class="mb-10px last:mb-0 p-10px bg-#f9f9f9 rounded-4px dark:bg-#333">
          <label class="flex items-center gap-8px cursor-pointer font-500">
            <input type="checkbox" v-model="options.reduce_bitrate" class="w-18px h-18px" />
            降低码率 (视频)
          </label>
          <div v-if="options.reduce_bitrate" class="mt-10px pl-26px flex items-center gap-10px flex-wrap">
            <label class="color-#666 dark:color-#aaa">码率:</label>
            <select v-model="options.target_bitrate" class="p-5px-10px border border-#ddd rounded-4px dark:bg-#444 dark:border-#555 dark:color-#f6f6f6">
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
        <div class="mb-10px last:mb-0 p-10px bg-#f9f9f9 rounded-4px dark:bg-#333">
          <label class="flex items-center gap-8px cursor-pointer font-500">
            <input type="checkbox" v-model="options.reduce_level" class="w-18px h-18px" />
            降低Level (视频)
          </label>
          <div v-if="options.reduce_level" class="mt-10px pl-26px flex items-center gap-10px flex-wrap">
            <label class="color-#666 dark:color-#aaa">Profile:</label>
            <select v-model="options.target_profile" class="p-5px-10px border border-#ddd rounded-4px dark:bg-#444 dark:border-#555 dark:color-#f6f6f6">
              <option value="baseline">Baseline</option>
              <option value="main">Main</option>
              <option value="high">High</option>
            </select>
            <label class="color-#666 dark:color-#aaa">Level:</label>
            <select v-model="options.target_level" class="p-5px-10px border border-#ddd rounded-4px dark:bg-#444 dark:border-#555 dark:color-#f6f6f6">
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
        <div class="mb-10px last:mb-0 p-10px bg-#f9f9f9 rounded-4px dark:bg-#333">
          <label class="flex items-center gap-8px cursor-pointer font-500">
            <input type="checkbox" v-model="options.convert_h265_to_h264" class="w-18px h-18px" />
            H.265 转 H.264 (视频)
          </label>
        </div>

        <!-- 视频格式转换 -->
        <div class="mb-10px last:mb-0 p-10px bg-#f9f9f9 rounded-4px dark:bg-#333">
          <label class="flex items-center gap-8px cursor-pointer font-500">
            <input type="checkbox" v-model="options.convert_format" class="w-18px h-18px" />
            格式转换 (视频)
          </label>
          <div v-if="options.convert_format" class="mt-10px pl-26px flex items-center gap-10px flex-wrap">
            <label class="color-#666 dark:color-#aaa">目标格式:</label>
            <select v-model="options.target_format" class="p-5px-10px border border-#ddd rounded-4px dark:bg-#444 dark:border-#555 dark:color-#f6f6f6">
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

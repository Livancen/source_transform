<script setup lang="ts">
import type { ProcessOptions } from "../types";

defineProps<{
  open: boolean;
  options: ProcessOptions;
  enableRatioCrop: boolean;
  ratios: string[];
  newRatio: string;
  ratioError: string;
  videoFilesLength: number;
}>();

const emit = defineEmits<{
  openCropPreview: [];
  "update:enableRatioCrop": [value: boolean];
  "update:newRatio": [value: string];
  addRatio: [];
  removeRatio: [index: number];
}>();
</script>

<template>
  <div class="options-strip-body shrink-0 bg-bg1 border-b border-border" :class="{ 'is-open': open }">
    <div class="pt-12px px-14px pb-14px flex flex-col gap-10px">
      <div class="flex flex-wrap gap-8px items-start">
        <!-- 压缩 -->
        <div class="opt-chip" :class="{ 'opt-chip-on': options.compress }">
          <label class="switch">
            <input type="checkbox" v-model="options.compress" />
            <span class="slider"></span>
          </label>
          <span class="text-12px font-500" :class="options.compress ? 'color-t1' : 'color-t2'">压缩</span>
          <span
            v-show="options.compress"
            class="inline-flex flex-wrap items-center gap-6px pl-6px ml-2px border-l border-border"
          >
            <span class="text-10px color-t3">质量</span>
            <input class="field w-64px! h-24px! px-6px! text-11px!" type="number" v-model.number="options.compress_quality" min="1" max="100" />
            <label class="inline-flex items-center gap-4px text-11px color-t2 cursor-pointer whitespace-nowrap">
              <input type="radio" :value="false" v-model="options.compress_resize" class="accent-accent" />
              仅质量
            </label>
            <label class="inline-flex items-center gap-4px text-11px color-t2 cursor-pointer whitespace-nowrap">
              <input type="radio" :value="true" v-model="options.compress_resize" class="accent-accent" />
              +降分辨率
            </label>
            <template v-if="options.compress_resize">
              <input class="field w-64px! h-24px! px-6px! text-11px!" type="number" v-model.number="options.compress_width" min="1" title="宽" />
              <span class="text-10px color-t3">×</span>
              <input class="field w-64px! h-24px! px-6px! text-11px!" type="number" v-model.number="options.compress_height" min="1" title="高" />
            </template>
          </span>
        </div>

        <!-- 降分辨率 -->
        <div class="opt-chip" :class="{ 'opt-chip-on': options.reduce_resolution }">
          <label class="switch">
            <input type="checkbox" v-model="options.reduce_resolution" />
            <span class="slider"></span>
          </label>
          <span class="text-12px font-500" :class="options.reduce_resolution ? 'color-t1' : 'color-t2'">降分辨率</span>
          <span v-show="options.reduce_resolution" class="inline-flex items-center gap-6px pl-6px ml-2px border-l border-border">
            <input class="field w-64px! h-24px! px-6px! text-11px!" type="number" v-model.number="options.target_width" min="1" title="宽" />
            <span class="text-10px color-t3">×</span>
            <input class="field w-64px! h-24px! px-6px! text-11px!" type="number" v-model.number="options.target_height" min="1" title="高" />
          </span>
        </div>

        <!-- 降码率 -->
        <div class="opt-chip" :class="{ 'opt-chip-on': options.reduce_bitrate }">
          <label class="switch">
            <input type="checkbox" v-model="options.reduce_bitrate" />
            <span class="slider"></span>
          </label>
          <span class="text-12px font-500" :class="options.reduce_bitrate ? 'color-t1' : 'color-t2'">降码率</span>
          <span v-show="options.reduce_bitrate" class="inline-flex items-center gap-6px pl-6px ml-2px border-l border-border">
            <select class="field w-auto! min-w-72px h-24px! px-6px! text-11px!" v-model="options.target_bitrate">
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
          </span>
        </div>

        <!-- Profile/Level -->
        <div class="opt-chip" :class="{ 'opt-chip-on': options.reduce_level }">
          <label class="switch">
            <input type="checkbox" v-model="options.reduce_level" />
            <span class="slider"></span>
          </label>
          <span class="text-12px font-500" :class="options.reduce_level ? 'color-t1' : 'color-t2'">Profile/Level</span>
          <span v-show="options.reduce_level" class="inline-flex items-center gap-6px pl-6px ml-2px border-l border-border">
            <select class="field w-auto! min-w-72px h-24px! px-6px! text-11px!" v-model="options.target_profile">
              <option value="baseline">Baseline</option>
              <option value="main">Main</option>
              <option value="high">High</option>
            </select>
            <select class="field w-auto! min-w-72px h-24px! px-6px! text-11px!" v-model="options.target_level">
              <option value="3.0">3.0</option>
              <option value="3.1">3.1</option>
              <option value="4.0">4.0</option>
              <option value="4.1">4.1</option>
              <option value="4.2">4.2</option>
              <option value="5.0">5.0</option>
              <option value="5.1">5.1</option>
              <option value="5.2">5.2</option>
            </select>
          </span>
        </div>

        <!-- H265 -->
        <div class="opt-chip" :class="{ 'opt-chip-on': options.convert_h265_to_h264 }">
          <label class="switch">
            <input type="checkbox" v-model="options.convert_h265_to_h264" />
            <span class="slider"></span>
          </label>
          <span class="text-12px font-500" :class="options.convert_h265_to_h264 ? 'color-t1' : 'color-t2'">H.265→H.264</span>
        </div>

        <!-- 格式 -->
        <div class="opt-chip" :class="{ 'opt-chip-on': options.convert_format }">
          <label class="switch">
            <input type="checkbox" v-model="options.convert_format" />
            <span class="slider"></span>
          </label>
          <span class="text-12px font-500" :class="options.convert_format ? 'color-t1' : 'color-t2'">转格式</span>
          <span v-show="options.convert_format" class="inline-flex items-center gap-6px pl-6px ml-2px border-l border-border">
            <select class="field w-auto! min-w-72px h-24px! px-6px! text-11px!" v-model="options.target_format">
              <option value="mp4">MP4</option>
              <option value="avi">AVI</option>
              <option value="mkv">MKV</option>
              <option value="mov">MOV</option>
              <option value="webm">WebM</option>
              <option value="flv">FLV</option>
            </select>
          </span>
        </div>

        <!-- 裁剪 -->
        <div class="opt-chip" :class="{ 'opt-chip-on': options.crop }">
          <label class="switch">
            <input type="checkbox" v-model="options.crop" />
            <span class="slider"></span>
          </label>
          <span class="text-12px font-500" :class="options.crop ? 'color-t1' : 'color-t2'">裁剪</span>
          <span v-show="options.crop" class="inline-flex items-center gap-6px pl-6px ml-2px border-l border-border">
            <span class="text-10px color-t3">{{ options.crop_width }}×{{ options.crop_height }}</span>
            <span class="text-10px color-t3">@{{ options.crop_x }},{{ options.crop_y }}</span>
            <button
              class="tb-btn h-24px! px-8px! text-11px!"
              type="button"
              :disabled="videoFilesLength === 0"
              @click="emit('openCropPreview')"
            >
              设置区域
            </button>
          </span>
        </div>

        <!-- 旋转 -->
        <div class="opt-chip" :class="{ 'opt-chip-on': options.rotate }">
          <label class="switch">
            <input type="checkbox" v-model="options.rotate" />
            <span class="slider"></span>
          </label>
          <span class="text-12px font-500" :class="options.rotate ? 'color-t1' : 'color-t2'">旋转</span>
          <span v-show="options.rotate" class="inline-flex items-center gap-6px pl-6px ml-2px border-l border-border">
            <select class="field w-auto! min-w-72px h-24px! px-6px! text-11px!" v-model.number="options.rotation_degrees">
              <option :value="90">顺时针 90°</option>
              <option :value="180">180°</option>
              <option :value="270">顺时针 270°</option>
              <option :value="-90">逆时针 90°</option>
            </select>
          </span>
        </div>

        <!-- 静音 -->
        <div class="opt-chip" :class="{ 'opt-chip-on': options.mute }">
          <label class="switch">
            <input type="checkbox" v-model="options.mute" />
            <span class="slider"></span>
          </label>
          <span class="text-12px font-500" :class="options.mute ? 'color-t1' : 'color-t2'">静音</span>
        </div>

        <!-- 帧率 -->
        <div class="opt-chip" :class="{ 'opt-chip-on': options.change_framerate }">
          <label class="switch">
            <input type="checkbox" v-model="options.change_framerate" />
            <span class="slider"></span>
          </label>
          <span class="text-12px font-500" :class="options.change_framerate ? 'color-t1' : 'color-t2'">帧率</span>
          <span v-show="options.change_framerate" class="inline-flex items-center gap-6px pl-6px ml-2px border-l border-border">
            <input class="field w-48px! h-24px! px-6px! text-11px!" type="number" v-model.number="options.target_framerate" min="1" max="120" step="1" />
            <span class="text-10px color-t3">fps</span>
          </span>
        </div>
      </div>

      <!-- 比例裁剪 -->
      <div class="flex flex-wrap items-center gap-10px py-10px px-12px rounded-8px bg-bg0 border border-dashed border-border">
        <div class="flex items-center gap-8px font-500 text-12px">
          <label class="switch">
            <input
              type="checkbox"
              :checked="enableRatioCrop"
              @change="emit('update:enableRatioCrop', ($event.target as HTMLInputElement).checked)"
            />
            <span class="slider"></span>
          </label>
          比例裁剪
        </div>
        <div class="flex flex-wrap gap-6px flex-1">
          <span
            v-for="(r, i) in ratios"
            :key="r"
            class="inline-flex items-center gap-6px h-28px px-10px rounded-full bg-accent-soft border border-accent/20 color-accent text-12px font-500 font-mono"
          >
            {{ r }}
            <button
              type="button"
              class="border-none bg-transparent color-t3 cursor-pointer text-14px leading-none p-0 hover:color-danger"
              @click="emit('removeRatio', i)"
            >×</button>
          </span>
          <span v-if="ratios.length === 0" class="text-11px color-t3">暂无比例，请添加</span>
        </div>
        <div class="flex gap-6px items-center">
          <input
            class="h-28px w-88px px-10px rounded-full border border-border bg-bg1 color-t1 text-12px font-mono outline-none focus:border-accent"
            :value="newRatio"
            type="text"
            placeholder="W:H"
            @input="emit('update:newRatio', ($event.target as HTMLInputElement).value)"
            @keyup.enter="emit('addRatio')"
          />
          <button
            class="w-28px h-28px p-0 rounded-full border-none bg-accent color-white text-16px grid place-items-center cursor-pointer hover:brightness-110"
            type="button"
            @click="emit('addRatio')"
          >+</button>
        </div>
        <p v-if="ratioError" class="text-11px color-danger w-full">{{ ratioError }}</p>
      </div>
    </div>
  </div>
</template>

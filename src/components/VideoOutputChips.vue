<script setup lang="ts">
import type { RatioVideoOutput } from "../types";

defineProps<{
  video: RatioVideoOutput;
}>();
</script>

<template>
  <div class="flex flex-wrap gap-8px items-center">
    <span class="text-11px font-500 color-t3 shrink-0 w-36px">视频</span>
    <div class="opt-chip" :class="{ 'opt-chip-on': video.compress }">
      <label class="switch">
        <input type="checkbox" v-model="video.compress" />
        <span class="slider"></span>
      </label>
      <span
        class="text-12px font-500"
        :class="video.compress ? 'color-t1' : 'color-t2'"
        >压缩</span
      >
      <span
        v-show="video.compress"
        class="inline-flex items-center gap-6px pl-6px ml-2px border-l border-border"
      >
        <span class="text-10px color-t3">质量</span>
        <input
          class="field w-64px! h-24px! px-6px! text-11px!"
          type="number"
          v-model.number="video.compress_quality"
          min="1"
          max="100"
        />
      </span>
    </div>

    <div class="opt-chip" :class="{ 'opt-chip-on': video.convert_mp4 }">
      <label class="switch">
        <input type="checkbox" v-model="video.convert_mp4" />
        <span class="slider"></span>
      </label>
      <span
        class="text-12px font-500"
        :class="video.convert_mp4 ? 'color-t1' : 'color-t2'"
        >转 MP4</span
      >
    </div>

    <div class="opt-chip" :class="{ 'opt-chip-on': video.resize }">
      <label class="switch">
        <input type="checkbox" v-model="video.resize" />
        <span class="slider"></span>
      </label>
      <span
        class="text-12px font-500"
        :class="video.resize ? 'color-t1' : 'color-t2'"
        >分辨率</span
      >
      <span
        v-show="video.resize"
        class="inline-flex items-center gap-6px pl-6px ml-2px border-l border-border"
      >
        <input
          class="field w-64px! h-24px! px-6px! text-11px!"
          type="number"
          v-model.number="video.target_width"
          min="1"
          title="宽"
        />
        <span class="text-10px color-t3">×</span>
        <input
          class="field w-64px! h-24px! px-6px! text-11px!"
          type="number"
          v-model.number="video.target_height"
          min="1"
          title="高"
        />
      </span>
    </div>

    <div class="opt-chip" :class="{ 'opt-chip-on': video.convert_h265_to_h264 }">
      <label class="switch">
        <input type="checkbox" v-model="video.convert_h265_to_h264" />
        <span class="slider"></span>
      </label>
      <span
        class="text-12px font-500"
        :class="video.convert_h265_to_h264 ? 'color-t1' : 'color-t2'"
        >H.265→H.264</span
      >
    </div>

    <div class="opt-chip" :class="{ 'opt-chip-on': video.set_level }">
      <label class="switch">
        <input type="checkbox" v-model="video.set_level" />
        <span class="slider"></span>
      </label>
      <span
        class="text-12px font-500"
        :class="video.set_level ? 'color-t1' : 'color-t2'"
        >Profile/Level</span
      >
      <span
        v-show="video.set_level"
        class="inline-flex items-center gap-6px pl-6px ml-2px border-l border-border"
      >
        <select
          class="field w-auto! min-w-72px h-24px! px-6px! text-11px!"
          v-model="video.target_profile"
        >
          <option value="baseline">Baseline</option>
          <option value="main">Main</option>
          <option value="high">High</option>
        </select>
        <select
          class="field w-auto! min-w-72px h-24px! px-6px! text-11px!"
          v-model="video.target_level"
        >
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

    <div class="opt-chip" :class="{ 'opt-chip-on': video.reduce_bitrate }">
      <label class="switch">
        <input type="checkbox" v-model="video.reduce_bitrate" />
        <span class="slider"></span>
      </label>
      <span
        class="text-12px font-500"
        :class="video.reduce_bitrate ? 'color-t1' : 'color-t2'"
        >码率</span
      >
      <span
        v-show="video.reduce_bitrate"
        class="inline-flex items-center gap-6px pl-6px ml-2px border-l border-border"
      >
        <select
          class="field w-auto! min-w-72px h-24px! px-6px! text-11px!"
          v-model="video.target_bitrate"
        >
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

    <div class="opt-chip" :class="{ 'opt-chip-on': video.mute }">
      <label class="switch">
        <input type="checkbox" v-model="video.mute" />
        <span class="slider"></span>
      </label>
      <span
        class="text-12px font-500"
        :class="video.mute ? 'color-t1' : 'color-t2'"
        >静音</span
      >
    </div>

    <div class="opt-chip" :class="{ 'opt-chip-on': video.change_framerate }">
      <label class="switch">
        <input type="checkbox" v-model="video.change_framerate" />
        <span class="slider"></span>
      </label>
      <span
        class="text-12px font-500"
        :class="video.change_framerate ? 'color-t1' : 'color-t2'"
        >帧率</span
      >
      <span
        v-show="video.change_framerate"
        class="inline-flex items-center gap-6px pl-6px ml-2px border-l border-border"
      >
        <input
          class="field w-48px! h-24px! px-6px! text-11px!"
          type="number"
          v-model.number="video.target_framerate"
          min="1"
          max="120"
          step="1"
        />
        <span class="text-10px color-t3">fps</span>
      </span>
    </div>
  </div>
</template>

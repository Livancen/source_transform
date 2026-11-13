<script setup lang="ts">
import type { RatioImageOutput } from "../types";

defineProps<{
  image: RatioImageOutput;
}>();
</script>

<template>
  <div class="flex flex-wrap gap-8px items-center">
    <span class="text-11px font-500 color-t3 shrink-0 w-36px">图片</span>
    <div class="opt-chip" :class="{ 'opt-chip-on': image.compress }">
      <label class="switch">
        <input type="checkbox" v-model="image.compress" />
        <span class="slider"></span>
      </label>
      <span
        class="text-12px font-500"
        :class="image.compress ? 'color-t1' : 'color-t2'"
        >压缩</span
      >
      <span
        v-show="image.compress"
        class="inline-flex items-center gap-6px pl-6px ml-2px border-l border-border"
      >
        <span class="text-10px color-t3">质量</span>
        <input
          class="field w-64px! h-24px! px-6px! text-11px!"
          type="number"
          v-model.number="image.compress_quality"
          min="1"
          max="100"
        />
      </span>
    </div>

    <div class="opt-chip" :class="{ 'opt-chip-on': image.convert_format }">
      <label class="switch">
        <input type="checkbox" v-model="image.convert_format" />
        <span class="slider"></span>
      </label>
      <span
        class="text-12px font-500"
        :class="image.convert_format ? 'color-t1' : 'color-t2'"
        >转格式</span
      >
      <span
        v-show="image.convert_format"
        class="inline-flex items-center gap-6px pl-6px ml-2px border-l border-border"
      >
        <select
          class="field w-auto! min-w-72px h-24px! px-6px! text-11px!"
          v-model="image.target_format"
        >
          <option value="jpg">JPG</option>
          <option value="png">PNG</option>
          <option value="webp">WebP</option>
          <option value="bmp">BMP</option>
          <option value="tiff">TIFF</option>
        </select>
      </span>
    </div>

    <div class="opt-chip" :class="{ 'opt-chip-on': image.resize }">
      <label class="switch">
        <input type="checkbox" v-model="image.resize" />
        <span class="slider"></span>
      </label>
      <span
        class="text-12px font-500"
        :class="image.resize ? 'color-t1' : 'color-t2'"
        >分辨率</span
      >
      <span
        v-show="image.resize"
        class="inline-flex items-center gap-6px pl-6px ml-2px border-l border-border"
      >
        <input
          class="field w-64px! h-24px! px-6px! text-11px!"
          type="number"
          v-model.number="image.target_width"
          min="1"
          title="宽"
        />
        <span class="text-10px color-t3">×</span>
        <input
          class="field w-64px! h-24px! px-6px! text-11px!"
          type="number"
          v-model.number="image.target_height"
          min="1"
          title="高"
        />
      </span>
    </div>
  </div>
</template>

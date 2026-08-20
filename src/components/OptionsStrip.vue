<script setup lang="ts">
import { computed, ref, watch } from "vue";
import type { ProcessOptions, WorkMode } from "../types";
import { watermarkSummary } from "../types";
import WatermarkPopover from "./WatermarkPopover.vue";
import type { WatermarkDraft } from "./WatermarkPopover.vue";

const props = defineProps<{
  open: boolean;
  workMode: WorkMode;
  options: ProcessOptions;
  ratios: string[];
  newRatio: string;
  ratioError: string;
}>();

const emit = defineEmits<{
  "update:newRatio": [value: string];
  addRatio: [];
  removeRatio: [index: number];
}>();

const showImageOpts = () => props.workMode === "image";
const showVideoOpts = () => props.workMode === "video";
const showRatioOpts = () => props.workMode === "ratio";
const showBatchOpts = () => showImageOpts() || showVideoOpts();

const watermarkVisible = ref(false);
const watermarkChipRef = ref<HTMLElement | null>(null);
const watermarkHint = computed(() => watermarkSummary(props.options));

watch(
  () => [props.open, props.workMode] as const,
  () => {
    if (!props.open || !showBatchOpts()) {
      watermarkVisible.value = false;
    }
  },
);

function openWatermark() {
  if (!showBatchOpts()) return;
  watermarkVisible.value = true;
}

function applyWatermarkDraft(draft: WatermarkDraft) {
  const o = props.options;
  o.watermark = true;
  o.watermark_type = draft.watermark_type;
  o.watermark_text = draft.watermark_text;
  o.watermark_font_size = draft.watermark_font_size;
  o.watermark_font_color = draft.watermark_font_color;
  o.watermark_font_opacity = draft.watermark_font_opacity;
  o.watermark_stroke = draft.watermark_stroke;
  o.watermark_stroke_width = draft.watermark_stroke_width;
  o.watermark_stroke_color = draft.watermark_stroke_color;
  o.watermark_image_path = draft.watermark_image_path;
  o.watermark_image_scale = draft.watermark_image_scale;
  o.watermark_image_opacity = draft.watermark_image_opacity;
  o.watermark_position = draft.watermark_position;
  o.watermark_margin_x = draft.watermark_margin_x;
  o.watermark_margin_y = draft.watermark_margin_y;
  o.watermark_rotation = draft.watermark_rotation;
  o.watermark_tile = draft.watermark_tile;
  o.watermark_tile_gap_x = draft.watermark_tile_gap_x;
  o.watermark_tile_gap_y = draft.watermark_tile_gap_y;
  watermarkVisible.value = false;
}
</script>

<template>
  <div
    v-if="showBatchOpts() || showRatioOpts()"
    class="options-strip-body shrink-0 bg-bg1 border-b border-border"
    :class="{ 'is-open': open }"
  >
    <div class="py-2px px-12px flex flex-col gap-10px">
      <!-- 图片 / 视频 批量选项 -->
      <div v-if="showBatchOpts()" class="flex flex-wrap gap-8px items-start">
        <!-- 压缩 -->
        <div class="opt-chip" :class="{ 'opt-chip-on': options.compress }">
          <label class="switch">
            <input type="checkbox" v-model="options.compress" />
            <span class="slider"></span>
          </label>
          <span
            class="text-12px font-500"
            :class="options.compress ? 'color-t1' : 'color-t2'"
            >压缩</span
          >
          <span
            v-show="options.compress"
            class="inline-flex flex-wrap items-center gap-6px pl-6px ml-2px border-l border-border"
          >
            <span class="text-10px color-t3">质量</span>
            <input
              class="field w-64px! h-24px! px-6px! text-11px!"
              type="number"
              v-model.number="options.compress_quality"
              min="1"
              max="100"
            />
            <label
              class="inline-flex items-center gap-4px text-11px color-t2 cursor-pointer whitespace-nowrap"
            >
              <input
                type="radio"
                :value="false"
                v-model="options.compress_resize"
                class="accent-secondary"
              />
              仅质量
            </label>
            <label
              class="inline-flex items-center gap-4px text-11px color-t2 cursor-pointer whitespace-nowrap"
            >
              <input
                type="radio"
                :value="true"
                v-model="options.compress_resize"
                class="accent-secondary"
              />
              +降分辨率
            </label>
            <template v-if="options.compress_resize">
              <input
                class="field w-64px! h-24px! px-6px! text-11px!"
                type="number"
                v-model.number="options.compress_width"
                min="1"
                title="宽"
              />
              <span class="text-10px color-t3">×</span>
              <input
                class="field w-64px! h-24px! px-6px! text-11px!"
                type="number"
                v-model.number="options.compress_height"
                min="1"
                title="高"
              />
            </template>
          </span>
        </div>

        <!-- 降分辨率 -->
        <div
          class="opt-chip"
          :class="{ 'opt-chip-on': options.reduce_resolution }"
        >
          <label class="switch">
            <input type="checkbox" v-model="options.reduce_resolution" />
            <span class="slider"></span>
          </label>
          <span
            class="text-12px font-500"
            :class="options.reduce_resolution ? 'color-t1' : 'color-t2'"
            >降分辨率</span
          >
          <span
            v-show="options.reduce_resolution"
            class="inline-flex items-center gap-6px pl-6px ml-2px border-l border-border"
          >
            <input
              class="field w-64px! h-24px! px-6px! text-11px!"
              type="number"
              v-model.number="options.target_width"
              min="1"
              title="宽"
            />
            <span class="text-10px color-t3">×</span>
            <input
              class="field w-64px! h-24px! px-6px! text-11px!"
              type="number"
              v-model.number="options.target_height"
              min="1"
              title="高"
            />
          </span>
        </div>

        <!-- 转格式 -->
        <div
          class="opt-chip"
          :class="{ 'opt-chip-on': options.convert_format }"
        >
          <label class="switch">
            <input type="checkbox" v-model="options.convert_format" />
            <span class="slider"></span>
          </label>
          <span
            class="text-12px font-500"
            :class="options.convert_format ? 'color-t1' : 'color-t2'"
            >转格式</span
          >
          <span
            v-show="options.convert_format"
            class="inline-flex items-center gap-6px pl-6px ml-2px border-l border-border"
          >
            <select
              v-if="showImageOpts()"
              class="field w-auto! min-w-72px h-24px! px-6px! text-11px!"
              v-model="options.target_format"
            >
              <option value="jpg">JPG</option>
              <option value="png">PNG</option>
              <option value="webp">WebP</option>
              <option value="bmp">BMP</option>
              <option value="tiff">TIFF</option>
            </select>
            <select
              v-else
              class="field w-auto! min-w-72px h-24px! px-6px! text-11px!"
              v-model="options.target_format"
            >
              <option value="mp4">MP4</option>
              <option value="avi">AVI</option>
              <option value="mkv">MKV</option>
              <option value="mov">MOV</option>
              <option value="webm">WebM</option>
              <option value="flv">FLV</option>
            </select>
          </span>
        </div>

        <!-- 旋转 -->
        <div class="opt-chip" :class="{ 'opt-chip-on': options.rotate }">
          <label class="switch">
            <input type="checkbox" v-model="options.rotate" />
            <span class="slider"></span>
          </label>
          <span
            class="text-12px font-500"
            :class="options.rotate ? 'color-t1' : 'color-t2'"
            >旋转</span
          >
          <span
            v-show="options.rotate"
            class="inline-flex items-center gap-6px pl-6px ml-2px border-l border-border"
          >
            <select
              class="field w-auto! min-w-72px h-24px! px-6px! text-11px!"
              v-model.number="options.rotation_degrees"
            >
              <option :value="90">顺时针 90°</option>
              <option :value="180">180°</option>
              <option :value="270">顺时针 270°</option>
              <option :value="-90">逆时针 90°</option>
            </select>
          </span>
        </div>

        <!-- 仅视频 -->
        <template v-if="showVideoOpts()">
          <div
            class="opt-chip"
            :class="{ 'opt-chip-on': options.reduce_bitrate }"
          >
            <label class="switch">
              <input type="checkbox" v-model="options.reduce_bitrate" />
              <span class="slider"></span>
            </label>
            <span
              class="text-12px font-500"
              :class="options.reduce_bitrate ? 'color-t1' : 'color-t2'"
              >降码率</span
            >
            <span
              v-show="options.reduce_bitrate"
              class="inline-flex items-center gap-6px pl-6px ml-2px border-l border-border"
            >
              <select
                class="field w-auto! min-w-72px h-24px! px-6px! text-11px!"
                v-model="options.target_bitrate"
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

          <div
            class="opt-chip"
            :class="{ 'opt-chip-on': options.reduce_level }"
          >
            <label class="switch">
              <input type="checkbox" v-model="options.reduce_level" />
              <span class="slider"></span>
            </label>
            <span
              class="text-12px font-500"
              :class="options.reduce_level ? 'color-t1' : 'color-t2'"
              >Profile/Level</span
            >
            <span
              v-show="options.reduce_level"
              class="inline-flex items-center gap-6px pl-6px ml-2px border-l border-border"
            >
              <select
                class="field w-auto! min-w-72px h-24px! px-6px! text-11px!"
                v-model="options.target_profile"
              >
                <option value="baseline">Baseline</option>
                <option value="main">Main</option>
                <option value="high">High</option>
              </select>
              <select
                class="field w-auto! min-w-72px h-24px! px-6px! text-11px!"
                v-model="options.target_level"
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

          <div
            class="opt-chip"
            :class="{ 'opt-chip-on': options.convert_h265_to_h264 }"
          >
            <label class="switch">
              <input
                type="checkbox"
                v-model="options.convert_h265_to_h264"
                @change="
                  options.convert_h265_to_h264 &&
                  (options.convert_h264_to_h265 = false)
                "
              />
              <span class="slider"></span>
            </label>
            <span
              class="text-12px font-500"
              :class="options.convert_h265_to_h264 ? 'color-t1' : 'color-t2'"
              >H.265→H.264</span
            >
          </div>

          <div
            class="opt-chip"
            :class="{ 'opt-chip-on': options.convert_h264_to_h265 }"
          >
            <label class="switch">
              <input
                type="checkbox"
                v-model="options.convert_h264_to_h265"
                @change="
                  options.convert_h264_to_h265 &&
                  (options.convert_h265_to_h264 = false)
                "
              />
              <span class="slider"></span>
            </label>
            <span
              class="text-12px font-500"
              :class="options.convert_h264_to_h265 ? 'color-t1' : 'color-t2'"
              >H.264→H.265</span
            >
          </div>

          <div class="opt-chip" :class="{ 'opt-chip-on': options.mute }">
            <label class="switch">
              <input type="checkbox" v-model="options.mute" />
              <span class="slider"></span>
            </label>
            <span
              class="text-12px font-500"
              :class="options.mute ? 'color-t1' : 'color-t2'"
              >静音</span
            >
          </div>

          <div
            class="opt-chip"
            :class="{ 'opt-chip-on': options.change_framerate }"
          >
            <label class="switch">
              <input type="checkbox" v-model="options.change_framerate" />
              <span class="slider"></span>
            </label>
            <span
              class="text-12px font-500"
              :class="options.change_framerate ? 'color-t1' : 'color-t2'"
              >帧率</span
            >
            <span
              v-show="options.change_framerate"
              class="inline-flex items-center gap-6px pl-6px ml-2px border-l border-border"
            >
              <input
                class="field w-48px! h-24px! px-6px! text-11px!"
                type="number"
                v-model.number="options.target_framerate"
                min="1"
                max="120"
                step="1"
              />
              <span class="text-10px color-t3">fps</span>
            </span>
          </div>
        </template>

        <!-- 水印（图片 / 视频共用） -->
        <div
          ref="watermarkChipRef"
          class="opt-chip relative"
          :class="{ 'opt-chip-on': options.watermark }"
        >
          <label class="switch">
            <input type="checkbox" v-model="options.watermark" />
            <span class="slider"></span>
          </label>
          <span
            class="text-12px font-500"
            :class="options.watermark ? 'color-t1' : 'color-t2'"
            >水印</span
          >
          <span
            class="inline-flex items-center gap-6px pl-6px ml-2px border-l border-border"
          >
            <select
              class="field w-auto! min-w-64px h-24px! px-6px! text-11px!"
              v-model="options.watermark_type"
              :disabled="!options.watermark"
            >
              <option value="text">文字</option>
              <option value="image">图片</option>
            </select>
            <button
              class="tb-btn h-24px! px-8px! text-11px!"
              type="button"
              @click.stop="openWatermark"
            >
              配置…
            </button>
            <span
              v-if="options.watermark && watermarkHint"
              class="text-10px color-t3 max-w-140px truncate"
              :title="watermarkHint"
            >
              {{ watermarkHint }}
            </span>
          </span>
        </div>
      </div>

      <!-- 比例裁剪 -->
      <div
        v-if="showRatioOpts()"
        class="flex flex-wrap items-center gap-10px py-2px px-12px rounded-4px bg-bg0 border border-dashed border-border"
      >
        <div class="flex items-center gap-8px font-500 text-12px color-t1">
          比例列表
          <span class="text-11px color-t3 font-400"
            >（图+视频批量 · 专用命名）</span
          >
        </div>
        <div class="flex flex-wrap gap-6px flex-1">
          <span
            v-for="(r, i) in ratios"
            :key="r"
            class="inline-flex items-center gap-6px h-28px px-10px rounded-full bg-secondary-soft border border-secondary/20 color-secondary text-12px font-500 font-mono"
          >
            {{ r }}
            <button
              type="button"
              class="border-none bg-transparent color-t3 cursor-pointer text-14px leading-none p-0 hover:color-danger"
              @click="emit('removeRatio', i)"
            >
              ×
            </button>
          </span>
          <span v-if="ratios.length === 0" class="text-11px color-t3"
            >暂无比例，请添加</span
          >
        </div>
        <div class="flex gap-6px items-center">
          <input
            class="h-24px w-88px px-10px rounded-full border border-border bg-bg1 color-t1 text-12px font-mono outline-none focus:border-secondary"
            :value="newRatio"
            type="text"
            placeholder="W:H"
            @input="
              emit('update:newRatio', ($event.target as HTMLInputElement).value)
            "
            @keyup.enter="emit('addRatio')"
          />
          <button
            class="w-28px h-28px p-0 lh-28px rounded-full border-none bg-secondary color-white text-16px grid place-items-center cursor-pointer hover:bg-secondary-hover"
            type="button"
            @click="emit('addRatio')"
          >
            +
          </button>
        </div>
        <p v-if="ratioError" class="text-11px color-danger w-full">
          {{ ratioError }}
        </p>
      </div>
    </div>
  </div>

  <WatermarkPopover
    :visible="watermarkVisible"
    :options="options"
    :anchor-el="watermarkChipRef"
    @close="watermarkVisible = false"
    @apply="applyWatermarkDraft"
  />
</template>

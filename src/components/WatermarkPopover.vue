<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import type { ProcessOptions, WatermarkPosition, WatermarkType } from "../types";
import { WATERMARK_POSITIONS, defaultProcessOptions } from "../types";

export type WatermarkDraft = ReturnType<typeof cloneWatermark>;

const props = defineProps<{
  visible: boolean;
  options: ProcessOptions;
  anchorEl: HTMLElement | null;
}>();

const emit = defineEmits<{
  close: [];
  apply: [draft: WatermarkDraft];
}>();

const panelRef = ref<HTMLElement | null>(null);
const draft = ref(cloneWatermark(props.options));
const panelStyle = ref<Record<string, string>>({
  left: "0px",
  top: "0px",
  visibility: "hidden",
});

const isText = computed(() => draft.value.watermark_type === "text");

watch(
  () => props.visible,
  async (v) => {
    if (!v) {
      panelStyle.value = {
        left: "0px",
        top: "0px",
        visibility: "hidden",
      };
      return;
    }
    draft.value = cloneWatermark(props.options);
    panelStyle.value = {
      left: "0px",
      top: "0px",
      visibility: "hidden",
    };
    await nextTick();
    positionPanel();
    requestAnimationFrame(() => {
      positionPanel();
      panelStyle.value = {
        ...panelStyle.value,
        visibility: "visible",
      };
    });
  },
);

onMounted(() => {
  window.addEventListener("resize", onWinChange);
  window.addEventListener("scroll", onWinChange, true);
  document.addEventListener("keydown", onKeydown);
  document.addEventListener("pointerdown", onDocPointerDown, true);
});

onBeforeUnmount(() => {
  window.removeEventListener("resize", onWinChange);
  window.removeEventListener("scroll", onWinChange, true);
  document.removeEventListener("keydown", onKeydown);
  document.removeEventListener("pointerdown", onDocPointerDown, true);
});

function cloneWatermark(src: ProcessOptions) {
  return {
    watermark_type: src.watermark_type as WatermarkType,
    watermark_text: src.watermark_text,
    watermark_font_size: src.watermark_font_size,
    watermark_font_color: src.watermark_font_color,
    watermark_font_opacity: src.watermark_font_opacity,
    watermark_stroke: src.watermark_stroke,
    watermark_stroke_width: src.watermark_stroke_width,
    watermark_stroke_color: src.watermark_stroke_color,
    watermark_image_path: src.watermark_image_path,
    watermark_image_scale: src.watermark_image_scale,
    watermark_image_opacity: src.watermark_image_opacity,
    watermark_position: src.watermark_position as WatermarkPosition,
    watermark_margin_x: src.watermark_margin_x,
    watermark_margin_y: src.watermark_margin_y,
    watermark_rotation: normalizeRotation(src.watermark_rotation ?? 0),
    watermark_tile: src.watermark_tile,
    watermark_tile_gap_x: src.watermark_tile_gap_x,
    watermark_tile_gap_y: src.watermark_tile_gap_y,
  };
}

function normalizeRotation(deg: number) {
  if (!Number.isFinite(deg)) return 0;
  let v = Math.round(deg);
  while (v > 180) v -= 360;
  while (v < -180) v += 360;
  return v;
}

const ROTATION_PRESETS = [-45, -30, 0, 30, 45] as const;
const dialRef = ref<HTMLElement | null>(null);
const dialDragging = ref(false);

const previewStyle = computed(() => ({
  transform: `rotate(${draft.value.watermark_rotation || 0}deg)`,
  opacity: Math.max(
    0.15,
    (isText.value
      ? draft.value.watermark_font_opacity
      : draft.value.watermark_image_opacity) / 100,
  ),
  color: draft.value.watermark_font_color || "#fff",
}));

function setRotation(deg: number) {
  draft.value.watermark_rotation = normalizeRotation(deg);
}

function angleFromPointer(e: PointerEvent) {
  const el = dialRef.value;
  if (!el) return draft.value.watermark_rotation;
  const rect = el.getBoundingClientRect();
  const cx = rect.left + rect.width / 2;
  const cy = rect.top + rect.height / 2;
  // 0° = 向上（12 点），顺时针为正，与 CSS rotate 一致
  const rad = Math.atan2(e.clientX - cx, cy - e.clientY);
  return normalizeRotation((rad * 180) / Math.PI);
}

function onDialPointerDown(e: PointerEvent) {
  dialDragging.value = true;
  (e.currentTarget as HTMLElement).setPointerCapture?.(e.pointerId);
  setRotation(angleFromPointer(e));
}

function onDialPointerMove(e: PointerEvent) {
  if (!dialDragging.value) return;
  setRotation(angleFromPointer(e));
}

function onDialPointerUp(e: PointerEvent) {
  dialDragging.value = false;
  try {
    (e.currentTarget as HTMLElement).releasePointerCapture?.(e.pointerId);
  } catch {
    /* ignore */
  }
}

function resetDraft() {
  const def = defaultProcessOptions();
  draft.value = cloneWatermark(def);
  draft.value.watermark_type = props.options.watermark_type;
}

function setType(t: WatermarkType) {
  draft.value.watermark_type = t;
}

function setPosition(p: WatermarkPosition) {
  draft.value.watermark_position = p;
}

async function pickImage() {
  const selected = await openDialog({
    multiple: false,
    filters: [
      {
        name: "图片",
        extensions: ["png", "jpg", "jpeg", "webp", "bmp", "gif"],
      },
    ],
  });
  if (typeof selected === "string") {
    draft.value.watermark_image_path = selected;
  }
}

function imageName(path: string) {
  if (!path) return "未选择";
  const parts = path.replace(/\\/g, "/").split("/");
  return parts[parts.length - 1] || path;
}

function onWinChange() {
  if (props.visible) positionPanel();
}

function positionPanel() {
  const anchor = props.anchorEl;
  const panel = panelRef.value;
  if (!props.visible || !panel || !anchor) return;

  const rect = anchor.getBoundingClientRect();
  const pw = panel.offsetWidth || 360;
  const ph = panel.offsetHeight || 420;
  const gap = 6;
  let left = rect.left;
  let top = rect.bottom + gap;

  if (left + pw > window.innerWidth - 8) {
    left = Math.max(8, window.innerWidth - pw - 8);
  }
  if (left < 8) left = 8;

  if (top + ph > window.innerHeight - 8) {
    top = Math.max(8, rect.top - ph - gap);
  }
  if (top < 8) top = 8;

  panelStyle.value = {
    left: `${left}px`,
    top: `${top}px`,
    visibility: panelStyle.value.visibility || "hidden",
  };
}

function onKeydown(e: KeyboardEvent) {
  if (props.visible && e.key === "Escape") emit("close");
}

function onDocPointerDown(e: PointerEvent) {
  if (!props.visible) return;
  const path = typeof e.composedPath === "function" ? e.composedPath() : [];
  if (panelRef.value && path.includes(panelRef.value)) return;
  if (props.anchorEl && path.includes(props.anchorEl)) return;
  emit("close");
}

function onDone() {
  emit("apply", { ...draft.value });
}
</script>

<template>
  <Teleport to="body">
    <div
      v-if="visible"
      ref="panelRef"
      class="fixed z-3000 w-400px max-w-[calc(100vw-16px)] bg-bg1 rounded-10px border border-border shadow-xl overflow-hidden"
      :style="panelStyle"
      @pointerdown.stop
      @click.stop
    >
      <div
        class="px-14px py-10px border-b border-border flex items-center justify-between gap-10px"
      >
        <div class="text-13px font-600 color-t1">水印设置</div>
        <button
          class="tb-btn text-16px! px-8px!"
          type="button"
          @click="emit('close')"
        >
          ×
        </button>
      </div>

      <div class="px-14px py-12px flex flex-col gap-12px max-h-70vh overflow-y-auto">
        <div class="flex items-center gap-8px">
          <span class="text-11px color-t3 w-36px shrink-0">类型</span>
          <label
            class="inline-flex items-center gap-4px text-12px color-t2 cursor-pointer"
          >
            <input
              type="radio"
              class="accent-secondary"
              :checked="isText"
              @change="setType('text')"
            />
            文字
          </label>
          <label
            class="inline-flex items-center gap-4px text-12px color-t2 cursor-pointer"
          >
            <input
              type="radio"
              class="accent-secondary"
              :checked="!isText"
              @change="setType('image')"
            />
            图片
          </label>
        </div>

        <template v-if="isText">
          <div class="flex items-center gap-8px">
            <span class="text-11px color-t3 w-36px shrink-0">文案</span>
            <input
              class="field flex-1 min-w-0 h-28px! text-12px!"
              type="text"
              v-model="draft.watermark_text"
              placeholder="水印文字"
            />
          </div>
          <div class="flex items-center gap-8px flex-wrap">
            <span class="text-11px color-t3 w-36px shrink-0">字号</span>
            <input
              class="field w-72px! h-28px! text-12px!"
              type="number"
              v-model.number="draft.watermark_font_size"
              min="8"
              max="400"
            />
            <span class="text-11px color-t3">颜色</span>
            <input
              class="w-28px h-28px p-0 border border-border rounded-4px bg-bg0 cursor-pointer"
              type="color"
              v-model="draft.watermark_font_color"
            />
            <span class="text-11px color-t3">透明度</span>
            <input
              class="field w-64px! h-28px! text-12px!"
              type="number"
              v-model.number="draft.watermark_font_opacity"
              min="0"
              max="100"
            />
            <span class="text-10px color-t3">%</span>
          </div>
          <div class="flex items-center gap-8px flex-wrap">
            <span class="text-11px color-t3 w-36px shrink-0">描边</span>
            <label class="switch">
              <input type="checkbox" v-model="draft.watermark_stroke" />
              <span class="slider"></span>
            </label>
            <template v-if="draft.watermark_stroke">
              <input
                class="field w-56px! h-28px! text-12px!"
                type="number"
                v-model.number="draft.watermark_stroke_width"
                min="1"
                max="20"
                title="描边宽度"
              />
              <input
                class="w-28px h-28px p-0 border border-border rounded-4px bg-bg0 cursor-pointer"
                type="color"
                v-model="draft.watermark_stroke_color"
              />
            </template>
          </div>
        </template>

        <template v-else>
          <div class="flex items-center gap-8px min-w-0">
            <span class="text-11px color-t3 w-36px shrink-0">图片</span>
            <span
              class="flex-1 min-w-0 text-11px color-t2 font-mono truncate"
              :title="draft.watermark_image_path"
            >
              {{ imageName(draft.watermark_image_path) }}
            </span>
            <button
              class="tb-btn h-28px! px-8px! text-11px!"
              type="button"
              @click="pickImage"
            >
              选择…
            </button>
          </div>
          <div class="flex items-center gap-8px flex-wrap">
            <span class="text-11px color-t3 w-36px shrink-0">缩放</span>
            <input
              class="field w-72px! h-28px! text-12px!"
              type="number"
              v-model.number="draft.watermark_image_scale"
              min="1"
              max="100"
            />
            <span class="text-10px color-t3">% 宽</span>
            <span class="text-11px color-t3">透明度</span>
            <input
              class="field w-64px! h-28px! text-12px!"
              type="number"
              v-model.number="draft.watermark_image_opacity"
              min="0"
              max="100"
            />
            <span class="text-10px color-t3">%</span>
          </div>
        </template>

        <div class="border-t border-border pt-10px flex flex-col gap-8px">
          <div class="flex items-start gap-8px">
            <span class="text-11px color-t3 w-36px shrink-0 pt-6px">位置</span>
            <div class="grid grid-cols-3 gap-4px w-120px">
              <button
                v-for="p in WATERMARK_POSITIONS"
                :key="p.id"
                type="button"
                class="h-28px rounded-4px border text-10px cursor-pointer"
                :class="
                  draft.watermark_position === p.id
                    ? 'border-secondary bg-secondary-soft color-secondary'
                    : 'border-border bg-bg0 color-t2 hover:border-secondary/40'
                "
                :title="p.label"
                @click="setPosition(p.id)"
              >
                {{ p.label }}
              </button>
            </div>
          </div>
          <div class="flex items-center gap-8px flex-wrap">
            <span class="text-11px color-t3 w-36px shrink-0">边距</span>
            <span class="text-10px color-t3">X</span>
            <input
              class="field w-64px! h-28px! text-12px!"
              type="number"
              v-model.number="draft.watermark_margin_x"
              min="0"
              max="2000"
            />
            <span class="text-10px color-t3">Y</span>
            <input
              class="field w-64px! h-28px! text-12px!"
              type="number"
              v-model.number="draft.watermark_margin_y"
              min="0"
              max="2000"
            />
          </div>

          <!-- 角度：旋钮 + 预览 + 数值 -->
          <div class="flex items-start gap-10px">
            <span class="text-11px color-t3 w-36px shrink-0 pt-10px">角度</span>
            <div class="flex-1 min-w-0 flex flex-col gap-8px">
              <div class="flex items-center gap-12px">
                <div
                  ref="dialRef"
                  class="relative w-72px h-72px rounded-full border border-border bg-bg0 cursor-pointer select-none shrink-0"
                  :class="{ 'border-secondary': dialDragging }"
                  title="拖动调整角度"
                  @pointerdown.prevent="onDialPointerDown"
                  @pointermove="onDialPointerMove"
                  @pointerup="onDialPointerUp"
                  @pointercancel="onDialPointerUp"
                >
                  <div
                    class="absolute left-1/2 top-1/2 w-2px h-28px -ml-1px -mt-28px rounded-1px bg-secondary origin-bottom"
                    :style="{
                      transform: `rotate(${draft.watermark_rotation || 0}deg)`,
                    }"
                  ></div>
                  <div
                    class="absolute left-1/2 top-1/2 w-8px h-8px -ml-4px -mt-4px rounded-full bg-secondary"
                  ></div>
                </div>
                <div
                  class="flex-1 min-w-0 h-72px rounded-8px border border-border bg-[#2a2d34] grid place-items-center overflow-hidden"
                >
                  <div
                    class="px-10px py-4px text-13px font-600 whitespace-nowrap max-w-90% truncate"
                    :style="previewStyle"
                  >
                    {{
                      isText
                        ? draft.watermark_text || "水印"
                        : imageName(draft.watermark_image_path) || "图片"
                    }}
                  </div>
                </div>
              </div>
              <div class="flex items-center gap-8px flex-wrap">
                <input
                  class="flex-1 min-w-120px accent-secondary"
                  type="range"
                  min="-180"
                  max="180"
                  step="1"
                  :value="draft.watermark_rotation"
                  @input="
                    setRotation(
                      Number(($event.target as HTMLInputElement).value),
                    )
                  "
                />
                <input
                  class="field w-56px! h-28px! text-12px!"
                  type="number"
                  min="-180"
                  max="180"
                  :value="draft.watermark_rotation"
                  @change="
                    setRotation(
                      Number(($event.target as HTMLInputElement).value),
                    )
                  "
                />
                <span class="text-10px color-t3">°</span>
              </div>
              <div class="flex flex-wrap gap-4px">
                <button
                  v-for="p in ROTATION_PRESETS"
                  :key="p"
                  type="button"
                  class="h-24px px-8px rounded-4px border text-10px cursor-pointer"
                  :class="
                    draft.watermark_rotation === p
                      ? 'border-secondary bg-secondary-soft color-secondary'
                      : 'border-border bg-bg0 color-t2'
                  "
                  @click="setRotation(p)"
                >
                  {{ p }}°
                </button>
              </div>
            </div>
          </div>

          <div class="flex items-center gap-8px flex-wrap">
            <span class="text-11px color-t3 w-36px shrink-0">平铺</span>
            <label class="switch">
              <input type="checkbox" v-model="draft.watermark_tile" />
              <span class="slider"></span>
            </label>
            <template v-if="draft.watermark_tile">
              <span class="text-10px color-t3">间距 X</span>
              <input
                class="field w-56px! h-28px! text-12px!"
                type="number"
                v-model.number="draft.watermark_tile_gap_x"
                min="0"
                max="4000"
              />
              <span class="text-10px color-t3">Y</span>
              <input
                class="field w-56px! h-28px! text-12px!"
                type="number"
                v-model.number="draft.watermark_tile_gap_y"
                min="0"
                max="4000"
              />
            </template>
          </div>
        </div>
      </div>

      <div
        class="px-14px py-10px border-t border-border flex items-center justify-end gap-8px"
      >
        <button class="tb-btn h-30px! text-11px!" type="button" @click="resetDraft">
          重置
        </button>
        <button
          class="tb-btn tb-btn-success h-30px! text-11px!"
          type="button"
          @click="onDone"
        >
          完成
        </button>
      </div>
    </div>
  </Teleport>
</template>

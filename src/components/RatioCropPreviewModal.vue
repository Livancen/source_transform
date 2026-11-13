<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from "vue";
import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import type { FileInfo } from "../types";
import { computeCoverCrop, parseRatio } from "../utils/ratioCrop";

const props = defineProps<{
  visible: boolean;
  file: FileInfo | null;
  ratios: string[];
  offsetX: number;
  offsetY: number;
}>();

const emit = defineEmits<{
  close: [];
  "update:offsetX": [value: number];
  "update:offsetY": [value: number];
}>();

const previewUrl = ref("");
const mediaW = ref(0);
const mediaH = ref(0);
const loading = ref(false);
const error = ref("");
const activeIndex = ref(0);
const videoFailed = ref(false);
const framePending = ref(false);
const outputVideoRef = ref<HTMLVideoElement | null>(null);
const sourceVideoRef = ref<HTMLVideoElement | null>(null);

const isVideo = computed(() => props.file?.file_type === "video");

const activeRatio = computed(() => props.ratios[activeIndex.value] ?? "");

const targetRatio = computed(() =>
  activeRatio.value ? parseRatio(activeRatio.value) : null,
);

const crop = computed(() => {
  if (!mediaW.value || !mediaH.value || targetRatio.value == null) return null;
  return computeCoverCrop(
    mediaW.value,
    mediaH.value,
    targetRatio.value,
    props.offsetX || 0,
    props.offsetY || 0,
  );
});

const shiftStyle = computed(() => {
  const box = crop.value;
  if (!box || !mediaW.value || !mediaH.value) return {};
  return {
    width: `${(mediaW.value / box.cropW) * 100}%`,
    height: `${(mediaH.value / box.cropH) * 100}%`,
    transform: `translate(${(-box.cropX / mediaW.value) * 100}%, ${(-box.cropY / mediaH.value) * 100}%)`,
    transformOrigin: "top left",
  };
});

const outputFrameStyle = computed(() => {
  const box = crop.value;
  if (!box) return {};
  return {
    aspectRatio: `${box.cropW} / ${box.cropH}`,
    width: `min(100%, calc(62vh * ${box.cropW} / ${box.cropH}))`,
    maxHeight: "62vh",
  };
});

const cropBoxStyle = computed(() => {
  const box = crop.value;
  if (!box || !mediaW.value || !mediaH.value) return {};
  return {
    left: `${(box.cropX / mediaW.value) * 100}%`,
    top: `${(box.cropY / mediaH.value) * 100}%`,
    width: `${(box.cropW / mediaW.value) * 100}%`,
    height: `${(box.cropH / mediaH.value) * 100}%`,
  };
});

const shade = computed(() => {
  const box = crop.value;
  if (!box || !mediaW.value || !mediaH.value) return null;
  const x = (box.cropX / mediaW.value) * 100;
  const y = (box.cropY / mediaH.value) * 100;
  const w = (box.cropW / mediaW.value) * 100;
  const h = (box.cropH / mediaH.value) * 100;
  return {
    top: { height: `${y}%` },
    bottom: { top: `${y + h}%` },
    left: { top: `${y}%`, height: `${h}%`, width: `${x}%` },
    right: { top: `${y}%`, height: `${h}%`, left: `${x + w}%` },
  };
});

const offsetNote = computed(() => {
  const box = crop.value;
  if (!box) return "";
  const horizontal = mediaW.value - box.cropW > 1;
  const vertical = mediaH.value - box.cropH > 1;
  if (!horizontal && !vertical) return "与原比例一致，不会裁掉画面，偏移不生效";
  const parts: string[] = [];
  parts.push(
    horizontal
      ? `水平 ${offsetText("x", props.offsetX)}`
      : "水平已贴边",
  );
  parts.push(
    vertical ? `垂直 ${offsetText("y", props.offsetY)}` : "垂直已贴边",
  );
  return parts.join(" · ");
});

watch(
  () => props.ratios,
  (ratios) => {
    if (activeIndex.value >= ratios.length) activeIndex.value = 0;
  },
);

watch(
  () => [props.visible, props.file?.path] as const,
  async ([visible, path]) => {
    if (!visible || !path || !props.file) {
      resetMedia();
      return;
    }
    activeIndex.value = 0;
    await loadMedia(props.file);
  },
);

function resetMedia() {
  previewUrl.value = "";
  mediaW.value = 0;
  mediaH.value = 0;
  error.value = "";
  loading.value = false;
  videoFailed.value = false;
  framePending.value = false;
  pauseVideos();
}

function pauseVideos() {
  for (const video of [outputVideoRef.value, sourceVideoRef.value]) {
    if (!video) continue;
    video.pause();
    video.removeAttribute("src");
    video.load();
  }
}

async function loadMedia(file: FileInfo) {
  loading.value = true;
  error.value = "";
  previewUrl.value = "";
  mediaW.value = 0;
  mediaH.value = 0;
  videoFailed.value = false;
  framePending.value = false;
  try {
    if (file.file_type === "video") {
      const dimensions = await invoke<[number, number]>("get_video_dimensions", {
        videoPath: file.path,
      });
      mediaW.value = dimensions[0];
      mediaH.value = dimensions[1];
      previewUrl.value = convertFileSrc(file.path);
    } else {
      const [dimensions, url] = await Promise.all([
        invoke<[number, number]>("get_image_dimensions", {
          imagePath: file.path,
        }),
        invoke<string>("load_image_preview", { imagePath: file.path }),
      ]);
      mediaW.value = dimensions[0];
      mediaH.value = dimensions[1];
      previewUrl.value = url;
    }
    if (!mediaW.value || !mediaH.value) {
      throw new Error("无法读取尺寸");
    }
  } catch (e) {
    error.value = String(e);
  } finally {
    loading.value = false;
  }
}

function selectRatio(index: number) {
  activeIndex.value = index;
}

function stepRatio(delta: number) {
  if (props.ratios.length === 0) return;
  const next = (activeIndex.value + delta + props.ratios.length) % props.ratios.length;
  activeIndex.value = next;
}

function clampOffset(value: number) {
  if (!Number.isFinite(value)) return 0;
  return Math.min(100, Math.max(-100, Math.round(value)));
}

function offsetText(axis: "x" | "y", value: number) {
  const v = clampOffset(value);
  if (v === 0) return "居中";
  if (axis === "x") return v > 0 ? `偏右 ${v}%` : `偏左 ${-v}%`;
  return v > 0 ? `偏下 ${v}%` : `偏上 ${-v}%`;
}

function setOffset(axis: "x" | "y", value: number) {
  const next = clampOffset(value);
  if (axis === "x") emit("update:offsetX", next);
  else emit("update:offsetY", next);
}

async function onVideoError() {
  if (framePending.value || videoFailed.value || !props.file) return;
  framePending.value = true;
  try {
    previewUrl.value = await invoke<string>("extract_video_frame", {
      videoPath: props.file.path,
    });
    videoFailed.value = true;
  } catch (e) {
    error.value = `视频无法播放，且提取首帧失败: ${e}`;
  }
}

function onKey(e: KeyboardEvent) {
  if (!props.visible) return;
  if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) {
    if (e.key === "Escape") emit("close");
    return;
  }
  if (e.key === "Escape") {
    e.stopPropagation();
    emit("close");
  } else if (e.key === "ArrowLeft") {
    e.preventDefault();
    stepRatio(-1);
  } else if (e.key === "ArrowRight") {
    e.preventDefault();
    stepRatio(1);
  }
}

watch(
  () => props.visible,
  (visible) => {
    if (visible) window.addEventListener("keydown", onKey);
    else window.removeEventListener("keydown", onKey);
  },
  { immediate: true },
);

onBeforeUnmount(() => {
  window.removeEventListener("keydown", onKey);
});
</script>

<template>
  <div
    v-if="visible && file"
    class="modal-mask z-1600!"
    @click.self="emit('close')"
  >
    <div
      class="modal-panel w-980px! max-w-94vw! max-h-92vh! overflow-hidden! flex flex-col"
      @click.stop
    >
      <div
        class="shrink-0 h-48px px-16px flex items-center justify-between gap-12px border-b border-border bg-bg2"
      >
        <div class="min-w-0">
          <div class="text-13px font-600 truncate" :title="file.name">
            {{ file.name }}
          </div>
          <div class="text-11px color-t3 mt-2px">
            裁切预览
            <template v-if="mediaW && mediaH">
              · 源 {{ mediaW }}×{{ mediaH }}
            </template>
            <template v-if="isVideo && !videoFailed"> · 播放预览</template>
            <template v-else-if="isVideo"> · 首帧预览</template>
          </div>
        </div>
        <button class="tb-btn shrink-0 text-20px lh-20px" type="button" title="关闭" @click="emit('close')">
          ×
        </button>
      </div>

      <div class="shrink-0 px-16px py-10px border-b border-border flex flex-wrap items-center gap-8px">
        <span class="text-11px color-t3 shrink-0">比例</span>
        <button
          v-for="(ratio, index) in ratios"
          :key="ratio"
          type="button"
          class="h-26px px-10px rounded-full border text-12px font-mono font-500 cursor-pointer"
          :class="
            index === activeIndex
              ? 'bg-secondary color-white border-secondary'
              : 'bg-bg1 color-t1 border-border hover:border-secondary'
          "
          @click="selectRatio(index)"
        >
          {{ ratio }}
        </button>
        <span v-if="ratios.length === 0" class="text-12px color-warning">
          还没有比例，请先在选项条里添加
        </span>
        <span v-else class="text-10px color-t3 ml-auto">← → 切换</span>
      </div>

      <div class="flex-1 min-h-0 overflow-auto p-16px">
        <div v-if="loading" class="py-48px text-center text-13px color-t3">
          加载预览中…
        </div>
        <div
          v-else-if="error"
          class="py-48px text-center text-13px color-danger"
        >
          {{ error }}
        </div>
        <div
          v-else-if="previewUrl && !crop"
          class="py-48px text-center text-13px color-t3"
        >
          {{
            ratios.length
              ? "当前比例无效"
              : "添加比例后，这里会显示偏移后的输出区域"
          }}
        </div>
        <div
          v-else-if="previewUrl && crop && shade"
          class="preview-layout items-start"
        >
          <div class="min-w-0 flex flex-col gap-8px">
            <div class="text-12px font-600 color-t1">最终输出</div>
            <div class="rounded-4px bg-[#1b1b1b] p-12px flex items-center justify-center min-h-180px">
              <div class="overflow-hidden bg-black" :style="outputFrameStyle">
                <video
                  v-if="isVideo && !videoFailed"
                  ref="outputVideoRef"
                  :key="file.path"
                  :src="previewUrl"
                  class="block max-w-none object-fill pointer-events-none"
                  :style="shiftStyle"
                  muted
                  loop
                  autoplay
                  playsinline
                  @error="onVideoError"
                />
                <img
                  v-else
                  :src="previewUrl"
                  class="block max-w-none object-fill select-none"
                  :style="shiftStyle"
                  draggable="false"
                  :alt="file.name"
                />
              </div>
            </div>
            <div class="text-11px color-t3 font-mono">
              {{ activeRatio }} · {{ crop.cropW }}×{{ crop.cropH }} · 起点
              ({{ crop.cropX }}, {{ crop.cropY }})
            </div>
          </div>

          <div class="min-w-0 flex flex-col gap-8px">
            <div class="text-12px font-600 color-t1">原画面</div>
            <div
              class="relative overflow-hidden rounded-4px bg-black"
              :style="{ aspectRatio: `${mediaW} / ${mediaH}` }"
            >
              <video
                v-if="isVideo && !videoFailed"
                ref="sourceVideoRef"
                :key="`${file.path}-source`"
                :src="previewUrl"
                class="block w-full h-full object-fill pointer-events-none"
                muted
                loop
                autoplay
                playsinline
                @error="onVideoError"
              />
              <img
                v-else
                :src="previewUrl"
                class="block w-full h-full object-fill select-none"
                draggable="false"
                alt=""
              />
              <div class="absolute left-0 right-0 top-0 bg-black/55" :style="shade.top" />
              <div class="absolute left-0 right-0 bottom-0 bg-black/55" :style="shade.bottom" />
              <div class="absolute left-0 bg-black/55" :style="shade.left" />
              <div class="absolute right-0 bg-black/55" :style="shade.right" />
              <div
                class="absolute border-2 border-white pointer-events-none"
                :style="cropBoxStyle"
              />
            </div>
            <p class="text-11px color-t3 leading-relaxed">暗区会被裁掉，白框是导出区域。</p>
          </div>
        </div>
      </div>

      <div
        v-if="!loading && !error"
        class="shrink-0 px-16px py-12px border-t border-border bg-bg2 flex flex-col gap-8px"
      >
        <div class="text-11px color-t2">{{ offsetNote }}</div>
        <div class="flex flex-wrap items-center gap-x-16px gap-y-8px">
          <label class="inline-flex items-center gap-8px min-w-220px flex-1">
            <span class="text-11px color-t2 w-28px">水平</span>
            <input
              class="flex-1 min-w-80px accent-secondary"
              type="range"
              min="-100"
              max="100"
              step="1"
              :value="offsetX || 0"
              @input="setOffset('x', Number(($event.target as HTMLInputElement).value))"
            />
            <span class="text-11px color-t3 w-64px">{{ offsetText("x", offsetX) }}</span>
          </label>
          <label class="inline-flex items-center gap-8px min-w-220px flex-1">
            <span class="text-11px color-t2 w-28px">垂直</span>
            <input
              class="flex-1 min-w-80px accent-secondary"
              type="range"
              min="-100"
              max="100"
              step="1"
              :value="offsetY || 0"
              @input="setOffset('y', Number(($event.target as HTMLInputElement).value))"
            />
            <span class="text-11px color-t3 w-64px">{{ offsetText("y", offsetY) }}</span>
          </label>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.preview-layout {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 220px;
  gap: 16px;
}

@media (max-width: 760px) {
  .preview-layout {
    grid-template-columns: 1fr;
  }
}
</style>

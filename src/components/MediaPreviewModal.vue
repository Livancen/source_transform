<script setup lang="ts">
import { ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { FileInfo } from "../types";
import { formatFileSizeMb } from "../types";

const props = defineProps<{
  visible: boolean;
  file: FileInfo | null;
}>();

const emit = defineEmits<{
  close: [];
}>();

const previewUrl = ref("");
const loading = ref(false);
const error = ref("");

watch(
  () => [props.visible, props.file?.path] as const,
  async ([visible, path]) => {
    if (!visible || !path || !props.file) {
      previewUrl.value = "";
      error.value = "";
      loading.value = false;
      return;
    }
    loading.value = true;
    error.value = "";
    previewUrl.value = "";
    try {
      if (props.file.file_type === "video") {
        previewUrl.value = await invoke<string>("extract_video_frame", {
          videoPath: path,
        });
      } else {
        previewUrl.value = await invoke<string>("load_image_preview", {
          imagePath: path,
        });
      }
    } catch (e) {
      error.value = String(e);
    } finally {
      loading.value = false;
    }
  },
  { immediate: true },
);

function onMaskClick() {
  emit("close");
}
</script>

<template>
  <div
    v-if="visible && file"
    class="fixed inset-0 z-1000 flex items-center justify-center bg-black/55 p-20px"
    @click.self="onMaskClick"
  >
    <div
      class="relative bg-bg1 rounded-12px border border-border shadow-xl max-w-90vw max-h-90vh w-auto flex flex-col overflow-hidden"
      @click.stop
    >
      <div
        class="shrink-0 h-48px px-16px flex items-center justify-between gap-12px border-b border-border"
      >
        <div class="min-w-0 flex-1">
          <div class="text-13px font-600 truncate" :title="file.name">
            {{ file.name }}
          </div>
          <div class="text-11px color-t3 mt-2px">
            {{ file.file_type === "video" ? "视频" : "图片" }}
            · {{ formatFileSizeMb(file.size_bytes) }}
            <span v-if="file.file_type === 'video'" class="ml-4px"
              >（预览首帧）</span
            >
          </div>
        </div>
        <button
          class="tb-btn shrink-0 text-20px lh-20px"
          type="button"
          title="关闭"
          @click="emit('close')"
        >
          x
        </button>
      </div>

      <div
        class="flex-1 min-h-0 overflow-auto p-16px flex items-center justify-center bg-white min-h-200px max-h-[calc(90vh-48px)]"
      >
        <div v-if="loading" class="text-13px color-t3 py-40px">加载预览中…</div>
        <div
          v-else-if="error"
          class="text-13px color-danger py-40px px-20px text-center max-w-480px"
        >
          {{ error }}
        </div>
        <img
          v-else-if="previewUrl"
          :src="previewUrl"
          class="max-w-full max-h-[calc(90vh-100px)] object-contain select-none"
          draggable="false"
          :alt="file.name"
        />
      </div>
    </div>
  </div>
</template>

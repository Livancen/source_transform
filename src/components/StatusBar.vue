<script setup lang="ts">
defineProps<{
  inputCount: number;
  imageCount: number;
  videoCount: number;
  outputCount: number;
  message: string;
  uploadUrl: string;
}>();
</script>

<template>
  <footer
    class="shrink-0 h-28px flex items-center gap-16px px-12px bg-bg2 border-t border-border text-11px color-t3"
  >
    <span class="inline-flex items-center gap-6px min-w-0">
      version <strong class="color-t2 font-500">{{ 1234 }}</strong>
    </span>
    <span class="inline-flex items-center gap-6px min-w-0">
      输入 <strong class="color-t2 font-500">{{ inputCount }} 项</strong>
    </span>
    <span class="w-1px h-12px bg-border shrink-0"></span>
    <span class="inline-flex items-center gap-6px">
      图片 <strong class="color-t2 font-500">{{ imageCount }}</strong>
    </span>
    <span class="inline-flex items-center gap-6px">
      视频 <strong class="color-t2 font-500">{{ videoCount }}</strong>
    </span>
    <span class="w-1px h-12px bg-border shrink-0"></span>
    <span class="inline-flex items-center gap-6px">
      输出 <strong class="color-t2 font-500">{{ outputCount }} 项</strong>
    </span>
    <span class="w-1px h-12px bg-border shrink-0"></span>
    <span
      class="inline-flex items-center gap-6px min-w-0 truncate max-w-40vw color-t2"
      :title="message"
      >{{ message || "就绪" }}</span
    >
    <span class="flex-1 min-w-8px"></span>
    <span v-if="uploadUrl" class="inline-flex items-center gap-6px min-w-0">
      上传
      <a
        class="color-accent-hover font-mono text-11px no-underline truncate hover:underline"
        :href="uploadUrl"
        target="_blank"
        rel="noreferrer"
        >{{ uploadUrl }}</a
      >
    </span>
  </footer>
</template>

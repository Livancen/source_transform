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
  <footer class="statusbar">
    <span class="item">输入 <strong>{{ inputCount }} 项</strong></span>
    <span class="sep"></span>
    <span class="item">图片 <strong>{{ imageCount }}</strong></span>
    <span class="item">视频 <strong>{{ videoCount }}</strong></span>
    <span class="sep"></span>
    <span class="item">输出 <strong>{{ outputCount }} 项</strong></span>
    <span class="sep"></span>
    <span class="item msg" :title="message">{{ message || "就绪" }}</span>
    <span class="grow"></span>
    <span v-if="uploadUrl" class="item">
      上传
      <a class="upload" :href="uploadUrl" target="_blank" rel="noreferrer">{{ uploadUrl }}</a>
    </span>
  </footer>
</template>

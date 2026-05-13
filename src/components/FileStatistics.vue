<script setup lang="ts">
defineProps<{
  imageCount: number;
  videoCount: number;
  totalCount: number;
  isProcessing: boolean;
}>();

const emit = defineEmits<{
  refresh: [];
}>();
</script>

<template>
  <section class="section stats-section">
    <h2>文件统计</h2>
    <div class="stats">
      <div class="stat-item">
        <span class="stat-label">图片</span>
        <span class="stat-value">{{ imageCount }}</span>
      </div>
      <div class="stat-item">
        <span class="stat-label">视频</span>
        <span class="stat-value">{{ videoCount }}</span>
      </div>
      <div class="stat-row-last">
        <div class="stat-item stat-total">
          <span class="stat-label">总计</span>
          <span class="stat-value">{{ totalCount }}</span>
        </div>
        <button
          class="refresh-btn"
          @click="emit('refresh')"
          :disabled="isProcessing"
        >
          刷新
        </button>
      </div>
    </div>
  </section>
</template>

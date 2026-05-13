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
  <section
    class="w-200px shrink-0 mb-0 bg-white rounded-8px p-15px shadow-sm flex flex-col dark:bg-#2d2d2d"
  >
    <h2
      class="text-16px mb-10px color-#555 border-b border-b-#ddd border-b-solid pb-5px dark:color-#ccc dark:border-b-#444"
    >
      文件统计
    </h2>
    <div class="flex flex-col gap-8px flex-1">
      <div
        class="flex justify-between p-4px bg-#e9ecef rounded-4px dark:bg-#444"
      >
        <span class="color-#666 dark:color-#aaa">图片</span>
        <span class="font-600">{{ imageCount }}</span>
      </div>
      <div
        class="flex justify-between p-4px bg-#e9ecef rounded-4px dark:bg-#444"
      >
        <span class="color-#666 dark:color-#aaa">视频</span>
        <span class="font-600">{{ videoCount }}</span>
      </div>
      <div class="flex gap-8px">
        <div
          class="flex-1 flex justify-between p-4px bg-#e9ecef rounded-4px dark:bg-#444"
        >
          <span class="color-#666 dark:color-#aaa">总计</span>
          <span class="font-600">{{ totalCount }}</span>
        </div>
        <button
          class="flex-1 p-4px text-12px"
          @click="emit('refresh')"
          :disabled="isProcessing"
        >
          刷新
        </button>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import type { ProcessProgress } from "../types";

defineProps<{
  isProcessing: boolean;
  progress: ProcessProgress | null;
  resultMessage: string;
  filesLength: number;
}>();

const emit = defineEmits<{
  startProcess: [];
}>();
</script>

<template>
  <!-- 进度显示 -->
  <section
    class="bg-white rounded-8px p-15px shadow-sm dark:bg-#2d2d2d"
    v-if="isProcessing && progress"
  >
    <div class="flex flex-col items-center gap-8px">
      <span class="text-14px color-#333"
        >正在处理第 {{ progress.current }} 个文件，共
        {{ progress.total }} 个</span
      >
      <!-- <span
        class="text-12px color-#666 max-w-full overflow-hidden text-ellipsis whitespace-nowrap"
        v-if="progress.current_file"
        >{{ progress.current_file }}</span
      > -->
    </div>
  </section>

  <!-- 结果消息 -->
  <section
    class="bg-white rounded-8px p-15px shadow-sm dark:bg-#2d2d2d"
    v-if="resultMessage"
  >
    <h2
      class="text-16px mb-10px color-#555 border-b border-b-#ddd border-b-solid pb-5px dark:color-#ccc dark:border-b-#444"
    >
      处理结果
    </h2>
    <pre
      class="bg-#f8f9fa p-10px rounded-4px whitespace-pre-wrap font-mono text-12px max-h-200px overflow-y-auto dark:bg-#333 dark:color-#f6f6f6"
      >{{ resultMessage }}</pre
    >
  </section>
</template>

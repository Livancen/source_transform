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
  <!-- 操作按钮 -->
  <section class="section">
    <button
      class="primary-btn"
      @click="emit('startProcess')"
      :disabled="isProcessing || filesLength === 0"
    >
      {{ isProcessing ? "处理中..." : "开始处理" }}
    </button>
  </section>

  <!-- 进度显示 -->
  <section class="section" v-if="isProcessing && progress">
    <div class="processing-status">
      <span class="processing-text"
        >正在处理第 {{ progress.current }} 个文件，共
        {{ progress.total }} 个</span
      >
      <span class="processing-file" v-if="progress.current_file">{{
        progress.current_file
      }}</span>
    </div>
  </section>

  <!-- 结果消息 -->
  <section class="section" v-if="resultMessage">
    <h2>处理结果</h2>
    <pre class="result-message">{{ resultMessage }}</pre>
  </section>
</template>

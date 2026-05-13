<script setup lang="ts">
defineProps<{
  inputDir: string;
  outputDir: string;
}>();

const emit = defineEmits<{
  selectInput: [];
  selectOutput: [];
  openFolder: [path: string];
}>();
</script>

<template>
  <section class="flex-1 mb-0 bg-white rounded-8px p-15px shadow-sm dark:bg-#2d2d2d">
    <h2 class="text-16px mb-10px color-#555 border-b border-b-#ddd border-b-solid pb-5px dark:color-#ccc dark:border-b-#444">目录设置</h2>
    <div class="flex items-center gap-10px mb-10px">
      <label class="w-40px shrink-0">输入:</label>
      <input type="text" :value="inputDir" readonly class="flex-1 p-8px border border-#ddd rounded-4px bg-#f9f9f9 dark:bg-#333 dark:border-#444 dark:color-#f6f6f6" />
      <button @click="emit('selectInput')">选择</button>
      <button @click="emit('openFolder', inputDir)" :disabled="!inputDir">
        打开
      </button>
    </div>
    <div class="flex items-center gap-10px mb-10px">
      <label class="w-40px shrink-0">输出:</label>
      <input type="text" :value="outputDir" readonly class="flex-1 p-8px border border-#ddd rounded-4px bg-#f9f9f9 dark:bg-#333 dark:border-#444 dark:color-#f6f6f6" />
      <button @click="emit('selectOutput')">选择</button>
      <button @click="emit('openFolder', outputDir)" :disabled="!outputDir">
        打开
      </button>
    </div>
  </section>
</template>

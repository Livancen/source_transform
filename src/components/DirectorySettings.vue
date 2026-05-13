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
  <section class="section dir-section">
    <h2>目录设置</h2>
    <div class="dir-row">
      <label>输入:</label>
      <input type="text" :value="inputDir" readonly />
      <button @click="emit('selectInput')">选择</button>
      <button @click="emit('openFolder', inputDir)" :disabled="!inputDir">
        打开
      </button>
    </div>
    <div class="dir-row">
      <label>输出:</label>
      <input type="text" :value="outputDir" readonly />
      <button @click="emit('selectOutput')">选择</button>
      <button @click="emit('openFolder', outputDir)" :disabled="!outputDir">
        打开
      </button>
    </div>
  </section>
</template>

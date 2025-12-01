<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";

interface FileInfo {
  path: string;
  name: string;
  file_type: string;
}

interface ProcessProgress {
  current: number;
  total: number;
  current_file: string;
  status: string;
}

interface ProcessOptions {
  compress: boolean;
  compress_quality: number;
  compress_resize: boolean;
  compress_width: number;
  compress_height: number;
  reduce_resolution: boolean;
  target_width: number;
  target_height: number;
  reduce_bitrate: boolean;
  target_bitrate: string;
  reduce_profile: boolean;
  target_profile: string;
  rotate: boolean;
  rotation_degrees: number;
}

// 目录状态
const inputDir = ref("");
const outputDir = ref("");

// 文件列表
const files = ref<FileInfo[]>([]);

// 处理选项
const options = ref<ProcessOptions>({
  compress: false,
  compress_quality: 80,
  compress_resize: false,
  compress_width: 1280,
  compress_height: 720,
  reduce_resolution: false,
  target_width: 1920,
  target_height: 1080,
  reduce_bitrate: false,
  target_bitrate: "2M",
  reduce_profile: false,
  target_profile: "main",
  rotate: false,
  rotation_degrees: 90,
});

// 处理状态
const isProcessing = ref(false);
const progress = ref<ProcessProgress | null>(null);
const resultMessage = ref("");

// 计算属性
const imageCount = computed(
  () => files.value.filter((f) => f.file_type === "image").length
);
const videoCount = computed(
  () => files.value.filter((f) => f.file_type === "video").length
);
const progressPercent = computed(() => {
  if (!progress.value) return 0;
  return Math.round((progress.value.current / progress.value.total) * 100);
});

// 初始化
onMounted(async () => {
  try {
    const dirs = await invoke<[string, string]>("get_custom_dirs");
    inputDir.value = dirs[0];
    outputDir.value = dirs[1];
    await scanFiles();
  } catch (e) {
    console.error("初始化失败:", e);
  }

  // 监听进度事件
  await listen<ProcessProgress>("process-progress", (event) => {
    progress.value = event.payload;
  });
});

// 选择输入目录
async function selectInputDir() {
  const selected = await open({
    directory: true,
    title: "选择输入目录",
  });
  if (selected) {
    inputDir.value = selected as string;
    await saveCustomDirs();
    await scanFiles();
  }
}

// 选择输出目录
async function selectOutputDir() {
  const selected = await open({
    directory: true,
    title: "选择输出目录",
  });
  if (selected) {
    outputDir.value = selected as string;
    await saveCustomDirs();
  }
}

// 保存自定义目录
async function saveCustomDirs() {
  try {
    await invoke("set_custom_dirs", {
      inputPath: inputDir.value,
      outputPath: outputDir.value,
    });
  } catch (e) {
    console.error("保存目录失败:", e);
  }
}

// 扫描文件
async function scanFiles() {
  if (!inputDir.value) return;
  try {
    files.value = await invoke<FileInfo[]>("scan_input_files", {
      inputDir: inputDir.value,
    });
  } catch (e) {
    console.error("扫描文件失败:", e);
    files.value = [];
  }
}

// 开始处理
async function startProcess() {
  if (files.value.length === 0) {
    resultMessage.value = "没有可处理的文件";
    return;
  }

  isProcessing.value = true;
  progress.value = null;
  resultMessage.value = "";

  try {
    const result = await invoke<string>("process_files", {
      inputDir: inputDir.value,
      outputDir: outputDir.value,
      options: options.value,
    });
    resultMessage.value = result;
  } catch (e) {
    resultMessage.value = `处理失败: ${e}`;
  } finally {
    isProcessing.value = false;
  }
}

// 打开文件夹
async function openFolder(path: string) {
  try {
    await invoke("open_folder", { path });
  } catch (e) {
    console.error("打开文件夹失败:", e);
  }
}
</script>

<template>
  <main class="container">
    <!-- 目录设置 + 文件统计 -->
    <div class="top-row">
      <section class="section dir-section">
        <h2>目录设置</h2>
        <div class="dir-row">
          <label>输入:</label>
          <input type="text" v-model="inputDir" readonly />
          <button @click="selectInputDir">选择</button>
          <button @click="openFolder(inputDir)" :disabled="!inputDir">
            打开
          </button>
        </div>
        <div class="dir-row">
          <label>输出:</label>
          <input type="text" v-model="outputDir" readonly />
          <button @click="selectOutputDir">选择</button>
          <button @click="openFolder(outputDir)" :disabled="!outputDir">
            打开
          </button>
        </div>
      </section>

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
              <span class="stat-value">{{ files.length }}</span>
            </div>
            <button class="refresh-btn" @click="scanFiles" :disabled="isProcessing">刷新</button>
          </div>
        </div>
      </section>
    </div>

    <!-- 处理选项 -->
    <section class="section">
      <h2>处理选项</h2>
      <div class="options-grid">
        <!-- 左列 -->
        <div class="options-column">
          <!-- 压缩 -->
          <div class="option-group">
            <label class="checkbox-label">
              <input type="checkbox" v-model="options.compress" />
              启用压缩
            </label>
            <div v-if="options.compress" class="option-detail">
              <label>质量:</label>
              <input
                type="range"
                v-model.number="options.compress_quality"
                min="1"
                max="100"
              />
              <span>{{ options.compress_quality }}</span>
            </div>
            <div v-if="options.compress" class="compress-mode">
              <label class="radio-label">
                <input type="radio" :value="false" v-model="options.compress_resize" />
                仅压缩质量（保持原分辨率）
              </label>
              <label class="radio-label">
                <input type="radio" :value="true" v-model="options.compress_resize" />
                压缩质量 + 降低分辨率
              </label>
              <div v-if="options.compress_resize" class="option-detail compress-size">
                <label>宽:</label>
                <input type="number" v-model.number="options.compress_width" min="1" />
                <label>高:</label>
                <input type="number" v-model.number="options.compress_height" min="1" />
              </div>
            </div>
          </div>

          <!-- 降低分辨率 -->
          <div class="option-group">
            <label class="checkbox-label">
              <input type="checkbox" v-model="options.reduce_resolution" />
              降低分辨率
            </label>
            <div v-if="options.reduce_resolution" class="option-detail">
              <label>宽:</label>
              <input
                type="number"
                v-model.number="options.target_width"
                min="1"
              />
              <label>高:</label>
              <input
                type="number"
                v-model.number="options.target_height"
                min="1"
              />
            </div>
          </div>

          <!-- 旋转 -->
          <div class="option-group">
            <label class="checkbox-label">
              <input type="checkbox" v-model="options.rotate" />
              旋转
            </label>
            <div v-if="options.rotate" class="option-detail">
              <label>角度:</label>
              <select v-model.number="options.rotation_degrees">
                <option :value="90">顺时针 90°</option>
                <option :value="180">180°</option>
                <option :value="270">顺时针 270°</option>
                <option :value="-90">逆时针 90°</option>
              </select>
            </div>
          </div>
        </div>

        <!-- 右列 -->
        <div class="options-column">
          <!-- 降低码率 (仅视频) -->
          <div class="option-group">
            <label class="checkbox-label">
              <input type="checkbox" v-model="options.reduce_bitrate" />
              降低码率 (视频)
            </label>
            <div v-if="options.reduce_bitrate" class="option-detail">
              <label>码率:</label>
              <select v-model="options.target_bitrate">
                <option value="500k">500 Kbps</option>
                <option value="1M">1 Mbps</option>
                <option value="2M">2 Mbps</option>
                <option value="5M">5 Mbps</option>
                <option value="10M">10 Mbps</option>
              </select>
            </div>
          </div>

          <!-- 降低Profile等级 (仅视频) -->
          <div class="option-group">
            <label class="checkbox-label">
              <input type="checkbox" v-model="options.reduce_profile" />
              降低Profile (视频)
            </label>
            <div v-if="options.reduce_profile" class="option-detail">
              <label>Profile:</label>
              <select v-model="options.target_profile">
                <option value="baseline">Baseline</option>
                <option value="main">Main</option>
                <option value="high">High</option>
              </select>
            </div>
          </div>
        </div>
      </div>
    </section>

    <!-- 操作按钮 -->
    <section class="section">
      <button
        class="primary-btn"
        @click="startProcess"
        :disabled="isProcessing || files.length === 0"
      >
        {{ isProcessing ? "处理中..." : "开始处理" }}
      </button>
    </section>

    <!-- 进度显示 -->
    <section class="section" v-if="isProcessing && progress">
      <h2>处理进度</h2>
      <div class="progress-container">
        <div class="progress-bar">
          <div
            class="progress-fill"
            :style="{ width: progressPercent + '%' }"
          ></div>
        </div>
        <div class="progress-text">
          {{ progress.current }} / {{ progress.total }} ({{ progressPercent }}%)
        </div>
        <div class="current-file" v-if="progress.current_file">
          当前处理: {{ progress.current_file }}
        </div>
      </div>
    </section>

    <!-- 结果消息 -->
    <section class="section" v-if="resultMessage">
      <h2>处理结果</h2>
      <pre class="result-message">{{ resultMessage }}</pre>
    </section>
  </main>
</template>

<style>
* {
  margin: 0;
  padding: 0;
}
:root {
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  font-size: 14px;
  line-height: 1.5;
  font-weight: 400;
  color: #0f0f0f;
  background-color: #f6f6f6;
}

.container {
  max-width: 900px;
  margin: 0 auto;
  padding: 10px;
}

.top-row {
  display: flex;
  gap: 15px;
}

.dir-section {
  flex: 1;
  margin-bottom: 0;
}

.stats-section {
  width: 200px;
  flex-shrink: 0;
  margin-bottom: 0;
  display: flex;
  flex-direction: column;
}

.stats {
  display: flex;
  flex-direction: column;
  gap: 8px;
  flex: 1;
}

.stat-item {
  display: flex;
  justify-content: space-between;
  padding: 4px 8px;
  background: #e9ecef;
  border-radius: 4px;
}

.stat-row-last {
  display: flex;
  gap: 8px;
}

.stat-row-last .stat-item {
  flex: 1;
}

.refresh-btn {
  flex: 1;
  padding: 4px 8px;
  font-size: 12px;
}

.stat-label {
  color: #666;
}

.stat-value {
  font-weight: 600;
}

h2 {
  font-size: 16px;
  margin-bottom: 10px;
  color: #555;
  border-bottom: 1px solid #ddd;
  padding-bottom: 5px;
}

.section {
  background: #fff;
  border-radius: 8px;
  padding: 15px;
  margin-bottom: 15px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}

.dir-row {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 10px;
}

.dir-row label {
  width: 40px;
  flex-shrink: 0;
}

.dir-row input {
  flex: 1;
  padding: 8px;
  border: 1px solid #ddd;
  border-radius: 4px;
  background: #f9f9f9;
}

.options-grid {
  display: flex;
  gap: 15px;
}

.options-column {
  flex: 1;
}

.option-group {
  margin-bottom: 10px;
  padding: 10px;
  background: #f9f9f9;
  border-radius: 4px;
}

.option-group:last-child {
  margin-bottom: 0;
}

.checkbox-label {
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  font-weight: 500;
}

.checkbox-label.sub-option {
  font-weight: 400;
  font-size: 13px;
}

.compress-mode {
  margin-top: 10px;
  padding-left: 26px;
}

.radio-label {
  display: flex;
  align-items: center;
  gap: 6px;
  cursor: pointer;
  font-size: 13px;
  margin-bottom: 6px;
}

.radio-label input[type="radio"] {
  width: 14px;
  height: 14px;
}

.compress-size {
  margin-top: 8px;
  padding-left: 20px;
}

.checkbox-label input[type="checkbox"] {
  width: 18px;
  height: 18px;
}

.option-detail {
  margin-top: 10px;
  padding-left: 26px;
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
}

.option-detail label {
  color: #666;
}

.option-detail input[type="number"] {
  width: 100px;
  padding: 5px;
  border: 1px solid #ddd;
  border-radius: 4px;
}

.option-detail input[type="range"] {
  width: 150px;
}

.option-detail select {
  padding: 5px 10px;
  border: 1px solid #ddd;
  border-radius: 4px;
}

button {
  padding: 8px 16px;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  background: #6c757d;
  color: white;
  transition: background 0.2s;
}

button:hover:not(:disabled) {
  background: #5a6268;
}

button:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.primary-btn {
  width: 100%;
  padding: 12px;
  font-size: 16px;
  background: #007bff;
}

.primary-btn:hover:not(:disabled) {
  background: #0056b3;
}

.progress-container {
  margin-top: 10px;
}

.progress-bar {
  height: 20px;
  background: #e9ecef;
  border-radius: 10px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: linear-gradient(90deg, #007bff, #00c6ff);
  transition: width 0.3s;
}

.progress-text {
  text-align: center;
  margin-top: 5px;
  font-weight: 500;
}

.current-file {
  text-align: center;
  margin-top: 5px;
  color: #666;
  font-size: 12px;
}

.result-message {
  background: #f8f9fa;
  padding: 10px;
  border-radius: 4px;
  white-space: pre-wrap;
  font-family: monospace;
  font-size: 12px;
  max-height: 200px;
  overflow-y: auto;
}

@media (prefers-color-scheme: dark) {
  :root {
    color: #f6f6f6;
    background-color: #1a1a1a;
  }

  .section {
    background: #2d2d2d;
  }

  h2 {
    color: #ccc;
    border-bottom-color: #444;
  }

  .dir-row input {
    background: #333;
    border-color: #444;
    color: #f6f6f6;
  }

  .stat-item {
    background: #444;
  }

  .stat-label {
    color: #aaa;
  }

  .option-group {
    background: #333;
  }

  .option-detail label {
    color: #aaa;
  }

  .option-detail input,
  .option-detail select {
    background: #444;
    border-color: #555;
    color: #f6f6f6;
  }

  .progress-bar {
    background: #444;
  }

  .result-message {
    background: #333;
    color: #f6f6f6;
  }
}
</style>

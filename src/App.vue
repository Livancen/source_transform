<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import { convertFileSrc } from "@tauri-apps/api/core";

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

interface VideoInfo {
  path: string;
  name: string;
  codec: string;
  profile: string;
  level: string;
  width: number;
  height: number;
  framerate: number;
  bitrate: number;
}

interface DeviceCompatibility {
  device: string;
  compatible: boolean;
  reason: string;
}

interface VideoCompatibilityResult {
  path: string;
  name: string;
  video_info: VideoInfo;
  devices: DeviceCompatibility[];
  thumbnail: string;
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
  reduce_level: boolean;
  target_level: string;
  convert_h265_to_h264: boolean;
  convert_format: boolean;
  target_format: string;
  crop: boolean;
  crop_width: number;
  crop_height: number;
  crop_x: number;
  crop_y: number;
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
  reduce_level: false,
  target_level: "4.0",
  convert_h265_to_h264: false,
  convert_format: false,
  target_format: "mp4",
  crop: false,
  crop_width: 1280,
  crop_height: 720,
  crop_x: 0,
  crop_y: 0,
  rotate: false,
  rotation_degrees: 90,
});

// 处理状态
const isProcessing = ref(false);
const progress = ref<ProcessProgress | null>(null);
const resultMessage = ref("");

// 兼容性检测状态
const isDetecting = ref(false);
const compatibilityResults = ref<VideoCompatibilityResult[]>([]);
const showCompatibilityModal = ref(false);

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

// 裁剪预览状态
const cropPreviewVisible = ref(false);
const cropPreviewVideo = ref<string>("");
const cropFrameImage = ref<string>("");
const cropVideoWidth = ref(1920);
const cropVideoHeight = ref(1080);
const previewScale = ref(1);
const cropAreaRef = ref<HTMLDivElement | null>(null);
const previewContainerRef = ref<HTMLDivElement | null>(null);

// 拖拽状态
const isDragging = ref(false);
const isResizing = ref(false);
const resizeHandle = ref("");
const dragStartX = ref(0);
const dragStartY = ref(0);
const startCropX = ref(0);
const startCropY = ref(0);
const startCropW = ref(0);
const startCropH = ref(0);

// 获取视频列表
const videoFiles = computed(() =>
  files.value.filter((f) => f.file_type === "video")
);

// 打开裁剪预览
async function openCropPreview() {
  if (videoFiles.value.length === 0) {
    resultMessage.value = "没有可预览的视频文件";
    return;
  }
  // 使用第一个视频文件作为预览
  const firstVideo = videoFiles.value[0];
  cropPreviewVideo.value = firstVideo.path;

  // 获取视频尺寸
  try {
    const dimensions = await invoke<[number, number]>("get_video_dimensions", {
      videoPath: firstVideo.path,
    });
    cropVideoWidth.value = dimensions[0];
    cropVideoHeight.value = dimensions[1];

    // 提取视频第一帧
    const framePath = await invoke<string>("extract_video_frame", {
      videoPath: firstVideo.path,
    });
    cropFrameImage.value = framePath;

    // 计算预览缩放比例，最大宽度600px
    const maxPreviewWidth = 600;
    previewScale.value = Math.min(1, maxPreviewWidth / cropVideoWidth.value);

    // 初始化裁剪区域为视频中心
    if (options.value.crop_width > cropVideoWidth.value) {
      options.value.crop_width = cropVideoWidth.value;
    }
    if (options.value.crop_height > cropVideoHeight.value) {
      options.value.crop_height = cropVideoHeight.value;
    }
    options.value.crop_x = Math.floor((cropVideoWidth.value - options.value.crop_width) / 2);
    options.value.crop_y = Math.floor((cropVideoHeight.value - options.value.crop_height) / 2);

    cropPreviewVisible.value = true;
  } catch (e) {
    resultMessage.value = `获取视频信息失败: ${e}`;
  }
}

// 关闭裁剪预览
function closeCropPreview() {
  cropPreviewVisible.value = false;
}

// 确认裁剪设置
function confirmCrop() {
  options.value.crop = true;
  cropPreviewVisible.value = false;
}

// 开始拖拽裁剪区域
function startDrag(e: MouseEvent) {
  if (isResizing.value) return;
  isDragging.value = true;
  dragStartX.value = e.clientX;
  dragStartY.value = e.clientY;
  startCropX.value = options.value.crop_x;
  startCropY.value = options.value.crop_y;
  e.preventDefault();
}

// 开始调整大小
function startResize(e: MouseEvent, handle: string) {
  isResizing.value = true;
  resizeHandle.value = handle;
  dragStartX.value = e.clientX;
  dragStartY.value = e.clientY;
  startCropX.value = options.value.crop_x;
  startCropY.value = options.value.crop_y;
  startCropW.value = options.value.crop_width;
  startCropH.value = options.value.crop_height;
  e.preventDefault();
  e.stopPropagation();
}

// 处理鼠标移动
function handleMouseMove(e: MouseEvent) {
  if (!isDragging.value && !isResizing.value) return;

  const deltaX = Math.round((e.clientX - dragStartX.value) / previewScale.value);
  const deltaY = Math.round((e.clientY - dragStartY.value) / previewScale.value);

  if (isDragging.value) {
    let newX = startCropX.value + deltaX;
    let newY = startCropY.value + deltaY;

    // 边界限制
    newX = Math.max(0, Math.min(newX, cropVideoWidth.value - options.value.crop_width));
    newY = Math.max(0, Math.min(newY, cropVideoHeight.value - options.value.crop_height));

    options.value.crop_x = newX;
    options.value.crop_y = newY;
  } else if (isResizing.value) {
    let newX = startCropX.value;
    let newY = startCropY.value;
    let newW = startCropW.value;
    let newH = startCropH.value;

    const handle = resizeHandle.value;

    if (handle.includes('e')) {
      newW = Math.max(100, Math.min(startCropW.value + deltaX, cropVideoWidth.value - startCropX.value));
    }
    if (handle.includes('w')) {
      const maxDelta = startCropX.value;
      const clampedDelta = Math.max(-maxDelta, Math.min(deltaX, startCropW.value - 100));
      newX = startCropX.value + clampedDelta;
      newW = startCropW.value - clampedDelta;
    }
    if (handle.includes('s')) {
      newH = Math.max(100, Math.min(startCropH.value + deltaY, cropVideoHeight.value - startCropY.value));
    }
    if (handle.includes('n')) {
      const maxDelta = startCropY.value;
      const clampedDelta = Math.max(-maxDelta, Math.min(deltaY, startCropH.value - 100));
      newY = startCropY.value + clampedDelta;
      newH = startCropH.value - clampedDelta;
    }

    options.value.crop_x = newX;
    options.value.crop_y = newY;
    options.value.crop_width = newW;
    options.value.crop_height = newH;
  }
}

// 结束拖拽/调整大小
function handleMouseUp() {
  isDragging.value = false;
  isResizing.value = false;
}

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

// 检测兼容性
async function detectCompatibility() {
  if (!inputDir.value) {
    resultMessage.value = "请先选择输入目录";
    return;
  }

  isDetecting.value = true;
  compatibilityResults.value = [];

  try {
    const results = await invoke<VideoCompatibilityResult[]>("detect_video_compatibility", {
      inputDir: inputDir.value,
    });
    compatibilityResults.value = results;
    showCompatibilityModal.value = true;
  } catch (e) {
    resultMessage.value = `检测失败: ${e}`;
  } finally {
    isDetecting.value = false;
  }
}

// 关闭兼容性弹窗
function closeCompatibilityModal() {
  showCompatibilityModal.value = false;
}

// 获取不兼容的文件数量
function getIncompatibleCount(device: string): number {
  return compatibilityResults.value.filter(
    (r) => !r.devices.find((d) => d.device === device)?.compatible
  ).length;
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
  <main class="container" @contextmenu.prevent>
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
          <button
            class="detect-btn"
            @click="detectCompatibility"
            :disabled="isDetecting || videoCount === 0"
          >
            {{ isDetecting ? "检测中..." : "兼容性检测" }}
          </button>
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

          <!-- 裁剪 (仅视频) -->
          <div class="option-group">
            <label class="checkbox-label">
              <input type="checkbox" v-model="options.crop" />
              裁剪 (视频)
            </label>
            <div v-if="options.crop" class="option-detail crop-info">
              <span>{{ options.crop_width }} x {{ options.crop_height }}</span>
              <span>位置: ({{ options.crop_x }}, {{ options.crop_y }})</span>
              <button class="crop-btn" @click="openCropPreview" :disabled="videoFiles.length === 0">
                设置裁剪区域
              </button>
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
                <option value="200k">200 Kbps</option>
                <option value="500k">500 Kbps</option>
                <option value="800k">800 Kbps</option>
                <option value="1M">1 Mbps</option>
                <option value="2M">2 Mbps</option>
                <option value="3M">3 Mbps</option>
                <option value="5M">5 Mbps</option>
                <option value="8M">8 Mbps</option>
                <option value="10M">10 Mbps</option>
                <option value="15M">15 Mbps</option>
                <option value="20M">20 Mbps</option>
              </select>
            </div>
          </div>

          <!-- 降低Level等级 (仅视频) -->
          <div class="option-group">
            <label class="checkbox-label">
              <input type="checkbox" v-model="options.reduce_level" />
              降低Level (视频)
            </label>
            <div v-if="options.reduce_level" class="option-detail">
              <label>Level:</label>
              <select v-model="options.target_level">
                <option value="3.0">3.0 (SD 480p)</option>
                <option value="3.1">3.1 (720p@30fps)</option>
                <option value="4.0">4.0 (1080p@30fps)</option>
                <option value="4.1">4.1 (1080p@30fps)</option>
                <option value="4.2">4.2 (1080p@60fps)</option>
                <option value="5.0">5.0 (2K)</option>
                <option value="5.1">5.1 (4K@30fps)</option>
                <option value="5.2">5.2 (4K@60fps)</option>
              </select>
            </div>
          </div>

          <!-- H.265转H.264 (仅视频) -->
          <div class="option-group">
            <label class="checkbox-label">
              <input type="checkbox" v-model="options.convert_h265_to_h264" />
              H.265 转 H.264 (视频)
            </label>
          </div>

          <!-- 视频格式转换 -->
          <div class="option-group">
            <label class="checkbox-label">
              <input type="checkbox" v-model="options.convert_format" />
              格式转换 (视频)
            </label>
            <div v-if="options.convert_format" class="option-detail">
              <label>目标格式:</label>
              <select v-model="options.target_format">
                <option value="mp4">MP4</option>
                <option value="avi">AVI</option>
                <option value="mkv">MKV</option>
                <option value="mov">MOV</option>
                <option value="webm">WebM</option>
                <option value="flv">FLV</option>
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
      <div class="processing-status">
        <span class="processing-text">正在处理第 {{ progress.current }} 个文件，共 {{ progress.total }} 个</span>
        <span class="processing-file" v-if="progress.current_file">{{ progress.current_file }}</span>
      </div>
    </section>

    <!-- 结果消息 -->
    <section class="section" v-if="resultMessage">
      <h2>处理结果</h2>
      <pre class="result-message">{{ resultMessage }}</pre>
    </section>

    <!-- 裁剪预览弹窗 -->
    <div class="crop-modal" v-if="cropPreviewVisible" @mousemove="handleMouseMove" @mouseup="handleMouseUp" @mouseleave="handleMouseUp">
      <div class="crop-modal-content">
        <h3>设置裁剪区域</h3>
        <p class="crop-hint">拖拽移动裁剪框，拖拽边角调整大小</p>

        <div class="crop-preview-container" ref="previewContainerRef">
          <img
            :src="cropFrameImage"
            :style="{
              width: cropVideoWidth * previewScale + 'px',
              height: cropVideoHeight * previewScale + 'px'
            }"
            draggable="false"
          />

          <!-- 裁剪区域遮罩 -->
          <div class="crop-overlay">
            <!-- 上方遮罩 -->
            <div class="crop-mask crop-mask-top" :style="{
              height: options.crop_y * previewScale + 'px'
            }"></div>
            <!-- 下方遮罩 -->
            <div class="crop-mask crop-mask-bottom" :style="{
              height: (cropVideoHeight - options.crop_y - options.crop_height) * previewScale + 'px'
            }"></div>
            <!-- 左侧遮罩 -->
            <div class="crop-mask crop-mask-left" :style="{
              top: options.crop_y * previewScale + 'px',
              height: options.crop_height * previewScale + 'px',
              width: options.crop_x * previewScale + 'px'
            }"></div>
            <!-- 右侧遮罩 -->
            <div class="crop-mask crop-mask-right" :style="{
              top: options.crop_y * previewScale + 'px',
              height: options.crop_height * previewScale + 'px',
              width: (cropVideoWidth - options.crop_x - options.crop_width) * previewScale + 'px'
            }"></div>
          </div>

          <!-- 裁剪选择框 -->
          <div
            class="crop-area"
            ref="cropAreaRef"
            :style="{
              left: options.crop_x * previewScale + 'px',
              top: options.crop_y * previewScale + 'px',
              width: options.crop_width * previewScale + 'px',
              height: options.crop_height * previewScale + 'px'
            }"
            @mousedown="startDrag"
          >
            <!-- 调整大小的手柄 -->
            <div class="resize-handle resize-n" @mousedown.stop="startResize($event, 'n')"></div>
            <div class="resize-handle resize-s" @mousedown.stop="startResize($event, 's')"></div>
            <div class="resize-handle resize-e" @mousedown.stop="startResize($event, 'e')"></div>
            <div class="resize-handle resize-w" @mousedown.stop="startResize($event, 'w')"></div>
            <div class="resize-handle resize-ne" @mousedown.stop="startResize($event, 'ne')"></div>
            <div class="resize-handle resize-nw" @mousedown.stop="startResize($event, 'nw')"></div>
            <div class="resize-handle resize-se" @mousedown.stop="startResize($event, 'se')"></div>
            <div class="resize-handle resize-sw" @mousedown.stop="startResize($event, 'sw')"></div>

            <!-- 尺寸显示 -->
            <div class="crop-size-label">{{ options.crop_width }} x {{ options.crop_height }}</div>
          </div>
        </div>

        <!-- 精确输入 -->
        <div class="crop-inputs">
          <div class="crop-input-group">
            <label>宽度:</label>
            <input type="number" v-model.number="options.crop_width" min="100" :max="cropVideoWidth" />
          </div>
          <div class="crop-input-group">
            <label>高度:</label>
            <input type="number" v-model.number="options.crop_height" min="100" :max="cropVideoHeight" />
          </div>
          <div class="crop-input-group">
            <label>X:</label>
            <input type="number" v-model.number="options.crop_x" min="0" :max="cropVideoWidth - options.crop_width" />
          </div>
          <div class="crop-input-group">
            <label>Y:</label>
            <input type="number" v-model.number="options.crop_y" min="0" :max="cropVideoHeight - options.crop_height" />
          </div>
        </div>

        <div class="crop-video-info">
          原始尺寸: {{ cropVideoWidth }} x {{ cropVideoHeight }}
        </div>

        <div class="crop-modal-actions">
          <button @click="closeCropPreview">取消</button>
          <button class="primary-btn" @click="confirmCrop">确认</button>
        </div>
      </div>
    </div>

    <!-- 兼容性检测结果弹窗 -->
    <div class="compat-modal" v-if="showCompatibilityModal">
      <div class="compat-modal-content">
        <h3>兼容性检测结果</h3>

        <!-- 统计摘要 -->
        <div class="compat-summary">
          <div class="compat-summary-item">
            <span class="device-name">RK3399</span>
            <span :class="['compat-count', getIncompatibleCount('RK3399') > 0 ? 'has-issues' : 'all-good']">
              {{ getIncompatibleCount('RK3399') > 0 ? `${getIncompatibleCount('RK3399')} 个不兼容` : '全部兼容' }}
            </span>
          </div>
          <div class="compat-summary-item">
            <span class="device-name">RK3566</span>
            <span :class="['compat-count', getIncompatibleCount('RK3566') > 0 ? 'has-issues' : 'all-good']">
              {{ getIncompatibleCount('RK3566') > 0 ? `${getIncompatibleCount('RK3566')} 个不兼容` : '全部兼容' }}
            </span>
          </div>
          <div class="compat-summary-item">
            <span class="device-name">RK3588</span>
            <span :class="['compat-count', getIncompatibleCount('RK3588') > 0 ? 'has-issues' : 'all-good']">
              {{ getIncompatibleCount('RK3588') > 0 ? `${getIncompatibleCount('RK3588')} 个不兼容` : '全部兼容' }}
            </span>
          </div>
        </div>

        <!-- 详细结果列表 -->
        <div class="compat-results">
          <div
            class="compat-item"
            v-for="result in compatibilityResults"
            :key="result.path"
          >
            <div class="compat-file-info">
              <img
                v-if="result.thumbnail"
                :src="result.thumbnail"
                class="video-thumbnail"
                alt="缩略图"
              />
              <div class="file-details">
                <span class="file-name">{{ result.name }}</span>
                <span class="video-specs">
                  {{ result.video_info.codec.toUpperCase() }} |
                  {{ result.video_info.width }}x{{ result.video_info.height }} |
                  {{ Math.round(result.video_info.framerate) }}fps |
                  Level {{ result.video_info.level }}
                </span>
              </div>
            </div>
            <div class="compat-devices">
              <span
                v-for="device in result.devices"
                :key="device.device"
                :class="['device-badge', device.compatible ? 'compatible' : 'incompatible']"
                :title="device.reason"
              >
                {{ device.device }}
                <span class="device-status">{{ device.compatible ? '✓' : '✗' }}</span>
              </span>
            </div>
          </div>
        </div>

        <div class="compat-modal-actions">
          <button class="primary-btn" @click="closeCompatibilityModal">关闭</button>
        </div>
      </div>
    </div>
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
  user-select: none;
  -webkit-user-select: none;
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

.processing-status {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
}

.processing-text {
  font-size: 14px;
  color: #333;
}

.processing-file {
  font-size: 12px;
  color: #666;
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
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

/* 裁剪相关样式 */
.crop-info {
  flex-direction: column;
  align-items: flex-start !important;
  gap: 5px !important;
}

.crop-info span {
  font-size: 12px;
  color: #666;
}

.crop-btn {
  margin-top: 5px;
  padding: 4px 12px !important;
  font-size: 12px;
}

.crop-modal {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background: rgba(0, 0, 0, 0.7);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.crop-modal-content {
  background: #fff;
  border-radius: 8px;
  padding: 20px;
  max-width: 90%;
  max-height: 90%;
  overflow: auto;
}

.crop-modal-content h3 {
  margin-bottom: 10px;
  font-size: 18px;
}

.crop-hint {
  font-size: 12px;
  color: #666;
  margin-bottom: 15px;
}

.crop-preview-container {
  position: relative;
  display: inline-block;
  background: #000;
  margin-bottom: 15px;
}

.crop-preview-container img {
  display: block;
  user-select: none;
  -webkit-user-drag: none;
}

.crop-overlay {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  pointer-events: none;
}

.crop-mask {
  position: absolute;
  background: rgba(0, 0, 0, 0.5);
}

.crop-mask-top {
  top: 0;
  left: 0;
  width: 100%;
}

.crop-mask-bottom {
  bottom: 0;
  left: 0;
  width: 100%;
}

.crop-mask-left {
  left: 0;
}

.crop-mask-right {
  right: 0;
}

.crop-area {
  position: absolute;
  border: 2px solid #007bff;
  cursor: move;
  box-sizing: border-box;
}

.crop-area::before {
  content: '';
  position: absolute;
  top: 33.33%;
  left: 0;
  right: 0;
  height: 1px;
  background: rgba(255, 255, 255, 0.5);
}

.crop-area::after {
  content: '';
  position: absolute;
  top: 66.66%;
  left: 0;
  right: 0;
  height: 1px;
  background: rgba(255, 255, 255, 0.5);
}

.resize-handle {
  position: absolute;
  width: 10px;
  height: 10px;
  background: #007bff;
  border: 1px solid #fff;
}

.resize-n {
  top: -5px;
  left: 50%;
  transform: translateX(-50%);
  cursor: n-resize;
}

.resize-s {
  bottom: -5px;
  left: 50%;
  transform: translateX(-50%);
  cursor: s-resize;
}

.resize-e {
  right: -5px;
  top: 50%;
  transform: translateY(-50%);
  cursor: e-resize;
}

.resize-w {
  left: -5px;
  top: 50%;
  transform: translateY(-50%);
  cursor: w-resize;
}

.resize-ne {
  top: -5px;
  right: -5px;
  cursor: ne-resize;
}

.resize-nw {
  top: -5px;
  left: -5px;
  cursor: nw-resize;
}

.resize-se {
  bottom: -5px;
  right: -5px;
  cursor: se-resize;
}

.resize-sw {
  bottom: -5px;
  left: -5px;
  cursor: sw-resize;
}

.crop-size-label {
  position: absolute;
  bottom: 5px;
  left: 50%;
  transform: translateX(-50%);
  background: rgba(0, 0, 0, 0.7);
  color: #fff;
  padding: 2px 8px;
  border-radius: 3px;
  font-size: 12px;
  white-space: nowrap;
}

.crop-inputs {
  display: flex;
  gap: 15px;
  margin-bottom: 10px;
  flex-wrap: wrap;
}

.crop-input-group {
  display: flex;
  align-items: center;
  gap: 5px;
}

.crop-input-group label {
  font-size: 12px;
  color: #666;
}

.crop-input-group input {
  width: 80px;
  padding: 5px;
  border: 1px solid #ddd;
  border-radius: 4px;
}

.crop-video-info {
  font-size: 12px;
  color: #999;
  margin-bottom: 15px;
}

.crop-modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}

.crop-modal-actions .primary-btn {
  width: auto;
  padding: 8px 20px;
}

/* 检测按钮 */
.detect-btn {
  width: 100%;
  margin-top: 8px;
  padding: 6px 12px;
  font-size: 12px;
  background: #17a2b8;
}

.detect-btn:hover:not(:disabled) {
  background: #138496;
}

/* 兼容性检测弹窗 */
.compat-modal {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background: rgba(0, 0, 0, 0.7);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.compat-modal-content {
  background: #fff;
  border-radius: 8px;
  padding: 20px;
  max-width: 700px;
  width: 90%;
  max-height: 80%;
  overflow: auto;
}

.compat-modal-content h3 {
  margin-bottom: 15px;
  font-size: 18px;
}

.compat-summary {
  display: flex;
  gap: 15px;
  margin-bottom: 20px;
  padding: 15px;
  background: #f8f9fa;
  border-radius: 6px;
}

.compat-summary-item {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 5px;
}

.device-name {
  font-weight: 600;
  font-size: 14px;
}

.compat-count {
  font-size: 12px;
  padding: 2px 8px;
  border-radius: 10px;
}

.compat-count.all-good {
  background: #d4edda;
  color: #155724;
}

.compat-count.has-issues {
  background: #f8d7da;
  color: #721c24;
}

.compat-results {
  max-height: 400px;
  overflow-y: auto;
  border: 1px solid #ddd;
  border-radius: 6px;
}

.compat-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 15px;
  border-bottom: 1px solid #eee;
}

.compat-item:last-child {
  border-bottom: none;
}

.compat-file-info {
  display: flex;
  align-items: center;
  gap: 10px;
}

.video-thumbnail {
  width: 60px;
  height: 45px;
  object-fit: cover;
  border-radius: 4px;
  flex-shrink: 0;
}

.file-details {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.file-name {
  font-weight: 500;
  font-size: 13px;
}

.video-specs {
  font-size: 11px;
  color: #666;
}

.compat-devices {
  display: flex;
  gap: 8px;
}

.device-badge {
  display: flex;
  align-items: center;
  gap: 3px;
  padding: 3px 8px;
  border-radius: 4px;
  font-size: 11px;
  font-weight: 500;
  cursor: help;
}

.device-badge.compatible {
  background: #d4edda;
  color: #155724;
}

.device-badge.incompatible {
  background: #f8d7da;
  color: #721c24;
}

.device-status {
  font-weight: bold;
}

.compat-modal-actions {
  display: flex;
  justify-content: flex-end;
  margin-top: 15px;
}

.compat-modal-actions .primary-btn {
  width: auto;
  padding: 8px 20px;
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

  .result-message {
    background: #333;
    color: #f6f6f6;
  }

  .crop-modal-content {
    background: #2d2d2d;
    color: #f6f6f6;
  }

  .crop-hint {
    color: #aaa;
  }

  .crop-info span {
    color: #aaa;
  }

  .crop-input-group label {
    color: #aaa;
  }

  .crop-input-group input {
    background: #444;
    border-color: #555;
    color: #f6f6f6;
  }

  .crop-video-info {
    color: #888;
  }

  .compat-modal-content {
    background: #2d2d2d;
    color: #f6f6f6;
  }

  .compat-summary {
    background: #333;
  }

  .compat-results {
    border-color: #444;
  }

  .compat-item {
    border-bottom-color: #444;
  }

  .video-specs {
    color: #aaa;
  }
}
</style>

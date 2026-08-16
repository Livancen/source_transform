import { ref, computed, watch, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import type { FileInfo, ProcessProgress, ProcessOptions } from "../types";

const RATIO_STORAGE_KEY = "aspect-ratio-crop-ratios";
const OPTIONS_OPEN_KEY = "options-strip-open";

const inputDir = ref("");
const outputDir = ref("");
const files = ref<FileInfo[]>([]);
const outputFiles = ref<FileInfo[]>([]);
const selectedInputPath = ref("");
const selectedOutputPath = ref("");

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
  target_profile: "high",
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
  mute: false,
  change_framerate: false,
  target_framerate: 30,
});

const isProcessing = ref(false);
const progress = ref<ProcessProgress | null>(null);
const resultMessage = ref("");
const uploadUrl = ref("");
const optionsOpen = ref(true);
const enableRatioCrop = ref(false);
const ratios = ref<string[]>([]);
const newRatio = ref("");
const ratioError = ref("");
const initialized = ref(false);

const imageCount = computed(
  () => files.value.filter((f) => f.file_type === "image").length,
);
const videoCount = computed(
  () => files.value.filter((f) => f.file_type === "video").length,
);
const videoFiles = computed(() =>
  files.value.filter((f) => f.file_type === "video"),
);

const statusMessage = computed(() => {
  if (isProcessing.value && progress.value) {
    return `正在处理 ${progress.value.current}/${progress.value.total} · ${progress.value.current_file || progress.value.status || ""}`;
  }
  if (resultMessage.value) {
    const oneLine = resultMessage.value.replace(/\s+/g, " ").trim();
    return oneLine.length > 120 ? oneLine.slice(0, 120) + "…" : oneLine;
  }
  return "就绪";
});

function addRatio() {
  ratioError.value = "";
  const val = newRatio.value.trim();
  if (!val) {
    ratioError.value = "请输入比例";
    return;
  }
  const parts = val.split(":");
  if (parts.length !== 2 || !Number(parts[0]) || !Number(parts[1])) {
    ratioError.value = "格式错误，应为 W:H 如 1:1";
    return;
  }
  if (ratios.value.includes(val)) {
    ratioError.value = "该比例已存在";
    return;
  }
  ratios.value.push(val);
  newRatio.value = "";
}

function removeRatio(i: number) {
  ratios.value.splice(i, 1);
}

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

async function scanOutputFiles() {
  if (!outputDir.value) {
    outputFiles.value = [];
    return;
  }
  try {
    outputFiles.value = await invoke<FileInfo[]>("scan_input_files", {
      inputDir: outputDir.value,
    });
  } catch (e) {
    console.error("扫描输出文件失败:", e);
    outputFiles.value = [];
  }
}

async function scanAllFiles() {
  await Promise.all([scanFiles(), scanOutputFiles()]);
}

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

async function selectOutputDir() {
  const selected = await open({
    directory: true,
    title: "选择输出目录",
  });
  if (selected) {
    outputDir.value = selected as string;
    await saveCustomDirs();
    await scanOutputFiles();
  }
}

async function openFolder(path: string) {
  try {
    await invoke("open_folder", { path });
  } catch (e) {
    console.error("打开文件夹失败:", e);
  }
}

async function startProcess() {
  if (files.value.length === 0) {
    resultMessage.value = "没有可处理的文件";
    return;
  }

  isProcessing.value = true;
  progress.value = null;
  resultMessage.value = "";

  const messages: string[] = [];

  if (enableRatioCrop.value && ratios.value.length > 0) {
    try {
      const cropResult = await invoke<string>("crop_videos_by_ratios", {
        inputDir: inputDir.value,
        outputDir: outputDir.value,
        ratios: ratios.value,
      });
      messages.push(cropResult);
    } catch (e) {
      messages.push(`比例裁剪失败: ${e}`);
    }
  } else {
    try {
      const result = await invoke<string>("process_files", {
        inputDir: inputDir.value,
        outputDir: outputDir.value,
        options: options.value,
      });
      messages.push(result);
    } catch (e) {
      messages.push(`处理失败: ${e}`);
    }
  }

  resultMessage.value = messages.join("");
  isProcessing.value = false;
  progress.value = null;
  await scanOutputFiles();
}

async function initWorkspace() {
  if (initialized.value) return;
  initialized.value = true;

  const savedRatios = localStorage.getItem(RATIO_STORAGE_KEY);
  if (savedRatios) {
    try {
      ratios.value = JSON.parse(savedRatios);
    } catch {
      /* ignore */
    }
  }

  const savedOpen = localStorage.getItem(OPTIONS_OPEN_KEY);
  if (savedOpen != null) {
    optionsOpen.value = savedOpen === "1";
  }

  try {
    const dirs = await invoke<[string, string]>("get_custom_dirs");
    inputDir.value = dirs[0];
    outputDir.value = dirs[1];
    await scanAllFiles();

    const url = await invoke<string>("start_upload_server", {
      inputDir: inputDir.value,
    });
    uploadUrl.value = url;
  } catch (e) {
    console.error("初始化失败:", e);
  }

  await listen<ProcessProgress>("process-progress", (event) => {
    progress.value = event.payload;
  });
  await listen<ProcessProgress>("crop-progress", (event) => {
    progress.value = event.payload;
  });
}

watch(
  ratios,
  (val) => {
    localStorage.setItem(RATIO_STORAGE_KEY, JSON.stringify(val));
  },
  { deep: true },
);

watch(optionsOpen, (val) => {
  localStorage.setItem(OPTIONS_OPEN_KEY, val ? "1" : "0");
});

export function useWorkspace() {
  onMounted(() => {
    initWorkspace();
  });

  return {
    inputDir,
    outputDir,
    files,
    outputFiles,
    selectedInputPath,
    selectedOutputPath,
    options,
    isProcessing,
    progress,
    resultMessage,
    uploadUrl,
    optionsOpen,
    enableRatioCrop,
    ratios,
    newRatio,
    ratioError,
    imageCount,
    videoCount,
    videoFiles,
    statusMessage,
    addRatio,
    removeRatio,
    saveCustomDirs,
    scanFiles,
    scanOutputFiles,
    scanAllFiles,
    selectInputDir,
    selectOutputDir,
    openFolder,
    startProcess,
    initWorkspace,
  };
}

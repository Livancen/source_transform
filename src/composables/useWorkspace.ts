import { ref, computed, watch, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import type {
  FileInfo,
  ProcessProgress,
  ProcessOptions,
  NamingOptions,
  WorkMode,
  HwAccelOptions,
} from "../types";
import { defaultProcessOptions, defaultNamingOptions } from "../types";
import { useToast } from "./useToast";

const RATIO_STORAGE_KEY = "aspect-ratio-crop-ratios";
const OPTIONS_OPEN_KEY = "options-strip-open";
const WORK_MODE_KEY = "work-mode";
const WORK_MODES: WorkMode[] = [
  "image",
  "video",
  "ratio",
  "crop",
  "merge",
  "join",
];

function loadWorkMode(): WorkMode {
  try {
    const saved = localStorage.getItem(WORK_MODE_KEY) as WorkMode | null;
    if (saved && WORK_MODES.includes(saved)) return saved;
  } catch {
    /* ignore */
  }
  return "image";
}

function loadOptionsOpen(): boolean {
  try {
    const saved = localStorage.getItem(OPTIONS_OPEN_KEY);
    if (saved != null) return saved === "1";
  } catch {
    /* ignore */
  }
  return true;
}

function loadRatios(): string[] {
  try {
    const saved = localStorage.getItem(RATIO_STORAGE_KEY);
    if (saved) {
      const parsed = JSON.parse(saved) as unknown;
      if (Array.isArray(parsed)) {
        return parsed.filter((x): x is string => typeof x === "string");
      }
    }
  } catch {
    /* ignore */
  }
  return [];
}

const inputDir = ref("");
const outputDir = ref("");
const allInputFiles = ref<FileInfo[]>([]);
const outputFiles = ref<FileInfo[]>([]);
const selectedInputPath = ref("");
const selectedOutputPath = ref("");
/** 勾选的输入文件路径 */
const checkedPaths = ref<Set<string>>(new Set());

const workMode = ref<WorkMode>(loadWorkMode());
const options = ref<ProcessOptions>(defaultProcessOptions());
const naming = ref<NamingOptions>(defaultNamingOptions());

const isProcessing = ref(false);
const progress = ref<ProcessProgress | null>(null);
const resultMessage = ref("");
const uploadUrl = ref("");
const optionsOpen = ref(loadOptionsOpen());
const ratios = ref<string[]>(loadRatios());
const newRatio = ref("");
const ratioError = ref("");
const initialized = ref(false);
/** 异步初始化完成（目录扫描等），用于关闭首屏 loading */
const workspaceReady = ref(false);
const hwAccel = ref<HwAccelOptions | null>(null);
const { showToast } = useToast();

const files = computed(() => {
  switch (workMode.value) {
    case "image":
      return allInputFiles.value.filter((f) => f.file_type === "image");
    case "video":
      return allInputFiles.value.filter((f) => f.file_type === "video");
    case "ratio":
      return allInputFiles.value;
    case "crop":
    case "merge":
    case "join":
      return allInputFiles.value;
    default:
      return allInputFiles.value;
  }
});

const checkedFiles = computed(() =>
  files.value.filter((f) => checkedPaths.value.has(f.path)),
);

const imageCount = computed(
  () => allInputFiles.value.filter((f) => f.file_type === "image").length,
);
const videoCount = computed(
  () => allInputFiles.value.filter((f) => f.file_type === "video").length,
);

  const statusMessage = computed(() => {
  if (isProcessing.value && progress.value) {
    const pct =
      typeof progress.value.percent === "number" && Number.isFinite(progress.value.percent)
        ? ` ${Math.round(progress.value.percent)}%`
        : "";
    return `正在处理 ${progress.value.current}/${progress.value.total}${pct} · ${progress.value.current_file || progress.value.status || ""}`;
  }
  if (resultMessage.value) {
    const oneLine = resultMessage.value.replace(/\s+/g, " ").trim();
    return oneLine.length > 120 ? oneLine.slice(0, 120) + "…" : oneLine;
  }
  return "就绪";
});

const primaryActionLabel = computed(() => {
  if (isProcessing.value) return "处理中…";
  const n = checkedFiles.value.length;
  switch (workMode.value) {
    case "image":
      return `开始处理图片 (${n})`;
    case "video":
      return `开始处理视频 (${n})`;
    case "ratio":
      return `按比例裁剪 (${n}×${ratios.value.length || 0})`;
    case "crop":
      return "导出裁剪";
    case "merge":
      return "导出拼接";
    case "join":
      return "导出自定义拼接";
    default:
      return "开始处理";
  }
});

const canStart = computed(() => {
  if (isProcessing.value) return false;
  switch (workMode.value) {
    case "image":
    case "video":
      return checkedFiles.value.length > 0;
    case "ratio":
      return checkedFiles.value.length > 0 && ratios.value.length > 0;
    case "crop":
    case "merge":
    case "join":
      return true;
    default:
      return false;
  }
});

function isChecked(path: string) {
  return checkedPaths.value.has(path);
}

function toggleCheck(path: string) {
  const next = new Set(checkedPaths.value);
  if (next.has(path)) next.delete(path);
  else next.add(path);
  checkedPaths.value = next;
}

function setChecked(path: string, value: boolean) {
  const next = new Set(checkedPaths.value);
  if (value) next.add(path);
  else next.delete(path);
  checkedPaths.value = next;
}

function toggleCheckAllVisible() {
  const visible = files.value;
  const allOn = visible.length > 0 && visible.every((f) => checkedPaths.value.has(f.path));
  const next = new Set(checkedPaths.value);
  if (allOn) {
    for (const f of visible) next.delete(f.path);
  } else {
    for (const f of visible) next.add(f.path);
  }
  checkedPaths.value = next;
}

const allVisibleChecked = computed(() => {
  const visible = files.value;
  return visible.length > 0 && visible.every((f) => checkedPaths.value.has(f.path));
});

function pruneChecked() {
  const valid = new Set(allInputFiles.value.map((f) => f.path));
  const next = new Set<string>();
  for (const p of checkedPaths.value) {
    if (valid.has(p)) next.add(p);
  }
  checkedPaths.value = next;
}

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

async function saveNaming() {
  try {
    await invoke("set_naming_options", { naming: naming.value });
  } catch (e) {
    console.error("保存命名配置失败:", e);
  }
}

async function scanFiles() {
  if (!inputDir.value) return;
  try {
    allInputFiles.value = await invoke<FileInfo[]>("scan_input_files", {
      inputDir: inputDir.value,
    });
    pruneChecked();
  } catch (e) {
    console.error("扫描文件失败:", e);
    allInputFiles.value = [];
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
  if (
    workMode.value === "crop" ||
    workMode.value === "merge" ||
    workMode.value === "join"
  ) {
    return;
  }

  const paths = checkedFiles.value.map((f) => f.path);

  if (workMode.value === "image" || workMode.value === "video") {
    if (paths.length === 0) {
      resultMessage.value = "请先勾选要处理的文件";
      return;
    }
  }

  if (workMode.value === "ratio") {
    if (paths.length === 0) {
      resultMessage.value = "请先勾选要裁剪的文件";
      return;
    }
    if (ratios.value.length === 0) {
      resultMessage.value = "请至少添加一个比例";
      return;
    }
  }

  isProcessing.value = true;
  progress.value = null;
  resultMessage.value = "";

  try {
    if (workMode.value === "ratio") {
      const result = await invoke<string>("crop_by_ratios", {
        inputDir: inputDir.value,
        outputDir: outputDir.value,
        ratios: ratios.value,
        fileTypeFilter: null,
        filePaths: paths,
      });
      resultMessage.value = result;
      showToast(result || "处理完成", "success", 2000);
    } else {
      const filter = workMode.value === "image" ? "image" : "video";
      const result = await invoke<string>("process_files", {
        inputDir: inputDir.value,
        outputDir: outputDir.value,
        options: options.value,
        fileTypeFilter: filter,
        naming: naming.value,
        filePaths: paths,
      });
      resultMessage.value = result;
      showToast(result || "处理完成", "success", 2000);
    }
  } catch (e) {
    resultMessage.value = `处理失败: ${e}`;
    showToast(`处理失败: ${e}`, "error", 2000);
  }

  isProcessing.value = false;
  progress.value = null;
  await scanOutputFiles();
}

function beginWorkspaceJob(label = "处理中") {
  isProcessing.value = true;
  progress.value = {
    current: 1,
    total: 1,
    current_file: label,
    status: "processing",
    percent: 0,
  };
  resultMessage.value = "";
}

function endWorkspaceJob() {
  isProcessing.value = false;
  progress.value = null;
}

async function initWorkspace() {
  if (initialized.value) return;
  initialized.value = true;

  try {
    const dirs = await invoke<[string, string]>("get_custom_dirs");
    inputDir.value = dirs[0];
    outputDir.value = dirs[1];
    await scanAllFiles();

    try {
      naming.value = await invoke<NamingOptions>("get_naming_options");
    } catch {
      naming.value = defaultNamingOptions();
    }

    try {
      hwAccel.value = await invoke<HwAccelOptions>("get_hw_accel_options");
    } catch (e) {
      console.error("加载硬件加速配置失败:", e);
    }

    const url = await invoke<string>("start_upload_server", {
      inputDir: inputDir.value,
    });
    uploadUrl.value = url;
  } catch (e) {
    console.error("初始化失败:", e);
  }

  try {
    const applyProgress = (next: ProcessProgress) => {
      const prev = progress.value;
      if (
        prev &&
        typeof prev.percent === "number" &&
        typeof next.percent === "number" &&
        next.status !== "completed" &&
        next.percent + 0.05 < prev.percent &&
        next.current === prev.current &&
        next.total === prev.total
      ) {
        // 同文件内忽略回跳，保留更高进度
        progress.value = { ...next, percent: prev.percent };
        return;
      }
      progress.value = next;
    };
    await listen<ProcessProgress>("process-progress", (event) => {
      applyProgress(event.payload);
    });
    await listen<ProcessProgress>("crop-progress", (event) => {
      applyProgress(event.payload);
    });
  } catch (e) {
    console.error("注册进度监听失败:", e);
  } finally {
    workspaceReady.value = true;
  }
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

watch(workMode, (val) => {
  localStorage.setItem(WORK_MODE_KEY, val);
  selectedInputPath.value = "";
  checkedPaths.value = new Set();

  if (val === "image") {
    const imageExts = ["jpg", "png", "webp", "bmp", "tiff"];
    if (!imageExts.includes(options.value.target_format)) {
      options.value.target_format = "jpg";
    }
  } else if (val === "video") {
    const videoExts = ["mp4", "avi", "mkv", "mov", "webm", "flv"];
    if (!videoExts.includes(options.value.target_format)) {
      options.value.target_format = "mp4";
    }
  }
});

watch(
  naming,
  () => {
    saveNaming();
  },
  { deep: true },
);

async function setHwAccelMode(mode: string) {
  try {
    await invoke("set_hw_accel_mode", { mode });
    hwAccel.value = await invoke<HwAccelOptions>("get_hw_accel_options");
  } catch (e) {
    console.error("保存硬件加速配置失败:", e);
    showToast(`保存硬件加速失败: ${e}`, "error", 2000);
  }
}

async function refreshHwEncoders() {
  try {
    hwAccel.value = await invoke<HwAccelOptions>("detect_hw_encoders");
    showToast(
      `当前编码: H.264=${hwAccel.value.active_h264}, H.265=${hwAccel.value.active_hevc}`,
      "success",
      2500,
    );
  } catch (e) {
    console.error("探测硬件编码器失败:", e);
    showToast(`探测失败: ${e}`, "error", 2000);
  }
}

export function useWorkspace() {
  onMounted(() => {
    initWorkspace();
  });

  return {
    inputDir,
    outputDir,
    files,
    allInputFiles,
    outputFiles,
    selectedInputPath,
    selectedOutputPath,
    checkedPaths,
    checkedFiles,
    workMode,
    options,
    naming,
    isProcessing,
    progress,
    beginWorkspaceJob,
    endWorkspaceJob,
    resultMessage,
    uploadUrl,
    optionsOpen,
    workspaceReady,
    hwAccel,
    setHwAccelMode,
    refreshHwEncoders,
    ratios,
    newRatio,
    ratioError,
    imageCount,
    videoCount,
    statusMessage,
    primaryActionLabel,
    canStart,
    allVisibleChecked,
    isChecked,
    toggleCheck,
    setChecked,
    toggleCheckAllVisible,
    addRatio,
    removeRatio,
    saveCustomDirs,
    saveNaming,
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

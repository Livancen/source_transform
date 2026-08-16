import { ref, onMounted } from "vue";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

export type UpdatePhase =
  | "idle"
  | "checking"
  | "available"
  | "downloading"
  | "installing"
  | "done"
  | "error";

const visible = ref(false);
const phase = ref<UpdatePhase>("idle");
const currentVersion = ref("");
const latestVersion = ref("");
const releaseNotes = ref("");
const progressPct = ref(0);
const downloadedBytes = ref(0);
const totalBytes = ref(0);
const errorMessage = ref("");
const isUpdating = ref(false);

let pendingUpdate: Update | null = null;
let checkedOnce = false;

function formatBytes(n: number) {
  if (!n || n <= 0) return "0 B";
  const units = ["B", "KB", "MB", "GB"];
  let i = 0;
  let v = n;
  while (v >= 1024 && i < units.length - 1) {
    v /= 1024;
    i += 1;
  }
  return `${v.toFixed(i === 0 ? 0 : 1)} ${units[i]}`;
}

async function checkForUpdates(silent = true) {
  if (isUpdating.value) return;
  phase.value = "checking";
  errorMessage.value = "";
  if (!silent) {
    visible.value = true;
  }
  try {
    const update = await check();
    if (update) {
      pendingUpdate = update;
      currentVersion.value = update.currentVersion;
      latestVersion.value = update.version;
      releaseNotes.value = update.body || "";
      phase.value = "available";
      progressPct.value = 0;
      downloadedBytes.value = 0;
      totalBytes.value = 0;
      visible.value = true;
    } else {
      phase.value = "idle";
      pendingUpdate = null;
      if (!silent) {
        visible.value = true;
        errorMessage.value = "当前已是最新版本";
      }
    }
  } catch (e) {
    phase.value = "error";
    // 开发环境或无 release 时静默失败
    if (!silent) {
      errorMessage.value = `检查更新失败: ${e}`;
      visible.value = true;
    } else {
      phase.value = "idle";
      visible.value = false;
      console.warn("检查更新失败:", e);
    }
  }
}

function dismiss() {
  if (isUpdating.value) return;
  visible.value = false;
  if (phase.value === "error" || phase.value === "available") {
    phase.value = "idle";
  }
}

async function startUpdate() {
  if (!pendingUpdate || isUpdating.value) return;
  isUpdating.value = true;
  phase.value = "downloading";
  errorMessage.value = "";
  progressPct.value = 0;
  downloadedBytes.value = 0;
  totalBytes.value = 0;

  try {
    await pendingUpdate.downloadAndInstall((event) => {
      switch (event.event) {
        case "Started":
          totalBytes.value = event.data.contentLength ?? 0;
          downloadedBytes.value = 0;
          progressPct.value = 0;
          break;
        case "Progress":
          downloadedBytes.value += event.data.chunkLength;
          if (totalBytes.value > 0) {
            progressPct.value = Math.min(
              99,
              Math.round((downloadedBytes.value / totalBytes.value) * 100),
            );
          }
          break;
        case "Finished":
          progressPct.value = 100;
          phase.value = "installing";
          break;
      }
    });
    phase.value = "done";
    await relaunch();
  } catch (e) {
    phase.value = "error";
    errorMessage.value = `更新失败: ${e}`;
    isUpdating.value = false;
  }
}

export function useUpdater() {
  onMounted(() => {
    if (checkedOnce) return;
    checkedOnce = true;
    // 启动稍后再检查，避免拖慢首屏
    setTimeout(() => {
      checkForUpdates(true);
    }, 1500);
  });

  return {
    visible,
    phase,
    currentVersion,
    latestVersion,
    releaseNotes,
    progressPct,
    downloadedBytes,
    totalBytes,
    errorMessage,
    isUpdating,
    formatBytes,
    checkForUpdates,
    startUpdate,
    dismiss,
  };
}

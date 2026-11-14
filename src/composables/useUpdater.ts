import { ref, onMounted } from "vue";
import { invoke, Channel } from "@tauri-apps/api/core";
import { relaunch } from "@tauri-apps/plugin-process";

export type UpdatePhase =
  | "idle"
  | "checking"
  | "available"
  | "downloading"
  | "installing"
  | "done"
  | "error";

const MIRROR_KEY = "github_update_mirror";

const visible = ref(false);
const phase = ref<UpdatePhase>("idle");
const currentVersion = ref("");
const latestVersion = ref("");
const releaseNotes = ref("");
const progressPct = ref(0);
const downloadedBytes = ref(0);
const totalBytes = ref(0);
const errorMessage = ref("");
const downloadHint = ref("");
const isUpdating = ref(false);
const mirrorInput = ref(readSavedMirror());
const mirrorError = ref("");

let checkedOnce = false;

type DownloadEvent =
  | { event: "Started"; data: { contentLength?: number } }
  | { event: "Progress"; data: { chunkLength: number } }
  | { event: "Finished" }
  | { event: "Fallback" };

interface UpdateInfo {
  currentVersion: string;
  version: string;
  body?: string | null;
  mirrored: boolean;
}

export function readSavedMirror() {
  return (localStorage.getItem(MIRROR_KEY) || "").trim();
}

export function validateMirror(
  raw: string,
): { ok: true; value: string } | { ok: false; message: string } {
  const trimmed = raw.trim();
  if (!trimmed) return { ok: true, value: "" };
  const text = /^[a-zA-Z][a-zA-Z0-9+.-]*:\/\//.test(trimmed)
    ? trimmed
    : `https://${trimmed}`;
  let url: URL;
  try {
    url = new URL(text);
  } catch {
    return { ok: false, message: "加速域名不是合法网址" };
  }
  if (url.protocol !== "https:") {
    return { ok: false, message: "加速域名必须使用 https" };
  }
  if (!url.hostname) {
    return { ok: false, message: "加速域名缺少主机名" };
  }
  if (url.username || url.password) {
    return { ok: false, message: "加速域名不能包含账号密码" };
  }
  if (url.search || url.hash) {
    return { ok: false, message: "加速域名不能带参数或锚点" };
  }
  const host = url.hostname.toLowerCase();
  if (
    host === "github.com" ||
    host.endsWith(".github.com") ||
    host.endsWith("githubusercontent.com")
  ) {
    return { ok: false, message: "请填写加速站域名，不是 GitHub 的地址" };
  }
  const path = url.pathname.replace(/\/+$/, "");
  const port = url.port ? `:${url.port}` : "";
  const pathname = path === "/" ? "" : path;
  return { ok: true, value: `https://${url.hostname}${port}${pathname}` };
}

/** 校验并写入本地。返回值里的 value 为空表示不使用加速。 */
export function persistMirror():
  | { ok: true; value: string }
  | { ok: false; message: string } {
  const result = validateMirror(mirrorInput.value);
  if (!result.ok) {
    mirrorError.value = result.message;
    return result;
  }
  mirrorError.value = "";
  mirrorInput.value = result.value;
  if (result.value) localStorage.setItem(MIRROR_KEY, result.value);
  else localStorage.removeItem(MIRROR_KEY);
  return result;
}

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

function resetProgress() {
  progressPct.value = 0;
  downloadedBytes.value = 0;
  totalBytes.value = 0;
}

async function checkForUpdates(silent = true) {
  if (isUpdating.value) return;
  let mirrorValue = "";
  if (silent) {
    const saved = validateMirror(readSavedMirror());
    if (saved.ok) mirrorValue = saved.value;
    else console.warn(saved.message);
  } else {
    const mirror = persistMirror();
    if (!mirror.ok) {
      phase.value = "error";
      errorMessage.value = mirror.message;
      visible.value = true;
      return;
    }
    mirrorValue = mirror.value;
  }

  phase.value = "checking";
  errorMessage.value = "";
  downloadHint.value = "";
  if (!silent) {
    visible.value = true;
  }
  try {
    const update = await invoke<UpdateInfo | null>("check_app_update", {
      mirror: mirrorValue || null,
    });
    if (update) {
      currentVersion.value = update.currentVersion;
      latestVersion.value = update.version;
      releaseNotes.value = update.body || "";
      phase.value = "available";
      resetProgress();
      downloadHint.value = update.mirrored
        ? "将优先通过加速站下载，失败自动改用 GitHub"
        : "";
      visible.value = true;
    } else {
      phase.value = "idle";
      if (!silent) {
        visible.value = true;
        errorMessage.value = "当前已是最新版本";
      }
    }
  } catch (e) {
    phase.value = "error";
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
  if (isUpdating.value || phase.value !== "available") return;
  isUpdating.value = true;
  phase.value = "downloading";
  errorMessage.value = "";
  if (!downloadHint.value) downloadHint.value = "";
  resetProgress();

  try {
    const onEvent = new Channel<DownloadEvent>();
    onEvent.onmessage = (event) => {
      switch (event.event) {
        case "Started":
          phase.value = "downloading";
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
        case "Fallback":
          phase.value = "downloading";
          downloadHint.value = "加速站不可用，正在改用 GitHub 直连…";
          resetProgress();
          break;
      }
    };
    await invoke("download_and_install_app_update", { onEvent });
    phase.value = "done";
    await relaunch();
  } catch (e) {
    phase.value = "error";
    errorMessage.value = `更新失败: ${e}`;
    downloadHint.value = "";
    isUpdating.value = false;
  }
}

export function useUpdater() {
  onMounted(() => {
    if (checkedOnce) return;
    checkedOnce = true;
    mirrorInput.value = readSavedMirror();
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
    downloadHint,
    isUpdating,
    mirrorInput,
    mirrorError,
    formatBytes,
    persistMirror,
    checkForUpdates,
    startUpdate,
    dismiss,
  };
}

<script setup lang="ts">
import { computed, ref } from "vue";
import { useRouter } from "vue-router";
import { useWorkspace } from "../composables/useWorkspace";
import { useUpdater } from "../composables/useUpdater";
import { APP_VERSION, APP_NAME } from "../constants/app";

const router = useRouter();
const {
  inputDir,
  outputDir,
  uploadUrl,
  optionsOpen,
  naming,
  selectInputDir,
  selectOutputDir,
  openFolder,
  scanAllFiles,
} = useWorkspace();

const { phase, checkForUpdates } = useUpdater();
const checkingUpdate = ref(false);

function goHome() {
  router.push({ name: "home" });
}

async function onCheckUpdate() {
  if (checkingUpdate.value) return;
  checkingUpdate.value = true;
  try {
    await checkForUpdates(false);
  } finally {
    checkingUpdate.value = false;
  }
}

const namingPreview = computed(() => {
  const parts: string[] = [];
  if (naming.value.use_original_name) parts.push("原文件名");
  if (naming.value.use_timestamp) parts.push("时间戳");
  if (naming.value.use_datetime) parts.push("20260322_153045");
  if (naming.value.custom_text.trim())
    parts.push(naming.value.custom_text.trim());
  if (parts.length === 0) parts.push("原文件名");
  return parts.join("-") + ".ext";
});

const updateBtnLabel = computed(() => {
  if (checkingUpdate.value || phase.value === "checking") return "检查中…";
  if (phase.value === "downloading" || phase.value === "installing")
    return "更新中…";
  return "检查更新";
});
</script>

<template>
  <div class="app-shell bg-bg0 overflow-hidden">
    <header
      class="shrink-0 h-48px px-14px flex items-center gap-10px bg-bg1 border-b border-border"
    >
      <button class="tb-btn" type="button" @click="goHome">
        <svg
          class="w-15px h-15px"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
        >
          <path d="M19 12H5" />
          <polyline points="12 19 5 12 12 5" />
        </svg>
        返回
      </button>
      <div class="w-1px h-24px bg-border"></div>
      <div class="flex items-center gap-8px min-w-0">
        <div
          class="w-26px h-26px rounded-6px bg-secondary grid place-items-center color-white shrink-0"
        >
          <svg
            class="w-14px h-14px"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
          >
            <circle cx="12" cy="12" r="3" />
            <path
              d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"
            />
          </svg>
        </div>
        <div class="flex flex-col leading-tight min-w-0">
          <strong class="text-13px font-700">设置</strong>
          <span class="text-10px color-t3 truncate"
            >目录 · 命名 · 界面 · 更新</span
          >
        </div>
      </div>
    </header>

    <div class="flex-1 min-h-0 overflow-hidden p-12px">
      <div
        class="h-full max-w-1080px mx-auto grid grid-cols-2 grid-rows-[auto_1fr_auto] gap-10px"
      >
        <!-- 目录 -->
        <section
          class="bg-bg1 border border-border rounded-10px p-12px flex flex-col gap-8px min-h-0"
        >
          <h2 class="text-13px font-600 flex items-center gap-6px shrink-0">
            <span
              class="w-22px h-22px rounded-6px bg-secondary-soft color-secondary grid place-items-center"
            >
              <svg
                class="w-12px h-12px"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
              >
                <path
                  d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"
                />
              </svg>
            </span>
            目录设置
          </h2>
          <div class="flex flex-col gap-6px min-h-0">
            <div class="flex flex-col gap-4px">
              <span class="text-10px font-500 color-t3">输入目录</span>
              <div class="flex items-center gap-6px">
                <input
                  class="field flex-1 min-w-0 font-mono text-11px! h-28px!"
                  readonly
                  :value="inputDir"
                />
                <button
                  class="tb-btn h-28px! px-8px! text-11px!"
                  type="button"
                  @click="selectInputDir"
                >
                  选择
                </button>
                <button
                  class="tb-btn h-28px! px-8px! text-11px!"
                  type="button"
                  :disabled="!inputDir"
                  @click="openFolder(inputDir)"
                >
                  打开
                </button>
              </div>
            </div>
            <div class="flex flex-col gap-4px">
              <span class="text-10px font-500 color-t3">输出目录</span>
              <div class="flex items-center gap-6px">
                <input
                  class="field flex-1 min-w-0 font-mono text-11px! h-28px!"
                  readonly
                  :value="outputDir"
                />
                <button
                  class="tb-btn h-28px! px-8px! text-11px!"
                  type="button"
                  @click="selectOutputDir"
                >
                  选择
                </button>
                <button
                  class="tb-btn h-28px! px-8px! text-11px!"
                  type="button"
                  :disabled="!outputDir"
                  @click="openFolder(outputDir)"
                >
                  打开
                </button>
              </div>
            </div>
            <div
              v-if="uploadUrl"
              class="py-6px px-8px rounded-6px bg-secondary-soft border border-secondary/15 flex items-center gap-8px min-w-0"
            >
              <span class="text-10px color-t3 shrink-0">上传</span>
              <a
                class="color-secondary font-mono text-10px no-underline truncate hover:underline"
                :href="uploadUrl"
                target="_blank"
                rel="noreferrer"
                >{{ uploadUrl }}</a
              >
            </div>
          </div>
        </section>

        <!-- 界面 + 关于/更新 -->
        <section
          class="bg-bg1 border border-border rounded-10px p-12px flex flex-col gap-8px min-h-0"
        >
          <h2 class="text-13px font-600 flex items-center gap-6px shrink-0">
            <span
              class="w-22px h-22px rounded-6px bg-secondary-soft color-secondary grid place-items-center"
            >
              <svg
                class="w-12px h-12px"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
              >
                <rect x="3" y="3" width="18" height="18" rx="2" />
                <path d="M3 9h18" />
              </svg>
            </span>
            界面与更新
          </h2>

          <label
            class="flex items-center justify-between gap-10px p-10px rounded-8px bg-bg2 border border-border cursor-pointer"
          >
            <div class="min-w-0">
              <div class="text-12px font-500">默认展开处理选项</div>
              <div class="text-10px color-t3 mt-2px">进入首页时显示选项条</div>
            </div>
            <span class="switch shrink-0">
              <input type="checkbox" v-model="optionsOpen" />
              <span class="slider"></span>
            </span>
          </label>

          <div class="flex flex-wrap gap-8px">
            <button
              class="tb-btn h-30px! text-11px!"
              type="button"
              @click="scanAllFiles"
            >
              <svg
                class="w-14px h-14px"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
              >
                <polyline points="23 4 23 10 17 10" />
                <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10" />
              </svg>
              刷新文件列表
            </button>
            <button
              class="tb-btn tb-btn-success h-30px! text-11px!"
              type="button"
              :disabled="
                checkingUpdate ||
                phase === 'checking' ||
                phase === 'downloading' ||
                phase === 'installing'
              "
              @click="onCheckUpdate"
            >
              <svg
                class="w-14px h-14px"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
              >
                <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
                <polyline points="7 10 12 15 17 10" />
                <line x1="12" y1="15" x2="12" y2="3" />
              </svg>
              {{ updateBtnLabel }}
            </button>
          </div>

          <div class="mt-auto pt-4px text-11px color-t2 leading-relaxed">
            <div class="font-500 color-t1">{{ APP_NAME }}</div>
            <div class="mt-2px">
              版本 v{{ APP_VERSION }} · Tauri + Vue 3 + FFmpeg
            </div>
          </div>
        </section>

        <!-- 输出命名：跨两列 -->
        <section class="col-span-2">
          <div
            class="bg-bg1 border border-border rounded-10px p-12px flex flex-col gap-8px min-h-0 overflow-hidden"
          >
            <div class="flex items-start justify-between gap-12px shrink-0">
              <div>
                <h2 class="text-13px font-600 flex items-center gap-6px">
                  <span
                    class="w-22px h-22px rounded-6px bg-secondary-soft color-secondary grid place-items-center"
                  >
                    <svg
                      class="w-12px h-12px"
                      viewBox="0 0 24 24"
                      fill="none"
                      stroke="currentColor"
                      stroke-width="2"
                    >
                      <path
                        d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"
                      />
                      <polyline points="14 2 14 8 20 8" />
                    </svg>
                  </span>
                  输出文件命名
                </h2>
                <p class="text-10px color-t3 mt-4px">
                  适用于图片/视频批量、自定义裁剪、拼接；比例裁剪使用独立命名。
                </p>
              </div>
              <div
                class="py-6px px-10px rounded-6px bg-bg0 border border-border shrink-0 max-w-50%"
              >
                <span class="text-10px color-t3">预览 </span>
                <span class="text-11px font-mono color-t1">{{
                  namingPreview
                }}</span>
              </div>
            </div>

            <div class="grid grid-cols-4 gap-8px min-h-0 flex-1">
              <label
                class="flex justify-between items-center gap-8px p-10px rounded-8px bg-bg2 border border-border cursor-pointer min-h-0"
              >
                <div>
                  <div class="text-12px font-500">① 原名</div>
                  <div class="text-10px color-t3 mt-2px">保留原始文件名</div>
                </div>
                <span class="switch">
                  <input type="checkbox" v-model="naming.use_original_name" />
                  <span class="slider"></span>
                </span>
              </label>

              <label
                class="flex justify-between items-center gap-8px p-10px rounded-8px bg-bg2 border border-border cursor-pointer min-h-0"
              >
                <div>
                  <div class="text-12px font-500">② 时间戳</div>
                  <div class="text-10px color-t3 mt-2px">Unix 毫秒</div>
                </div>
                <span class="switch">
                  <input type="checkbox" v-model="naming.use_timestamp" />
                  <span class="slider"></span>
                </span>
              </label>

              <label
                class="flex justify-between items-center gap-8px p-10px rounded-8px bg-bg2 border border-border cursor-pointer min-h-0"
              >
                <div>
                  <div class="text-12px font-500">③ 标准时间</div>
                  <div class="text-10px color-t3 mt-2px">YYYYMMDD_HHMMSS</div>
                </div>
                <span class="switch">
                  <input type="checkbox" v-model="naming.use_datetime" />
                  <span class="slider"></span>
                </span>
              </label>

              <div
                class="flex flex-col gap-6px p-10px rounded-8px bg-bg2 border border-border min-h-0"
              >
                <div class="text-12px font-500">④ 自定义文本</div>
                <input
                  class="field w-full h-28px! text-11px!"
                  type="text"
                  v-model="naming.custom_text"
                  placeholder="可选"
                />
              </div>
            </div>
          </div>
        </section>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { useRouter } from "vue-router";
import { useWorkspace } from "../composables/useWorkspace";
import { APP_VERSION } from "../constants/app";

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

function goHome() {
  router.push({ name: "home" });
}

const namingPreview = computed(() => {
  const parts: string[] = [];
  if (naming.value.use_original_name) parts.push("原文件名");
  if (naming.value.use_timestamp) parts.push("时间戳");
  if (naming.value.use_datetime) parts.push("20260322_153045");
  if (naming.value.custom_text.trim()) parts.push(naming.value.custom_text.trim());
  if (parts.length === 0) parts.push("原文件名");
  return parts.join("-") + ".ext";
});
</script>

<template>
  <div class="app-shell bg-bg0">
    <header
      class="shrink-0 h-52px px-16px flex items-center gap-12px bg-bg1 border-b border-border"
    >
      <button class="tb-btn" type="button" @click="goHome">
        <svg class="w-15px h-15px" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M19 12H5"/><polyline points="12 19 5 12 12 5"/>
        </svg>
        返回
      </button>
      <div class="w-1px h-28px bg-border"></div>
      <div class="flex items-center gap-10px">
        <div
          class="w-30px h-30px rounded-8px bg-secondary grid place-items-center color-white"
        >
          <svg class="w-16px h-16px" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="3"/>
            <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/>
          </svg>
        </div>
        <div class="flex flex-col leading-tight">
          <strong class="text-13px font-700">设置</strong>
          <span class="text-10px color-t3">目录、命名与界面偏好</span>
        </div>
      </div>
    </header>

    <div class="flex-1 min-h-0 overflow-auto p-20px">
      <div class="max-w-720px mx-auto flex flex-col gap-16px">
        <!-- 目录 -->
        <section class="bg-bg1 border border-border rounded-12px p-18px">
          <h2 class="text-14px font-600 mb-14px flex items-center gap-8px">
            <span class="w-26px h-26px rounded-8px bg-secondary-soft color-secondary grid place-items-center">
              <svg class="w-14px h-14px" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
              </svg>
            </span>
            目录设置
          </h2>

          <div class="flex flex-col gap-12px">
            <div class="flex flex-col gap-6px">
              <span class="text-11px font-500 color-t3 uppercase tracking-wide">输入目录</span>
              <div class="flex items-center gap-8px">
                <input
                  class="field flex-1 min-w-0 font-mono text-11px!"
                  readonly
                  :value="inputDir"
                />
                <button class="tb-btn" type="button" @click="selectInputDir">选择</button>
                <button class="tb-btn" type="button" :disabled="!inputDir" @click="openFolder(inputDir)">打开</button>
              </div>
            </div>

            <div class="flex flex-col gap-6px">
              <span class="text-11px font-500 color-t3 uppercase tracking-wide">输出目录</span>
              <div class="flex items-center gap-8px">
                <input
                  class="field flex-1 min-w-0 font-mono text-11px!"
                  readonly
                  :value="outputDir"
                />
                <button class="tb-btn" type="button" @click="selectOutputDir">选择</button>
                <button class="tb-btn" type="button" :disabled="!outputDir" @click="openFolder(outputDir)">打开</button>
              </div>
            </div>

            <div
              v-if="uploadUrl"
              class="mt-4px py-10px px-12px rounded-6px bg-secondary-soft border border-secondary/15 flex items-center gap-10px"
            >
              <span class="text-11px color-t3 shrink-0">上传链接</span>
              <a
                class="color-secondary font-mono text-11px no-underline truncate hover:underline"
                :href="uploadUrl"
                target="_blank"
                rel="noreferrer"
              >{{ uploadUrl }}</a>
            </div>
          </div>
        </section>

        <!-- 输出命名 -->
        <section class="bg-bg1 border border-border rounded-12px p-18px">
          <h2 class="text-14px font-600 mb-14px flex items-center gap-8px">
            <span class="w-26px h-26px rounded-8px bg-secondary-soft color-secondary grid place-items-center">
              <svg class="w-14px h-14px" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
                <polyline points="14 2 14 8 20 8"/>
                <line x1="16" y1="13" x2="8" y2="13"/>
                <line x1="16" y1="17" x2="8" y2="17"/>
              </svg>
            </span>
            输出文件命名
          </h2>
          <p class="text-11px color-t3 mb-12px">
            以下规则对图片/视频批量、自定义裁剪、拼接生效；<strong class="color-t2">比例裁剪使用独立命名规范</strong>。
          </p>

          <div class="flex flex-col gap-8px">
            <label class="flex items-center justify-between gap-12px p-12px rounded-8px bg-bg2 border border-border cursor-pointer">
              <div>
                <div class="text-13px font-500">① 原名</div>
                <div class="text-11px color-t3 mt-2px">保留原始文件名（不含扩展名）</div>
              </div>
              <span class="switch">
                <input type="checkbox" v-model="naming.use_original_name" />
                <span class="slider"></span>
              </span>
            </label>

            <label class="flex items-center justify-between gap-12px p-12px rounded-8px bg-bg2 border border-border cursor-pointer">
              <div>
                <div class="text-13px font-500">② 时间戳</div>
                <div class="text-11px color-t3 mt-2px">Unix 毫秒时间戳</div>
              </div>
              <span class="switch">
                <input type="checkbox" v-model="naming.use_timestamp" />
                <span class="slider"></span>
              </span>
            </label>

            <label class="flex items-center justify-between gap-12px p-12px rounded-8px bg-bg2 border border-border cursor-pointer">
              <div>
                <div class="text-13px font-500">③ 标准时间</div>
                <div class="text-11px color-t3 mt-2px">格式 YYYYMMDD_HHMMSS</div>
              </div>
              <span class="switch">
                <input type="checkbox" v-model="naming.use_datetime" />
                <span class="slider"></span>
              </span>
            </label>

            <div class="p-12px rounded-8px bg-bg2 border border-border">
              <div class="text-13px font-500 mb-8px">④ 自定义文本</div>
              <input
                class="field w-full"
                type="text"
                v-model="naming.custom_text"
                placeholder="可选，将拼接到文件名中"
              />
            </div>
          </div>

          <div class="mt-12px py-10px px-12px rounded-6px bg-bg0 border border-border">
            <span class="text-11px color-t3">预览：</span>
            <span class="text-12px font-mono color-t1">{{ namingPreview }}</span>
          </div>
        </section>

        <!-- 界面 -->
        <section class="bg-bg1 border border-border rounded-12px p-18px">
          <h2 class="text-14px font-600 mb-14px flex items-center gap-8px">
            <span class="w-26px h-26px rounded-8px bg-secondary-soft color-secondary grid place-items-center">
              <svg class="w-14px h-14px" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <rect x="3" y="3" width="18" height="18" rx="2"/>
                <path d="M3 9h18"/>
              </svg>
            </span>
            界面偏好
          </h2>

          <label class="flex items-center justify-between gap-12px p-12px rounded-8px bg-bg2 border border-border cursor-pointer">
            <div>
              <div class="text-13px font-500">默认展开处理选项</div>
              <div class="text-11px color-t3 mt-2px">进入首页时是否显示选项条</div>
            </div>
            <span class="switch">
              <input type="checkbox" v-model="optionsOpen" />
              <span class="slider"></span>
            </span>
          </label>

          <div class="mt-12px">
            <button class="tb-btn" type="button" @click="scanAllFiles">
              <svg class="w-15px h-15px" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <polyline points="23 4 23 10 17 10"/>
                <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"/>
              </svg>
              立即刷新文件列表
            </button>
          </div>
        </section>

        <!-- 关于 -->
        <section class="bg-bg1 border border-border rounded-12px p-18px">
          <h2 class="text-14px font-600 mb-10px">关于</h2>
          <p class="text-12px color-t2 leading-relaxed">
            素材转换工具 v{{ APP_VERSION }} · 批量处理图片与视频 · 基于 Tauri + Vue 3 + FFmpeg
          </p>
        </section>
      </div>
    </div>
  </div>
</template>

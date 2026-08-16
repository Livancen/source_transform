<script setup lang="ts">
import { useUpdater } from "../composables/useUpdater";

const {
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
  startUpdate,
  dismiss,
} = useUpdater();
</script>

<template>
  <div
    v-if="visible"
    class="fixed inset-0 z-2000 flex items-center justify-center bg-black/50 p-20px"
    @click.self="!isUpdating && dismiss()"
  >
    <div
      class="w-440px max-w-full bg-bg1 rounded-12px border border-border shadow-xl overflow-hidden"
      @click.stop
    >
      <div class="px-18px py-14px border-b border-border flex items-center justify-between gap-12px">
        <div>
          <div class="text-14px font-600">
            {{
              phase === "available"
                ? "发现新版本"
                : phase === "downloading" || phase === "installing"
                  ? "正在更新"
                  : phase === "done"
                    ? "更新完成"
                    : phase === "error"
                      ? "更新提示"
                      : "检查更新"
            }}
          </div>
          <div v-if="phase === 'available'" class="text-11px color-t3 mt-4px">
            {{ currentVersion }} → {{ latestVersion }}
          </div>
        </div>
        <button
          v-if="!isUpdating"
          class="tb-btn text-16px! px-8px!"
          type="button"
          @click="dismiss"
        >
          ×
        </button>
      </div>

      <div class="px-18px py-16px flex flex-col gap-14px">
        <template v-if="phase === 'available'">
          <p class="text-13px color-t2 leading-relaxed">
            检测到新版本 <strong class="color-t1">{{ latestVersion }}</strong>，是否立即更新？
          </p>
          <div
            v-if="releaseNotes"
            class="max-h-140px overflow-auto p-10px rounded-8px bg-bg0 border border-border text-12px color-t2 whitespace-pre-wrap"
          >
            {{ releaseNotes }}
          </div>
          <div class="flex justify-end gap-8px pt-4px">
            <button class="tb-btn" type="button" @click="dismiss">稍后</button>
            <button class="tb-btn tb-btn-success" type="button" @click="startUpdate">
              立即更新
            </button>
          </div>
        </template>

        <template v-else-if="phase === 'downloading' || phase === 'installing'">
          <div class="text-13px color-t2">
            {{
              phase === "installing"
                ? "下载完成，正在安装…"
                : "正在下载更新包，请勿关闭应用"
            }}
          </div>
          <div class="h-8px rounded-full bg-bg0 border border-border overflow-hidden">
            <div
              class="h-full rounded-full bg-secondary transition-all duration-200"
              :style="{ width: progressPct + '%' }"
            ></div>
          </div>
          <div class="flex justify-between text-11px color-t3 font-mono">
            <span>{{ progressPct }}%</span>
            <span v-if="totalBytes > 0">
              {{ formatBytes(downloadedBytes) }} / {{ formatBytes(totalBytes) }}
            </span>
            <span v-else>{{ formatBytes(downloadedBytes) }}</span>
          </div>
        </template>

        <template v-else-if="phase === 'error' || errorMessage">
          <p class="text-13px color-t2 leading-relaxed">{{ errorMessage || "未知错误" }}</p>
          <div class="flex justify-end">
            <button class="tb-btn" type="button" @click="dismiss">关闭</button>
          </div>
        </template>

        <template v-else-if="phase === 'done'">
          <p class="text-13px color-t2">更新已安装，正在重启…</p>
        </template>
      </div>
    </div>
  </div>
</template>

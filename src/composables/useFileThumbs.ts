import { reactive, watch, type Ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { FileInfo } from "../types";

/** 懒加载缩略图缓存（path → dataURL） */
export function useFileThumbs(files: Ref<FileInfo[]>) {
  const thumbs = reactive<Record<string, string>>({});
  const loading = reactive<Record<string, boolean>>({});
  let queue: string[] = [];
  let running = 0;
  const MAX_CONCURRENT = 3;

  function enqueue(path: string, fileType: string) {
    if (thumbs[path] || loading[path]) return;
    loading[path] = true;
    queue.push(JSON.stringify({ path, fileType }));
    pump();
  }

  async function pump() {
    while (running < MAX_CONCURRENT && queue.length > 0) {
      const raw = queue.shift();
      if (!raw) break;
      const { path, fileType } = JSON.parse(raw) as { path: string; fileType: string };
      running += 1;
      try {
        const url = await invoke<string>("get_file_thumbnail", {
          path,
          fileType,
        });
        thumbs[path] = url;
      } catch {
        thumbs[path] = "";
      } finally {
        loading[path] = false;
        running -= 1;
        pump();
      }
    }
  }

  function ensureVisible(list: FileInfo[]) {
    for (const f of list) {
      enqueue(f.path, f.file_type);
    }
  }

  watch(
    files,
    (list) => {
      // 清理已不存在路径的缓存可省略，避免闪烁；只加载当前列表
      ensureVisible(list.slice(0, 40));
    },
    { immediate: true, deep: false },
  );

  return { thumbs, loading, ensureVisible, enqueue };
}

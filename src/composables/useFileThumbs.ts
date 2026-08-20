import { reactive } from "vue";
import { convertFileSrc, invoke } from "@tauri-apps/api/core";

/** 懒加载缩略图：由虚拟列表可见项触发，结果走磁盘缓存路径 */
export function useFileThumbs(_unused?: unknown) {
  const thumbs = reactive<Record<string, string>>({});
  const loading = reactive<Record<string, boolean>>({});
  const failed = reactive<Record<string, boolean>>({});
  let queue: string[] = [];
  let running = 0;
  const MAX_CONCURRENT = 3;

  function enqueue(path: string, fileType: string) {
    if (!path || thumbs[path] || loading[path] || failed[path]) return;
    loading[path] = true;
    queue.push(JSON.stringify({ path, fileType }));
    pump();
  }

  async function pump() {
    while (running < MAX_CONCURRENT && queue.length > 0) {
      const raw = queue.shift();
      if (!raw) break;
      const { path, fileType } = JSON.parse(raw) as {
        path: string;
        fileType: string;
      };
      running += 1;
      try {
        const diskPath = await invoke<string>("get_file_thumbnail", {
          path,
          fileType,
        });
        thumbs[path] = convertFileSrc(diskPath);
      } catch {
        failed[path] = true;
        thumbs[path] = "";
      } finally {
        loading[path] = false;
        running -= 1;
        pump();
      }
    }
  }

  return { thumbs, loading, failed, enqueue };
}

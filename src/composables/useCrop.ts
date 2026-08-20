import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { FileInfo, NamingOptions, CustomCropOptions } from "../types";

export function useCrop(
  onMessage: (msg: string) => void,
  hooks?: { onStart?: () => void; onEnd?: () => void },
) {
  const selectedFile = ref<FileInfo | null>(null);
  const cropFrameImage = ref("");
  const mediaWidth = ref(1920);
  const mediaHeight = ref(1080);
  const previewScale = ref(1);
  const cropX = ref(0);
  const cropY = ref(0);
  const cropWidth = ref(1280);
  const cropHeight = ref(720);
  const isExporting = ref(false);
  const isLoading = ref(false);

  const isDragging = ref(false);
  const isResizing = ref(false);
  const resizeHandle = ref("");
  const dragStartX = ref(0);
  const dragStartY = ref(0);
  const startCropX = ref(0);
  const startCropY = ref(0);
  const startCropW = ref(0);
  const startCropH = ref(0);

  /** 根据预览容器可用宽高，计算缩放比（裁剪坐标仍为素材像素） */
  function fitPreviewScale(containerW: number, containerH: number) {
    if (mediaWidth.value <= 0 || mediaHeight.value <= 0) {
      previewScale.value = 1;
      return;
    }
    const pad = 8;
    const availW = Math.max(40, containerW - pad);
    const availH = Math.max(40, containerH - pad);
    const scale = Math.min(
      1,
      availW / mediaWidth.value,
      availH / mediaHeight.value,
    );
    previewScale.value = Number.isFinite(scale) && scale > 0 ? scale : 1;
  }

  function initCropRect() {
    const mw = mediaWidth.value;
    const mh = mediaHeight.value;
    const preferW = Math.min(mw, Math.max(100, Math.floor(mw * 0.6)));
    const preferH = Math.min(mh, Math.max(100, Math.floor(mh * 0.6)));
    cropWidth.value = preferW;
    cropHeight.value = preferH;
    cropX.value = Math.floor((mw - preferW) / 2);
    cropY.value = Math.floor((mh - preferH) / 2);
  }

  async function loadFile(file: FileInfo) {
    selectedFile.value = file;
    isLoading.value = true;
    cropFrameImage.value = "";
    try {
      if (file.file_type === "video") {
        const dimensions = await invoke<[number, number]>("get_video_dimensions", {
          videoPath: file.path,
        });
        mediaWidth.value = dimensions[0];
        mediaHeight.value = dimensions[1];
        cropFrameImage.value = await invoke<string>("extract_video_frame", {
          videoPath: file.path,
        });
      } else {
        const dimensions = await invoke<[number, number]>("get_image_dimensions", {
          imagePath: file.path,
        });
        mediaWidth.value = dimensions[0];
        mediaHeight.value = dimensions[1];
        cropFrameImage.value = await invoke<string>("load_image_preview", {
          imagePath: file.path,
        });
      }

      initCropRect();
    } catch (e) {
      onMessage(`加载文件失败: ${e}`);
      selectedFile.value = null;
    } finally {
      isLoading.value = false;
    }
  }

  function clearFile() {
    selectedFile.value = null;
    cropFrameImage.value = "";
  }

  function startDrag(e: MouseEvent) {
    if (isResizing.value) return;
    isDragging.value = true;
    dragStartX.value = e.clientX;
    dragStartY.value = e.clientY;
    startCropX.value = cropX.value;
    startCropY.value = cropY.value;
    e.preventDefault();
  }

  function startResize(e: MouseEvent, handle: string) {
    isResizing.value = true;
    resizeHandle.value = handle;
    dragStartX.value = e.clientX;
    dragStartY.value = e.clientY;
    startCropX.value = cropX.value;
    startCropY.value = cropY.value;
    startCropW.value = cropWidth.value;
    startCropH.value = cropHeight.value;
    e.preventDefault();
    e.stopPropagation();
  }

  function handleMouseMove(e: MouseEvent) {
    if (!isDragging.value && !isResizing.value) return;

    const scale = previewScale.value || 1;
    const deltaX = Math.round((e.clientX - dragStartX.value) / scale);
    const deltaY = Math.round((e.clientY - dragStartY.value) / scale);

    if (isDragging.value) {
      let newX = startCropX.value + deltaX;
      let newY = startCropY.value + deltaY;
      newX = Math.max(0, Math.min(newX, mediaWidth.value - cropWidth.value));
      newY = Math.max(0, Math.min(newY, mediaHeight.value - cropHeight.value));
      cropX.value = newX;
      cropY.value = newY;
    } else if (isResizing.value) {
      let newX = startCropX.value;
      let newY = startCropY.value;
      let newW = startCropW.value;
      let newH = startCropH.value;
      const handle = resizeHandle.value;
      const minSize = Math.min(20, mediaWidth.value, mediaHeight.value);

      if (handle.includes("e")) {
        newW = Math.max(minSize, Math.min(startCropW.value + deltaX, mediaWidth.value - startCropX.value));
      }
      if (handle.includes("w")) {
        const maxDelta = startCropX.value;
        const clampedDelta = Math.max(-maxDelta, Math.min(deltaX, startCropW.value - minSize));
        newX = startCropX.value + clampedDelta;
        newW = startCropW.value - clampedDelta;
      }
      if (handle.includes("s")) {
        newH = Math.max(minSize, Math.min(startCropH.value + deltaY, mediaHeight.value - startCropY.value));
      }
      if (handle.includes("n")) {
        const maxDelta = startCropY.value;
        const clampedDelta = Math.max(-maxDelta, Math.min(deltaY, startCropH.value - minSize));
        newY = startCropY.value + clampedDelta;
        newH = startCropH.value - clampedDelta;
      }

      cropX.value = newX;
      cropY.value = newY;
      cropWidth.value = newW;
      cropHeight.value = newH;
    }
  }

  function handleMouseUp() {
    isDragging.value = false;
    isResizing.value = false;
  }

  async function exportCrop(outputDir: string, naming: NamingOptions) {
    if (!selectedFile.value) {
      onMessage("请先选择一个文件");
      return;
    }
    if (!outputDir) {
      onMessage("请先设置输出目录");
      return;
    }

    isExporting.value = true;
    hooks?.onStart?.();
    try {
      const options: CustomCropOptions = {
        input_path: selectedFile.value.path,
        output_dir: outputDir,
        crop_x: cropX.value,
        crop_y: cropY.value,
        crop_width: cropWidth.value,
        crop_height: cropHeight.value,
        naming,
      };
      const result = await invoke<string>("custom_crop", { options });
      onMessage(result);
    } catch (e) {
      onMessage(`裁剪失败: ${e}`);
    } finally {
      isExporting.value = false;
      hooks?.onEnd?.();
    }
  }

  return {
    selectedFile,
    cropFrameImage,
    mediaWidth,
    mediaHeight,
    previewScale,
    cropX,
    cropY,
    cropWidth,
    cropHeight,
    isExporting,
    isLoading,
    fitPreviewScale,
    loadFile,
    clearFile,
    startDrag,
    startResize,
    handleMouseMove,
    handleMouseUp,
    exportCrop,
  };
}

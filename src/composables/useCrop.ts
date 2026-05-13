import { ref, type Ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { ProcessOptions, FileInfo } from "../types";

export function useCrop(
  options: Ref<ProcessOptions>,
  videoFiles: Ref<FileInfo[]>,
  onError: (msg: string) => void
) {
  const cropPreviewVisible = ref(false);
  const cropPreviewVideo = ref<string>("");
  const cropFrameImage = ref<string>("");
  const cropVideoWidth = ref(1920);
  const cropVideoHeight = ref(1080);
  const previewScale = ref(1);
  const cropAreaRef = ref<HTMLDivElement | null>(null);
  const previewContainerRef = ref<HTMLDivElement | null>(null);

  // 拖拽状态
  const isDragging = ref(false);
  const isResizing = ref(false);
  const resizeHandle = ref("");
  const dragStartX = ref(0);
  const dragStartY = ref(0);
  const startCropX = ref(0);
  const startCropY = ref(0);
  const startCropW = ref(0);
  const startCropH = ref(0);

  async function openCropPreview() {
    if (videoFiles.value.length === 0) {
      onError("没有可预览的视频文件");
      return;
    }
    const firstVideo = videoFiles.value[0];
    cropPreviewVideo.value = firstVideo.path;

    try {
      const dimensions = await invoke<[number, number]>("get_video_dimensions", {
        videoPath: firstVideo.path,
      });
      cropVideoWidth.value = dimensions[0];
      cropVideoHeight.value = dimensions[1];

      const framePath = await invoke<string>("extract_video_frame", {
        videoPath: firstVideo.path,
      });
      cropFrameImage.value = framePath;

      const maxPreviewWidth = 600;
      previewScale.value = Math.min(1, maxPreviewWidth / cropVideoWidth.value);

      if (options.value.crop_width > cropVideoWidth.value) {
        options.value.crop_width = cropVideoWidth.value;
      }
      if (options.value.crop_height > cropVideoHeight.value) {
        options.value.crop_height = cropVideoHeight.value;
      }
      options.value.crop_x = Math.floor(
        (cropVideoWidth.value - options.value.crop_width) / 2
      );
      options.value.crop_y = Math.floor(
        (cropVideoHeight.value - options.value.crop_height) / 2
      );

      cropPreviewVisible.value = true;
    } catch (e) {
      onError(`获取视频信息失败: ${e}`);
    }
  }

  function closeCropPreview() {
    cropPreviewVisible.value = false;
  }

  function confirmCrop() {
    options.value.crop = true;
    cropPreviewVisible.value = false;
  }

  function startDrag(e: MouseEvent) {
    if (isResizing.value) return;
    isDragging.value = true;
    dragStartX.value = e.clientX;
    dragStartY.value = e.clientY;
    startCropX.value = options.value.crop_x;
    startCropY.value = options.value.crop_y;
    e.preventDefault();
  }

  function startResize(e: MouseEvent, handle: string) {
    isResizing.value = true;
    resizeHandle.value = handle;
    dragStartX.value = e.clientX;
    dragStartY.value = e.clientY;
    startCropX.value = options.value.crop_x;
    startCropY.value = options.value.crop_y;
    startCropW.value = options.value.crop_width;
    startCropH.value = options.value.crop_height;
    e.preventDefault();
    e.stopPropagation();
  }

  function handleMouseMove(e: MouseEvent) {
    if (!isDragging.value && !isResizing.value) return;

    const deltaX = Math.round(
      (e.clientX - dragStartX.value) / previewScale.value
    );
    const deltaY = Math.round(
      (e.clientY - dragStartY.value) / previewScale.value
    );

    if (isDragging.value) {
      let newX = startCropX.value + deltaX;
      let newY = startCropY.value + deltaY;

      newX = Math.max(
        0,
        Math.min(newX, cropVideoWidth.value - options.value.crop_width)
      );
      newY = Math.max(
        0,
        Math.min(newY, cropVideoHeight.value - options.value.crop_height)
      );

      options.value.crop_x = newX;
      options.value.crop_y = newY;
    } else if (isResizing.value) {
      let newX = startCropX.value;
      let newY = startCropY.value;
      let newW = startCropW.value;
      let newH = startCropH.value;

      const handle = resizeHandle.value;

      if (handle.includes("e")) {
        newW = Math.max(
          100,
          Math.min(
            startCropW.value + deltaX,
            cropVideoWidth.value - startCropX.value
          )
        );
      }
      if (handle.includes("w")) {
        const maxDelta = startCropX.value;
        const clampedDelta = Math.max(
          -maxDelta,
          Math.min(deltaX, startCropW.value - 100)
        );
        newX = startCropX.value + clampedDelta;
        newW = startCropW.value - clampedDelta;
      }
      if (handle.includes("s")) {
        newH = Math.max(
          100,
          Math.min(
            startCropH.value + deltaY,
            cropVideoHeight.value - startCropY.value
          )
        );
      }
      if (handle.includes("n")) {
        const maxDelta = startCropY.value;
        const clampedDelta = Math.max(
          -maxDelta,
          Math.min(deltaY, startCropH.value - 100)
        );
        newY = startCropY.value + clampedDelta;
        newH = startCropH.value - clampedDelta;
      }

      options.value.crop_x = newX;
      options.value.crop_y = newY;
      options.value.crop_width = newW;
      options.value.crop_height = newH;
    }
  }

  function handleMouseUp() {
    isDragging.value = false;
    isResizing.value = false;
  }

  return {
    cropPreviewVisible,
    cropPreviewVideo,
    cropFrameImage,
    cropVideoWidth,
    cropVideoHeight,
    previewScale,
    cropAreaRef,
    previewContainerRef,
    isDragging,
    isResizing,
    openCropPreview,
    closeCropPreview,
    confirmCrop,
    startDrag,
    startResize,
    handleMouseMove,
    handleMouseUp,
  };
}

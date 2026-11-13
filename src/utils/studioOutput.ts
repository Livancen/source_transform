import { watch, ref, type Ref } from "vue";
import type { RatioImageOutput, RatioVideoOutput } from "../types";
import { defaultRatioOutputOptions } from "../types";

const VIDEO_EXTS = ["mp4", "m4v", "mov", "mkv", "avi", "webm", "wmv", "flv"];

export function defaultStudioImage(): RatioImageOutput {
  return { ...defaultRatioOutputOptions().image };
}

/** 拼接 / 自定义拼接原来固定 30fps、High、4.0、静音。 */
export function defaultStudioVideo(): RatioVideoOutput {
  return {
    ...defaultRatioOutputOptions().video,
    change_framerate: true,
    target_framerate: 30,
    set_level: true,
    target_profile: "high",
    target_level: "4.0",
    mute: true,
    convert_mp4: true,
    convert_h265_to_h264: true,
    compress: false,
  };
}

export function useStudioOutput(key: string, videoInit: RatioVideoOutput) {
  const image = ref(defaultStudioImage());
  const video = ref({ ...videoInit });
  try {
    const raw = localStorage.getItem(key);
    if (raw) {
      const parsed = JSON.parse(raw) as {
        image?: Partial<RatioImageOutput>;
        video?: Partial<RatioVideoOutput>;
      };
      if (parsed.image) Object.assign(image.value, parsed.image);
      if (parsed.video) Object.assign(video.value, parsed.video);
    }
  } catch {
    /* ignore */
  }
  watch(
    [image, video],
    () => {
      localStorage.setItem(
        key,
        JSON.stringify({ image: image.value, video: video.value }),
      );
    },
    { deep: true },
  );
  return { image, video };
}

export function outputExt(
  kind: "image" | "video",
  image: RatioImageOutput,
  video: RatioVideoOutput,
  sourcePath = "",
) {
  if (kind === "image") {
    return image.convert_format ? image.target_format || "png" : "png";
  }
  if (video.convert_mp4) return "mp4";
  const ext = sourcePath.split(".").pop()?.toLowerCase() || "mp4";
  return VIDEO_EXTS.includes(ext) ? ext : "mp4";
}

export type StudioOutput = {
  image: Ref<RatioImageOutput>;
  video: Ref<RatioVideoOutput>;
};

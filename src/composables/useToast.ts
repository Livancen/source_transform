import { ref } from "vue";

const visible = ref(false);
const message = ref("");
const toastType = ref<"success" | "error" | "info">("success");

let hideTimer: ReturnType<typeof setTimeout> | null = null;

function showToast(
  msg: string,
  type: "success" | "error" | "info" = "success",
  durationMs = 2000,
) {
  if (hideTimer) {
    clearTimeout(hideTimer);
    hideTimer = null;
  }
  message.value = msg;
  toastType.value = type;
  visible.value = true;
  hideTimer = setTimeout(() => {
    visible.value = false;
    hideTimer = null;
  }, durationMs);
}

function hideToast() {
  if (hideTimer) {
    clearTimeout(hideTimer);
    hideTimer = null;
  }
  visible.value = false;
}

export function useToast() {
  return {
    visible,
    message,
    toastType,
    showToast,
    hideToast,
  };
}

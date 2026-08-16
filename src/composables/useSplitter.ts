import { ref, onMounted, onUnmounted } from "vue";

export function useSplitter(minPx = 220) {
  const leftPanePct = ref(50);
  const isDragging = ref(false);
  const panesRef = ref<HTMLElement | null>(null);

  function onSplitterDown(e: MouseEvent) {
    isDragging.value = true;
    e.preventDefault();
  }

  function onSplitterMove(e: MouseEvent) {
    if (!isDragging.value || !panesRef.value) return;
    const rect = panesRef.value.getBoundingClientRect();
    let x = e.clientX - rect.left;
    const max = rect.width - minPx;
    x = Math.max(minPx, Math.min(max, x));
    leftPanePct.value = (x / rect.width) * 100;
  }

  function onSplitterUp() {
    isDragging.value = false;
  }

  onMounted(() => {
    window.addEventListener("mousemove", onSplitterMove);
    window.addEventListener("mouseup", onSplitterUp);
  });

  onUnmounted(() => {
    window.removeEventListener("mousemove", onSplitterMove);
    window.removeEventListener("mouseup", onSplitterUp);
  });

  return {
    leftPanePct,
    isDragging,
    panesRef,
    onSplitterDown,
  };
}

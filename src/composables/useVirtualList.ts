import { computed, onBeforeUnmount, onMounted, ref, watch, type Ref } from "vue";

export function useVirtualList<T>(
  items: Ref<T[]>,
  options: {
    itemHeight: number;
    overscan?: number;
  },
) {
  const containerRef = ref<HTMLElement | null>(null);
  const scrollTop = ref(0);
  const viewportHeight = ref(0);
  const overscan = options.overscan ?? 6;
  const itemHeight = options.itemHeight;

  const totalHeight = computed(() => items.value.length * itemHeight);

  const startIndex = computed(() => {
    const raw = Math.floor(scrollTop.value / itemHeight) - overscan;
    return Math.max(0, raw);
  });

  const endIndex = computed(() => {
    const visible = Math.ceil(viewportHeight.value / itemHeight) + overscan * 2;
    return Math.min(items.value.length, startIndex.value + Math.max(visible, 1));
  });

  const visibleItems = computed(() => {
    const start = startIndex.value;
    const end = endIndex.value;
    return items.value.slice(start, end).map((item, i) => ({
      item,
      index: start + i,
      top: (start + i) * itemHeight,
    }));
  });

  const offsetY = computed(() => startIndex.value * itemHeight);

  function onScroll() {
    const el = containerRef.value;
    if (!el) return;
    scrollTop.value = el.scrollTop;
  }

  function measure() {
    const el = containerRef.value;
    if (!el) return;
    viewportHeight.value = el.clientHeight;
    scrollTop.value = el.scrollTop;
  }

  let ro: ResizeObserver | null = null;

  onMounted(() => {
    measure();
    const el = containerRef.value;
    if (el && typeof ResizeObserver !== "undefined") {
      ro = new ResizeObserver(() => measure());
      ro.observe(el);
    }
  });

  onBeforeUnmount(() => {
    ro?.disconnect();
    ro = null;
  });

  watch(
    () => items.value.length,
    () => {
      // 列表变短时避免 scrollTop 越界
      const el = containerRef.value;
      if (!el) return;
      const max = Math.max(0, items.value.length * itemHeight - el.clientHeight);
      if (el.scrollTop > max) el.scrollTop = max;
      measure();
    },
  );

  return {
    containerRef,
    totalHeight,
    visibleItems,
    offsetY,
    startIndex,
    endIndex,
    onScroll,
    measure,
    itemHeight,
  };
}

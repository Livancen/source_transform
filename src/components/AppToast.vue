<script setup lang="ts">
import { useToast } from "../composables/useToast";

const { visible, message, toastType } = useToast();
</script>

<template>
  <Transition name="app-toast">
    <div
      v-if="visible"
      class="fixed top-20px left-1/2 z-3000 -translate-x-1/2 max-w-90vw pointer-events-none"
      role="status"
    >
      <div
        class="px-16px py-10px rounded-8px text-13px font-500 shadow-lg border whitespace-pre-wrap text-center"
        :class="{
          'bg-#e8f5e9 color-#1b5e20 border-#c8e6c9': toastType === 'success',
          'bg-#ffebee color-#b71c1c border-#ffcdd2': toastType === 'error',
          'bg-bg1 color-t1 border-border': toastType === 'info',
        }"
      >
        {{ message }}
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.app-toast-enter-active,
.app-toast-leave-active {
  transition:
    opacity 0.2s ease,
    transform 0.2s ease;
}
.app-toast-enter-from,
.app-toast-leave-to {
  opacity: 0;
  transform: translate(-50%, -8px);
}
.app-toast-enter-to,
.app-toast-leave-from {
  opacity: 1;
  transform: translate(-50%, 0);
}
</style>

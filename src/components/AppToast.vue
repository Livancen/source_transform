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
        class="px-16px py-10px rounded-4px text-13px font-500 shadow-fluent-md border whitespace-pre-wrap text-center"
        :class="{
          'bg-success-soft color-success border-success/25': toastType === 'success',
          'bg-danger-soft color-danger border-danger/25': toastType === 'error',
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

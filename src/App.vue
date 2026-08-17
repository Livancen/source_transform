<script setup lang="ts">
import { watch } from "vue";
import { RouterView, useRoute } from "vue-router";
import UpdateModal from "./components/UpdateModal.vue";
import AppToast from "./components/AppToast.vue";
import { useUpdater } from "./composables/useUpdater";
import { useWorkspace } from "./composables/useWorkspace";

const route = useRoute();
const { workspaceReady } = useWorkspace();
// 挂载时自动检查更新
useUpdater();

function dismissBootScreen() {
  const el = document.getElementById("boot-screen");
  if (!el) return;
  el.classList.add("is-done");
  window.setTimeout(() => el.remove(), 220);
}

watch(
  workspaceReady,
  (ready) => {
    if (ready) dismissBootScreen();
  },
  { immediate: true },
);
</script>

<template>
  <RouterView v-slot="{ Component }">
    <KeepAlive>
      <component :is="Component" :key="String(route.name || route.path)" />
    </KeepAlive>
  </RouterView>
  <UpdateModal />
  <AppToast />
</template>

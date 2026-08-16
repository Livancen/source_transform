<script setup lang="ts">
import { RouterView, useRoute } from "vue-router";
import UpdateModal from "./components/UpdateModal.vue";
import AppToast from "./components/AppToast.vue";
import { useUpdater } from "./composables/useUpdater";

const route = useRoute();
// 挂载时自动检查更新
useUpdater();
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

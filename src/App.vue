<script setup lang="ts">
import { RouterView, useRoute } from "vue-router";

const route = useRoute();
</script>

<template>
  <RouterView v-slot="{ Component }">
    <KeepAlive>
      <component :is="Component" :key="String(route.name || route.path)" />
    </KeepAlive>
  </RouterView>
</template>

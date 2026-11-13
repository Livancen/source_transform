<script setup lang="ts">
import type { RatioImageOutput, RatioVideoOutput } from "../types";
import ImageOutputChips from "./ImageOutputChips.vue";
import VideoOutputChips from "./VideoOutputChips.vue";

defineProps<{
  image?: RatioImageOutput;
  video?: RatioVideoOutput;
}>();
</script>

<template>
  <div class="flex flex-col gap-8px">
    <ImageOutputChips v-if="image" :image="image" />
    <VideoOutputChips v-if="video" :video="video" />
  </div>
</template>

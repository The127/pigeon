<script setup lang="ts">
import { ref } from 'vue'
import AppSidebar from './AppSidebar.vue'

const collapsed = ref(false)
</script>

<template>
  <div class="flex h-screen">
    <AppSidebar v-model:collapsed="collapsed" />
    <main class="flex-1 overflow-auto p-6">
      <slot />
    </main>
  </div>
</template>

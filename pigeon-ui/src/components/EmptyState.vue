<script setup lang="ts">
import type { Component } from 'vue'

defineProps<{
  icon?: Component
  title: string
  description?: string
}>()
</script>

<template>
  <div class="flex flex-col items-center justify-center rounded-lg border border-dashed p-12 text-center">
    <component
      :is="icon"
      v-if="icon"
      class="mb-4 h-10 w-10 text-muted-foreground"
    />
    <h2 class="text-lg font-medium">{{ title }}</h2>
    <p v-if="description" class="mb-4 text-sm text-muted-foreground">
      {{ description }}
    </p>
    <slot />
  </div>
</template>

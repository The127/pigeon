<script setup lang="ts">
import { AlertCircle } from 'lucide-vue-next'

defineProps<{
  message: string
}>()
</script>

<template>
  <div class="flex items-center gap-2 rounded-md border border-destructive/20 bg-destructive/5 p-3 text-sm text-destructive">
    <AlertCircle class="h-4 w-4 shrink-0" />
    {{ message }}
  </div>
</template>

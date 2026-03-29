<script setup lang="ts">
import { Loader2 } from 'lucide-vue-next'

defineProps<{
  message?: string
}>()
</script>

<template>
  <div class="flex items-center gap-2 text-sm text-muted-foreground">
    <Loader2 class="h-4 w-4 animate-spin" />
    {{ message || 'Loading...' }}
  </div>
</template>

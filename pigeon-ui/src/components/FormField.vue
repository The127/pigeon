<script setup lang="ts">
import { Label } from '@/components/ui/label'

defineProps<{
  label: string
  htmlFor?: string
  description?: string
  error?: string
}>()
</script>

<template>
  <div class="space-y-2">
    <Label :for="htmlFor">{{ label }}</Label>
    <slot />
    <p v-if="description && !error" class="text-xs text-muted-foreground">
      {{ description }}
    </p>
    <p v-if="error" class="text-xs text-destructive">
      {{ error }}
    </p>
  </div>
</template>

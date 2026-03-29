<script setup lang="ts">
import { Label } from '@/components/ui/label'

defineProps<{
  label: string
  htmlFor?: string
  description?: string
  error?: string
  required?: boolean
}>()
</script>

<template>
  <div class="space-y-2">
    <Label :for="htmlFor">
      {{ label }}
      <span v-if="required" class="text-destructive ml-0.5">*</span>
    </Label>
    <slot />
    <p v-if="description && !error" class="text-xs text-muted-foreground">
      {{ description }}
    </p>
    <p v-if="error" class="text-xs text-destructive">
      {{ error }}
    </p>
  </div>
</template>

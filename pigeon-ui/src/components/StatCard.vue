<script setup lang="ts">
import type { Component } from 'vue'
import { cn } from '@/lib/utils'

defineProps<{
  title: string
  value: string | number
  description?: string
  icon?: Component
  variant?: 'default' | 'success' | 'destructive' | 'warning'
}>()

const variantClasses = {
  default: 'text-foreground',
  success: 'text-emerald-600',
  destructive: 'text-destructive',
  warning: 'text-amber-600',
}
</script>

<template>
  <div class="rounded-lg border bg-card p-4">
    <div class="flex items-center justify-between">
      <p class="text-sm font-medium text-muted-foreground">{{ title }}</p>
      <component
        :is="icon"
        v-if="icon"
        class="h-4 w-4 text-muted-foreground"
      />
    </div>
    <p :class="cn('mt-1 text-2xl font-semibold', variantClasses[variant || 'default'])">
      {{ value }}
    </p>
    <p v-if="description" class="mt-1 text-xs text-muted-foreground">
      {{ description }}
    </p>
  </div>
</template>

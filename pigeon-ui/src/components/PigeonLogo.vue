<script setup lang="ts">
import { cn } from '@/lib/utils'

const props = withDefaults(defineProps<{
  size?: 'sm' | 'md' | 'lg'
}>(), {
  size: 'md',
})

const sizeClasses = {
  sm: 'h-8 w-8',
  md: 'h-10 w-10',
  lg: 'h-12 w-12',
}
</script>

<template>
  <div :class="cn('flex shrink-0 items-center justify-center rounded-lg bg-primary', sizeClasses[props.size])">
    <svg
      viewBox="-1 4 26 22"
      fill="currentColor"
      class="text-primary-foreground"
      :class="{
        'h-5 w-5': props.size === 'sm',
        'h-6 w-6': props.size === 'md',
        'h-7 w-7': props.size === 'lg',
      }"
    >
      <path
        d="M24 8
           C22 7, 19 5, 16 5.5
           C12 6, 9 8, 7 11
           C5 14, 3 17, 1 22
           L0 25
           C1 24, 2 22, 4 20.5
           C6 18.5, 8 17.5, 10 17.5
           C12.5 17.5, 14 16, 15 14
           C16 12, 16.5 10, 17.5 9
           C18.5 8, 20 7.5, 21 7
           C22 7, 23 7.5, 24 8z"
      />
    </svg>
  </div>
</template>

<script setup lang="ts">
import { provideToast } from '@/composables/useToast'
import { X, CheckCircle2, AlertCircle, Info, AlertTriangle } from 'lucide-vue-next'

const { toasts, isHovered, remove } = provideToast()

const icons = {
  success: CheckCircle2,
  error: AlertCircle,
  info: Info,
  warning: AlertTriangle,
}

const styles = {
  success: 'bg-emerald-600 text-white',
  error: 'bg-destructive text-destructive-foreground',
  info: 'bg-card text-card-foreground border border-border',
  warning: 'bg-amber-500 text-white',
}

const progressStyles = {
  success: 'bg-emerald-300/40',
  error: 'bg-white/20',
  info: 'bg-primary/20',
  warning: 'bg-amber-200/40',
}
</script>

<template>
  <slot />

  <div
    class="fixed bottom-4 right-4 z-50 flex flex-col items-end"
    @mouseenter="isHovered = true"
    @mouseleave="isHovered = false"
  >
    <Transition
      v-for="(toast, idx) in toasts"
      :key="toast.id"
      name="toast"
    >
      <div
        :class="[
          'absolute right-0 w-80 rounded-lg px-4 py-3 shadow-lg transition-all duration-200',
          styles[toast.type],
        ]"
        :style="{
          zIndex: toasts.length - idx,
          bottom: isHovered ? `${idx * 64}px` : `${idx * 8}px`,
          opacity: isHovered || idx === 0 ? 1 : 0.85,
          scale: isHovered || idx === 0 ? '1' : `${1 - idx * 0.03}`,
          filter: isHovered || idx === 0 ? 'none' : `brightness(${1 - idx * 0.05})`,
        }"
      >
        <div class="flex items-start gap-2.5">
          <component
            :is="icons[toast.type]"
            class="mt-0.5 h-4 w-4 shrink-0"
          />
          <span class="flex-1 text-sm leading-snug">{{ toast.text }}</span>
          <button
            class="shrink-0 rounded-sm p-0.5 opacity-60 transition-opacity hover:opacity-100"
            @click="remove(toast.id)"
          >
            <X class="h-3.5 w-3.5" />
          </button>
        </div>

        <!-- Progress bar -->
        <div class="absolute bottom-0 left-0 h-0.5 w-full overflow-hidden rounded-b-lg">
          <div
            :class="progressStyles[toast.type]"
            class="h-full transition-none"
            :style="{ width: `${(toast.remaining / toast.timeout) * 100}%` }"
          />
        </div>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.toast-enter-active {
  transition: all 0.3s ease-out;
}
.toast-leave-active {
  transition: all 0.2s ease-in;
}
.toast-enter-from {
  transform: translateX(100%);
  opacity: 0;
}
.toast-leave-to {
  transform: translateX(100%);
  opacity: 0;
}
</style>

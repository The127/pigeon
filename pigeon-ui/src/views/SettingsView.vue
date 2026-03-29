<script setup lang="ts">
import { useTheme, type ThemeMode } from '@/composables/useTheme'
import { Button } from '@/components/ui/button'
import PageHeader from '@/components/PageHeader.vue'
import { Sun, Moon, Monitor } from 'lucide-vue-next'

const { mode, setMode } = useTheme()

const themeOptions: { value: ThemeMode; icon: typeof Sun; label: string; description: string }[] = [
  { value: 'auto', icon: Monitor, label: 'System', description: 'Follow your operating system preference' },
  { value: 'light', icon: Sun, label: 'Light', description: 'Light background with dark text' },
  { value: 'dark', icon: Moon, label: 'Dark', description: 'Dark background with light text' },
]
</script>

<template>
  <div class="space-y-8">
    <PageHeader
      title="Settings"
      description="Manage your preferences."
    />

    <div class="max-w-lg space-y-4">
      <h3 class="text-sm font-medium">Appearance</h3>
      <div class="grid gap-3">
        <button
          v-for="opt in themeOptions"
          :key="opt.value"
          class="flex items-center gap-4 rounded-lg border p-4 text-left transition-colors"
          :class="mode === opt.value
            ? 'border-primary bg-primary/5'
            : 'border-border hover:border-primary/40'"
          @click="setMode(opt.value)"
        >
          <div
            class="flex h-10 w-10 shrink-0 items-center justify-center rounded-md"
            :class="mode === opt.value ? 'bg-primary text-primary-foreground' : 'bg-muted text-muted-foreground'"
          >
            <component :is="opt.icon" class="h-5 w-5" />
          </div>
          <div>
            <p class="text-sm font-medium">{{ opt.label }}</p>
            <p class="text-xs text-muted-foreground">{{ opt.description }}</p>
          </div>
        </button>
      </div>
    </div>
  </div>
</template>

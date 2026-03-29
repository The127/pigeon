<script setup lang="ts">
import { useTheme, type ThemeMode, type AccentColor, type ColorblindMode } from '@/composables/useTheme'
import PageHeader from '@/components/PageHeader.vue'
import FilterSelect from '@/components/FilterSelect.vue'
import { Sun, Moon, Monitor } from 'lucide-vue-next'

const { mode, accent, colorblind, setMode, setAccent, setColorblind } = useTheme()

const themeOptions: { value: ThemeMode; icon: typeof Sun; label: string; description: string }[] = [
  { value: 'auto', icon: Monitor, label: 'System', description: 'Follow your operating system preference' },
  { value: 'light', icon: Sun, label: 'Light', description: 'Light background with dark text' },
  { value: 'dark', icon: Moon, label: 'Dark', description: 'Dark background with light text' },
]

const colorblindOptions: { value: string; label: string }[] = [
  { value: 'none', label: 'None' },
  { value: 'deuteranopia', label: 'Deuteranopia (reduced green)' },
  { value: 'protanopia', label: 'Protanopia (reduced red)' },
  { value: 'tritanopia', label: 'Tritanopia (reduced blue)' },
]

const accentOptions: { value: AccentColor; label: string; color: string }[] = [
  { value: 'amber', label: 'Amber', color: 'bg-amber-500' },
  { value: 'teal', label: 'Teal', color: 'bg-teal-500' },
  { value: 'indigo', label: 'Indigo', color: 'bg-indigo-500' },
  { value: 'rose', label: 'Rose', color: 'bg-rose-500' },
  { value: 'emerald', label: 'Emerald', color: 'bg-emerald-500' },
]
</script>

<template>
  <div class="space-y-8">
    <PageHeader
      title="Settings"
      description="Manage your preferences."
    />

    <div class="max-w-lg space-y-6">
      <!-- Appearance mode -->
      <div class="space-y-3">
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

      <!-- Accent color -->
      <div class="space-y-3">
        <h3 class="text-sm font-medium">Accent Color</h3>
        <div class="flex gap-3">
          <button
            v-for="opt in accentOptions"
            :key="opt.value"
            class="group flex flex-col items-center gap-2"
            @click="setAccent(opt.value)"
          >
            <div
              class="flex h-10 w-10 items-center justify-center rounded-full transition-all"
              :class="[
                opt.color,
                accent === opt.value
                  ? 'ring-2 ring-offset-2 ring-offset-background ring-primary scale-110'
                  : 'opacity-70 hover:opacity-100 hover:scale-105',
              ]"
            />
            <span
              class="text-xs transition-colors"
              :class="accent === opt.value ? 'text-foreground font-medium' : 'text-muted-foreground'"
            >
              {{ opt.label }}
            </span>
          </button>
        </div>
      </div>

      <!-- Accessibility -->
      <div class="space-y-3">
        <h3 class="text-sm font-medium">Accessibility</h3>
        <div class="flex items-center gap-3">
          <FilterSelect
            :model-value="colorblind"
            placeholder="Color vision"
            :options="colorblindOptions"
            @update:model-value="(v: string) => setColorblind(v as ColorblindMode)"
          />
          <span class="text-xs text-muted-foreground">Adjusts alert and chart colors for color vision deficiencies.</span>
        </div>
      </div>
    </div>
  </div>
</template>

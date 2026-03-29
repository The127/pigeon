<script setup lang="ts">
import { useTheme, type ThemeMode, type AccentColor, type ColorblindMode } from '@/composables/useTheme'
import PageHeader from '@/components/PageHeader.vue'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Sun, Moon, Monitor } from 'lucide-vue-next'

const { mode, accent, colorblind, highContrast, dyslexiaFont, setMode, setAccent, setColorblind, setHighContrast, setDyslexiaFont } = useTheme()

const themeOptions: { value: ThemeMode; icon: typeof Sun; label: string; description: string }[] = [
  { value: 'auto', icon: Monitor, label: 'System', description: 'Follow your operating system preference' },
  { value: 'light', icon: Sun, label: 'Light', description: 'Light background with dark text' },
  { value: 'dark', icon: Moon, label: 'Dark', description: 'Dark background with light text' },
]

const colorblindOptions: { value: string; label: string }[] = [
  { value: 'none', label: 'Default (no adjustment)' },
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
      <div class="space-y-6">
        <h3 class="text-sm font-medium">Accessibility</h3>

        <!-- Color vision -->
        <div class="space-y-2">
          <p class="text-sm font-medium">Color Vision</p>
          <Select
            :model-value="colorblind"
            @update:model-value="(v: any) => setColorblind(v as ColorblindMode)"
          >
            <SelectTrigger class="w-auto min-w-[280px]">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem
                v-for="opt in colorblindOptions"
                :key="opt.value"
                :value="opt.value"
              >
                {{ opt.label }}
              </SelectItem>
            </SelectContent>
          </Select>
          <p class="text-xs text-muted-foreground">
            Adjusts alert and chart colors for color vision deficiencies.
          </p>
        </div>

        <!-- High contrast -->
        <button
          class="flex items-center gap-4 rounded-lg border p-4 text-left transition-colors w-full"
          :class="highContrast
            ? 'border-primary bg-primary/5'
            : 'border-border hover:border-primary/40'"
          @click="setHighContrast(!highContrast)"
        >
          <div
            class="flex h-10 w-10 shrink-0 items-center justify-center rounded-md text-lg font-bold"
            :class="highContrast ? 'bg-primary text-primary-foreground' : 'bg-muted text-muted-foreground'"
          >
            Aa
          </div>
          <div>
            <p class="text-sm font-medium">High Contrast</p>
            <p class="text-xs text-muted-foreground">Increases text and border contrast for better readability.</p>
          </div>
        </button>

        <!-- Dyslexia font -->
        <button
          class="flex items-center gap-4 rounded-lg border p-4 text-left transition-colors w-full"
          :class="dyslexiaFont
            ? 'border-primary bg-primary/5'
            : 'border-border hover:border-primary/40'"
          @click="setDyslexiaFont(!dyslexiaFont)"
        >
          <div
            class="flex h-10 w-10 shrink-0 items-center justify-center rounded-md text-sm font-bold"
            :class="dyslexiaFont ? 'bg-primary text-primary-foreground' : 'bg-muted text-muted-foreground'"
          >
            Dy
          </div>
          <div>
            <p class="text-sm font-medium">Dyslexia-Friendly Font</p>
            <p class="text-xs text-muted-foreground">Uses OpenDyslexic, a typeface designed for readers with dyslexia.</p>
          </div>
        </button>
      </div>

      <!-- Preview -->
      <div class="space-y-3">
        <h3 class="text-sm font-medium">Preview</h3>
        <div class="rounded-lg border p-4 space-y-3">
          <p class="text-sm">This is how your text looks with the current settings.</p>
          <div class="flex flex-wrap gap-2">
            <span class="rounded-md bg-primary px-2 py-1 text-xs text-primary-foreground">Primary</span>
            <span class="rounded-md bg-destructive px-2 py-1 text-xs text-destructive-foreground">Destructive</span>
            <span class="rounded-md bg-emerald-500 px-2 py-1 text-xs text-white">Success</span>
            <span class="rounded-md bg-amber-500 px-2 py-1 text-xs text-white">Warning</span>
            <span class="rounded-md bg-muted px-2 py-1 text-xs text-muted-foreground">Muted</span>
          </div>
          <p class="text-xs text-muted-foreground">
            The quick brown fox jumps over the lazy dog. 0O1lI
          </p>
          <p class="font-mono text-xs text-muted-foreground">
            Monospace: abc123 → webhook_delivery_ok
          </p>
        </div>
      </div>
    </div>
  </div>
</template>

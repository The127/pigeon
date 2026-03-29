import { ref, watch } from 'vue'

export type ThemeMode = 'auto' | 'light' | 'dark'
export type AccentColor = 'amber' | 'teal' | 'indigo' | 'rose' | 'emerald'
export type ColorblindMode = 'none' | 'deuteranopia' | 'protanopia' | 'tritanopia'

const MODE_KEY = 'pigeon-theme'
const ACCENT_KEY = 'pigeon-accent'
const COLORBLIND_KEY = 'pigeon-colorblind'

const mode = ref<ThemeMode>(
  (localStorage.getItem(MODE_KEY) as ThemeMode) || 'auto',
)

const accent = ref<AccentColor>(
  (localStorage.getItem(ACCENT_KEY) as AccentColor) || 'amber',
)

const colorblind = ref<ColorblindMode>(
  (localStorage.getItem(COLORBLIND_KEY) as ColorblindMode) || 'none',
)

function applyTheme(m: ThemeMode) {
  const root = document.documentElement
  if (m === 'dark' || (m === 'auto' && window.matchMedia('(prefers-color-scheme: dark)').matches)) {
    root.classList.add('dark')
  } else {
    root.classList.remove('dark')
  }
}

function applyAccent(a: AccentColor) {
  document.documentElement.setAttribute('data-accent', a)
}

function applyColorblind(c: ColorblindMode) {
  document.documentElement.setAttribute('data-colorblind', c)
}

const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
mediaQuery.addEventListener('change', () => {
  if (mode.value === 'auto') applyTheme('auto')
})

watch(mode, (m) => {
  localStorage.setItem(MODE_KEY, m)
  applyTheme(m)
}, { immediate: true })

watch(accent, (a) => {
  localStorage.setItem(ACCENT_KEY, a)
  applyAccent(a)
}, { immediate: true })

watch(colorblind, (c) => {
  localStorage.setItem(COLORBLIND_KEY, c)
  applyColorblind(c)
}, { immediate: true })

export function useTheme() {
  return {
    mode,
    accent,
    colorblind,
    setMode: (m: ThemeMode) => { mode.value = m },
    setAccent: (a: AccentColor) => { accent.value = a },
    setColorblind: (c: ColorblindMode) => { colorblind.value = c },
  }
}

// Initialize on module load — ensures settings are applied before first render
applyTheme(mode.value)
applyAccent(accent.value)
applyColorblind(colorblind.value)

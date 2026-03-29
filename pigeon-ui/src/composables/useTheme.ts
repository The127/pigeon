import { ref, watch, type Ref } from 'vue'

export type ThemeMode = 'auto' | 'light' | 'dark'

const STORAGE_KEY = 'pigeon-theme'

const mode = ref<ThemeMode>(
  (localStorage.getItem(STORAGE_KEY) as ThemeMode) || 'auto',
)

function applyTheme(m: ThemeMode) {
  const root = document.documentElement
  if (m === 'dark' || (m === 'auto' && window.matchMedia('(prefers-color-scheme: dark)').matches)) {
    root.classList.add('dark')
  } else {
    root.classList.remove('dark')
  }
}

// Listen for system theme changes when in auto mode
const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
mediaQuery.addEventListener('change', () => {
  if (mode.value === 'auto') applyTheme('auto')
})

// Apply on change
watch(mode, (m) => {
  localStorage.setItem(STORAGE_KEY, m)
  applyTheme(m)
}, { immediate: true })

export function useTheme(): { mode: Ref<ThemeMode>; setMode: (m: ThemeMode) => void } {
  return {
    mode,
    setMode: (m: ThemeMode) => { mode.value = m },
  }
}

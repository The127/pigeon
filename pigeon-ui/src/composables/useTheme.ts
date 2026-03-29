import { ref, watch } from 'vue'

export type ThemeMode = 'auto' | 'light' | 'dark'
export type AccentColor = 'amber' | 'teal' | 'indigo' | 'rose' | 'emerald'
export type ColorblindMode = 'none' | 'deuteranopia' | 'protanopia' | 'tritanopia'

const MODE_KEY = 'pigeon-theme'
const ACCENT_KEY = 'pigeon-accent'
const COLORBLIND_KEY = 'pigeon-colorblind'
const CONTRAST_KEY = 'pigeon-contrast'
const DYSLEXIA_KEY = 'pigeon-dyslexia'

const mode = ref<ThemeMode>(
  (localStorage.getItem(MODE_KEY) as ThemeMode) || 'auto',
)

const accent = ref<AccentColor>(
  (localStorage.getItem(ACCENT_KEY) as AccentColor) || 'amber',
)

const colorblind = ref<ColorblindMode>(
  (localStorage.getItem(COLORBLIND_KEY) as ColorblindMode) || 'none',
)

const highContrast = ref(
  localStorage.getItem(CONTRAST_KEY) === 'true',
)

const dyslexiaFont = ref(
  localStorage.getItem(DYSLEXIA_KEY) === 'true',
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

function applyContrast(on: boolean) {
  if (on) {
    document.documentElement.setAttribute('data-contrast', 'high')
  } else {
    document.documentElement.removeAttribute('data-contrast')
  }
}

function applyDyslexia(on: boolean) {
  if (on) {
    document.documentElement.classList.add('dyslexia-font')
    // Load OpenDyslexic font if not already loaded
    if (!document.getElementById('opendyslexic-font')) {
      const link = document.createElement('link')
      link.id = 'opendyslexic-font'
      link.rel = 'stylesheet'
      link.href = 'https://fonts.cdnfonts.com/css/opendyslexic'
      document.head.appendChild(link)
    }
  } else {
    document.documentElement.classList.remove('dyslexia-font')
  }
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

watch(highContrast, (on) => {
  localStorage.setItem(CONTRAST_KEY, String(on))
  applyContrast(on)
}, { immediate: true })

watch(dyslexiaFont, (on) => {
  localStorage.setItem(DYSLEXIA_KEY, String(on))
  applyDyslexia(on)
}, { immediate: true })

export function useTheme() {
  return {
    mode,
    accent,
    colorblind,
    highContrast,
    dyslexiaFont,
    setMode: (m: ThemeMode) => { mode.value = m },
    setAccent: (a: AccentColor) => { accent.value = a },
    setColorblind: (c: ColorblindMode) => { colorblind.value = c },
    setHighContrast: (on: boolean) => { highContrast.value = on },
    setDyslexiaFont: (on: boolean) => { dyslexiaFont.value = on },
  }
}

// Initialize on module load
applyTheme(mode.value)
applyAccent(accent.value)
applyColorblind(colorblind.value)
applyContrast(highContrast.value)
applyDyslexia(dyslexiaFont.value)

import { inject, provide, ref, shallowRef, triggerRef, type InjectionKey, type Ref, type ShallowRef } from 'vue'

export interface Toast {
  id: number
  text: string
  type: 'success' | 'error' | 'info' | 'warning'
  timeout: number
  remaining: number
}

export interface ToastApi {
  toasts: ShallowRef<Toast[]>
  isHovered: Ref<boolean>
  show: (options: { text: string; type?: Toast['type']; timeout?: number }) => void
  success: (text: string) => void
  error: (text: string) => void
  info: (text: string) => void
  warning: (text: string) => void
  remove: (id: number) => void
}

const TOAST_KEY: InjectionKey<ToastApi> = Symbol('toast')

export function provideToast(options?: { maximum?: number; timeout?: number }): ToastApi {
  const maximum = options?.maximum ?? 5
  const defaultTimeout = options?.timeout ?? 5000

  const toasts = shallowRef<Toast[]>([])
  const isHovered = ref(false)

  function remove(id: number) {
    toasts.value = toasts.value.filter(t => t.id !== id)
  }

  function show(opts: { text: string; type?: Toast['type']; timeout?: number }) {
    if (toasts.value.length >= maximum) {
      toasts.value = toasts.value.slice(0, -1)
    }

    const id = Date.now() + Math.random()
    const duration = opts.timeout ?? defaultTimeout

    const toast: Toast = {
      id,
      text: opts.text,
      type: opts.type ?? 'info',
      timeout: duration,
      remaining: duration,
    }

    toasts.value = [toast, ...toasts.value]

    let start = Date.now()
    let pausedAt: number | null = null

    function update() {
      // Toast was removed
      if (!toasts.value.find(t => t.id === id)) return

      if (isHovered.value) {
        if (!pausedAt) pausedAt = Date.now()
        requestAnimationFrame(update)
        return
      }

      if (pausedAt) {
        start += Date.now() - pausedAt
        pausedAt = null
      }

      const elapsed = Date.now() - start
      toast.remaining = Math.max(0, duration - elapsed)
      triggerRef(toasts)

      if (elapsed < duration) {
        requestAnimationFrame(update)
      } else {
        remove(id)
      }
    }

    requestAnimationFrame(update)
  }

  const api: ToastApi = {
    toasts,
    isHovered,
    show,
    success: (text) => show({ text, type: 'success' }),
    error: (text) => show({ text, type: 'error' }),
    info: (text) => show({ text, type: 'info' }),
    warning: (text) => show({ text, type: 'warning' }),
    remove,
  }

  provide(TOAST_KEY, api)
  return api
}

export function useToast(): ToastApi {
  const api = inject(TOAST_KEY)
  if (!api) throw new Error('useToast() called outside of ToastProvider')
  return api
}

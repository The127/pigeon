import { createApp } from 'vue'
import { VueQueryPlugin } from '@tanstack/vue-query'
import App from './App.vue'
import router from './router'
import { initAuth } from './auth'
import './composables/useTheme' // Initialize theme settings on startup
import './assets/index.css'

async function bootstrap() {
  try {
    await initAuth()
  } catch (e) {
    console.warn('Auth init failed — OIDC config not available, login will be required', e)
  }

  const app = createApp(App)
  app.use(router)
  app.use(VueQueryPlugin)
  app.mount('#app')
}

bootstrap()

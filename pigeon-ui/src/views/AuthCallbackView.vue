<script setup lang="ts">
import { onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { handleCallback } from '@/auth'

const router = useRouter()

onMounted(async () => {
  try {
    await handleCallback()
    const redirect = (router.currentRoute.value.query.redirect as string) || '/apps'
    router.replace(redirect)
  } catch (e) {
    console.error('Auth callback failed', e)
    router.replace('/login')
  }
})
</script>

<template>
  <div class="flex min-h-screen items-center justify-center">
    <p class="text-sm text-muted-foreground">Signing in...</p>
  </div>
</template>

<script setup lang="ts">
import { useAuth } from '@/auth'

const { login } = useAuth()
</script>

<template>
  <div class="flex min-h-screen items-center justify-center">
    <div class="w-full max-w-sm space-y-6 text-center">
      <h1 class="text-2xl font-semibold tracking-tight">Pigeon</h1>
      <p class="text-sm text-muted-foreground">Sign in to manage your webhooks</p>
      <button
        class="inline-flex h-10 w-full items-center justify-center rounded-md bg-primary px-4 text-sm font-medium text-primary-foreground shadow hover:bg-primary/90"
        @click="login"
      >
        Sign in with SSO
      </button>
    </div>
  </div>
</template>

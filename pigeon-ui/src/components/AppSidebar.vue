<script setup lang="ts">
import { useRoute } from 'vue-router'
import { LayoutGrid, LogOut, Send } from 'lucide-vue-next'
import { useAuth } from '@/auth'

const route = useRoute()
const { user, logout } = useAuth()

const nav = [
  { name: 'Applications', to: '/apps', icon: LayoutGrid },
]

function isActive(path: string) {
  return route.path.startsWith(path)
}
</script>

<template>
  <aside class="flex h-screen w-56 flex-col border-r border-sidebar-border bg-sidebar">
    <div class="flex h-14 items-center gap-2 border-b border-sidebar-border px-4">
      <Send class="h-5 w-5 text-sidebar-primary" />
      <span class="text-sm font-semibold text-sidebar-foreground">Pigeon</span>
    </div>

    <nav class="flex-1 space-y-1 p-2">
      <RouterLink
        v-for="item in nav"
        :key="item.to"
        :to="item.to"
        class="flex items-center gap-2 rounded-md px-3 py-2 text-sm font-medium transition-colors"
        :class="
          isActive(item.to)
            ? 'bg-sidebar-accent text-sidebar-accent-foreground'
            : 'text-sidebar-foreground hover:bg-sidebar-accent hover:text-sidebar-accent-foreground'
        "
      >
        <component :is="item.icon" class="h-4 w-4" />
        {{ item.name }}
      </RouterLink>
    </nav>

    <div class="border-t border-sidebar-border p-2">
      <button
        class="flex w-full items-center gap-2 rounded-md px-3 py-2 text-sm text-sidebar-foreground transition-colors hover:bg-sidebar-accent"
        @click="logout"
      >
        <LogOut class="h-4 w-4" />
        <span v-if="user">{{ user.profile.email || user.profile.name || 'Sign out' }}</span>
        <span v-else>Sign out</span>
      </button>
    </div>
  </aside>
</template>

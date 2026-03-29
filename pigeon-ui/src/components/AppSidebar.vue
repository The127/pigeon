<script setup lang="ts">
import { useRoute } from 'vue-router'
import { LayoutGrid, LogOut, PanelLeftClose, PanelLeft, Sun, Moon, Monitor, ScrollText } from 'lucide-vue-next'
import { useAuth } from '@/auth'
import { useTheme, type ThemeMode } from '@/composables/useTheme'
import { Button } from '@/components/ui/button'
import PigeonLogo from '@/components/PigeonLogo.vue'
import { Separator } from '@/components/ui/separator'
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/components/ui/tooltip'

const collapsed = defineModel<boolean>('collapsed', { default: false })

const route = useRoute()
const { user, logout } = useAuth()
const { mode, setMode } = useTheme()

const nav = [
  { name: 'Applications', to: '/apps', icon: LayoutGrid },
  { name: 'Audit Log', to: '/audit-log', icon: ScrollText },
]

function isActive(path: string) {
  return route.path.startsWith(path)
}

const displayName = () => {
  if (!user.value) return 'Sign out'
  return user.value.profile.email || user.value.profile.name || 'Sign out'
}

const themeOptions: { value: ThemeMode; icon: typeof Sun; label: string }[] = [
  { value: 'auto', icon: Monitor, label: 'System' },
  { value: 'light', icon: Sun, label: 'Light' },
  { value: 'dark', icon: Moon, label: 'Dark' },
]

function cycleTheme() {
  const order: ThemeMode[] = ['auto', 'light', 'dark']
  const next = order[(order.indexOf(mode.value) + 1) % order.length]
  setMode(next)
}

const currentThemeIcon = () => {
  return themeOptions.find(o => o.value === mode.value)?.icon || Monitor
}

const currentThemeLabel = () => {
  return themeOptions.find(o => o.value === mode.value)?.label || 'System'
}
</script>

<template>
  <TooltipProvider :delay-duration="0">
    <aside
      class="flex h-screen flex-col border-r border-sidebar-border bg-sidebar transition-all duration-200"
      :class="collapsed ? 'w-16' : 'w-56'"
    >
      <!-- Header -->
      <div
        class="flex h-14 items-center border-b border-sidebar-border px-3"
        :class="collapsed ? 'justify-center' : 'justify-between'"
      >
        <div v-if="!collapsed" class="flex items-center gap-2">
          <PigeonLogo size="sm" />
          <span class="text-sm font-semibold text-sidebar-foreground">Pigeon</span>
        </div>
        <Tooltip>
          <TooltipTrigger as-child>
            <Button
              variant="ghost"
              size="icon"
              class="h-8 w-8 shrink-0 text-sidebar-foreground hover:bg-sidebar-accent hover:text-sidebar-accent-foreground"
              @click="collapsed = !collapsed"
            >
              <PanelLeftClose v-if="!collapsed" class="h-4 w-4" />
              <PanelLeft v-else class="h-4 w-4" />
            </Button>
          </TooltipTrigger>
          <TooltipContent side="right">
            {{ collapsed ? 'Expand' : 'Collapse' }} sidebar
          </TooltipContent>
        </Tooltip>
      </div>

      <!-- Nav -->
      <nav class="flex-1 space-y-1 p-2">
        <Tooltip v-for="item in nav" :key="item.to">
          <TooltipTrigger as-child>
            <RouterLink
              :to="item.to"
              class="flex items-center gap-2 rounded-md px-3 py-2 text-sm font-medium transition-colors"
              :class="[
                isActive(item.to)
                  ? 'bg-sidebar-accent text-sidebar-accent-foreground'
                  : 'text-sidebar-foreground hover:bg-sidebar-accent hover:text-sidebar-accent-foreground',
                collapsed ? 'justify-center px-0' : '',
              ]"
            >
              <component :is="item.icon" class="h-4 w-4 shrink-0" />
              <span v-show="!collapsed">{{ item.name }}</span>
            </RouterLink>
          </TooltipTrigger>
          <TooltipContent v-if="collapsed" side="right">
            {{ item.name }}
          </TooltipContent>
        </Tooltip>
      </nav>

      <Separator class="bg-sidebar-border" />

      <!-- Footer -->
      <div class="space-y-1 p-2">
        <!-- Theme toggle -->
        <Tooltip>
          <TooltipTrigger as-child>
            <Button
              variant="ghost"
              size="sm"
              class="w-full text-sidebar-foreground hover:bg-sidebar-accent hover:text-sidebar-accent-foreground"
              :class="collapsed ? 'justify-center px-0' : 'justify-start'"
              @click="cycleTheme"
            >
              <component :is="currentThemeIcon()" class="h-4 w-4 shrink-0" />
              <span v-show="!collapsed" class="ml-2">{{ currentThemeLabel() }}</span>
            </Button>
          </TooltipTrigger>
          <TooltipContent v-if="collapsed" side="right">
            Theme: {{ currentThemeLabel() }}
          </TooltipContent>
        </Tooltip>

        <!-- Sign out -->
        <Tooltip>
          <TooltipTrigger as-child>
            <Button
              variant="ghost"
              size="sm"
              class="w-full text-sidebar-foreground hover:bg-sidebar-accent hover:text-sidebar-accent-foreground"
              :class="collapsed ? 'justify-center px-0' : 'justify-start'"
              @click="logout"
            >
              <LogOut class="h-4 w-4 shrink-0" />
              <span v-show="!collapsed" class="ml-2 truncate">{{ displayName() }}</span>
            </Button>
          </TooltipTrigger>
          <TooltipContent v-if="collapsed" side="right">
            {{ displayName() }}
          </TooltipContent>
        </Tooltip>
      </div>
    </aside>
  </TooltipProvider>
</template>

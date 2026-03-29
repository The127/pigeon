<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { useAuth } from '@/auth'
import { Button } from '@/components/ui/button'
import { Separator } from '@/components/ui/separator'
import { Zap, RefreshCcw, Activity, ArrowRight, Check } from 'lucide-vue-next'
import PigeonLogo from '@/components/PigeonLogo.vue'

const { login } = useAuth()

const healthy = ref<boolean | null>(null)
let healthInterval: ReturnType<typeof setInterval> | null = null

async function checkHealth() {
  try {
    const res = await fetch('/health/ready')
    healthy.value = res.ok
  } catch {
    healthy.value = false
  }
}

onMounted(() => {
  checkHealth()
  healthInterval = setInterval(checkHealth, 15000)
})

onUnmounted(() => {
  if (healthInterval) clearInterval(healthInterval)
})

const features = [
  { icon: Zap, title: 'Fan-out delivery', desc: 'One message, every subscribed endpoint' },
  { icon: RefreshCcw, title: 'Automatic retries', desc: 'Exponential backoff with dead letter queue' },
  { icon: Activity, title: 'Observability', desc: 'Per-endpoint stats and delivery tracking' },
]
</script>

<template>
  <div class="grid min-h-screen lg:grid-cols-2">
    <!-- Left: Hero panel -->
    <div class="relative hidden overflow-hidden bg-primary lg:flex lg:flex-col lg:justify-between">
      <!-- Animated background -->
      <div class="absolute inset-0 overflow-hidden">
        <div class="login-orb login-orb-1" />
        <div class="login-orb login-orb-2" />
        <div class="login-orb login-orb-3" />
      </div>

      <!-- Content -->
      <div class="relative z-10 flex flex-1 flex-col justify-center px-12 xl:px-16">
        <div class="flex items-center gap-3 mb-8">
          <PigeonLogo />
          <span class="font-mono text-sm font-medium tracking-widest uppercase text-primary-foreground/60">
            Pigeon
          </span>
        </div>

        <h1 class="text-4xl font-bold tracking-tight text-primary-foreground xl:text-5xl" style="line-height: 1.1;">
          Reliable webhook delivery for your infrastructure.
        </h1>

        <p class="mt-4 max-w-md text-base text-primary-foreground/60 leading-relaxed">
          Self-hosted, observable, and built for production.
          Send once, deliver everywhere.
        </p>

        <Separator class="my-8 bg-primary-foreground/10" />

        <div class="space-y-5">
          <div
            v-for="(feature, i) in features"
            :key="i"
            class="flex items-start gap-3 login-feature"
            :style="{ animationDelay: `${0.8 + i * 0.15}s` }"
          >
            <div class="mt-0.5 flex h-7 w-7 shrink-0 items-center justify-center rounded-md bg-primary-foreground/10">
              <component :is="feature.icon" class="h-3.5 w-3.5 text-primary-foreground/80" />
            </div>
            <div>
              <p class="text-sm font-medium text-primary-foreground">{{ feature.title }}</p>
              <p class="text-sm text-primary-foreground/50">{{ feature.desc }}</p>
            </div>
          </div>
        </div>
      </div>

      <!-- Bottom status bar -->
      <div class="relative z-10 flex items-center justify-between border-t border-primary-foreground/10 px-12 py-4 xl:px-16">
        <div class="flex items-center gap-2 font-mono text-xs text-primary-foreground/40">
          <span
            class="inline-block h-1.5 w-1.5 rounded-full"
            :class="{
              'bg-emerald-400': healthy === true,
              'bg-red-400': healthy === false,
              'bg-primary-foreground/20 animate-pulse': healthy === null,
            }"
          />
          <span v-if="healthy === null">Checking...</span>
          <span v-else-if="healthy">All systems operational</span>
          <span v-else>Service unavailable</span>
        </div>
        <a
          href="https://github.com/The127/pigeon"
          target="_blank"
          rel="noopener noreferrer"
          class="font-mono text-xs text-primary-foreground/30 transition-colors hover:text-primary-foreground/60"
        >
          GitHub
        </a>
      </div>
    </div>

    <!-- Right: Sign-in -->
    <div class="flex flex-col items-center justify-center px-6 py-12">
      <div class="w-full max-w-sm space-y-8">
        <!-- Mobile logo -->
        <div class="flex items-center gap-3 lg:hidden">
          <PigeonLogo />
          <span class="font-mono text-sm font-medium tracking-widest uppercase text-muted-foreground">
            Pigeon
          </span>
        </div>

        <div>
          <h2 class="text-2xl font-semibold tracking-tight">Welcome back</h2>
          <p class="mt-1 text-sm text-muted-foreground">
            Sign in to your account to manage applications, endpoints, and deliveries.
          </p>
        </div>

        <Button class="group w-full" size="lg" @click="login">
          Continue with SSO
          <ArrowRight class="ml-2 h-4 w-4 transition-transform duration-200 group-hover:translate-x-1" />
        </Button>

        <div class="space-y-3 rounded-lg border border-border/60 bg-muted/30 p-4">
          <p class="text-xs font-medium uppercase tracking-wider text-muted-foreground">
            What you can do
          </p>
          <div
            v-for="item in [
              'Create applications & event types',
              'Configure endpoints with filtering',
              'Monitor delivery attempts in real-time',
              'Replay failed messages from dead letter queue',
            ]"
            :key="item"
            class="flex items-center gap-2 text-sm text-muted-foreground"
          >
            <Check class="h-3.5 w-3.5 shrink-0 text-foreground/40" />
            {{ item }}
          </div>
        </div>

        <p class="text-center text-xs text-muted-foreground/60">
          Authentication is handled by your organization's identity provider.
        </p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.login-feature {
  animation: feature-in 0.5s ease-out both;
}

@keyframes feature-in {
  from {
    opacity: 0;
    transform: translateX(-12px);
  }
  to {
    opacity: 1;
    transform: translateX(0);
  }
}

.login-orb {
  position: absolute;
  border-radius: 50%;
  filter: blur(80px);
}

.login-orb-1 {
  width: 500px;
  height: 500px;
  top: -10%;
  left: -10%;
  background: rgba(255, 255, 255, 0.04);
  animation: orb-drift-1 25s ease-in-out infinite alternate;
}

.login-orb-2 {
  width: 400px;
  height: 400px;
  bottom: -5%;
  right: -5%;
  background: rgba(255, 255, 255, 0.03);
  animation: orb-drift-2 20s ease-in-out infinite alternate;
}

.login-orb-3 {
  width: 300px;
  height: 300px;
  top: 40%;
  left: 30%;
  background: rgba(255, 255, 255, 0.02);
  animation: orb-drift-3 30s ease-in-out infinite alternate;
}

@keyframes orb-drift-1 {
  0% { transform: translate(0, 0); }
  100% { transform: translate(15%, 20%); }
}

@keyframes orb-drift-2 {
  0% { transform: translate(0, 0); }
  100% { transform: translate(-20%, -15%); }
}

@keyframes orb-drift-3 {
  0% { transform: translate(0, 0); }
  100% { transform: translate(10%, -25%); }
}
</style>

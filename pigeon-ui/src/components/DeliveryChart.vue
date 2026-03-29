<script setup lang="ts">
import { computed } from 'vue'
import type { TimeBucket } from '@/api/applications'
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/components/ui/tooltip'

const props = defineProps<{
  data: TimeBucket[]
  bucketLabel?: (bucket: string) => string
}>()

const maxValue = computed(() => {
  if (!props.data.length) return 1
  return Math.max(...props.data.map(b => b.succeeded + b.failed), 1)
})

const chartHeightPx = 160 // matches h-40

function barHeight(value: number) {
  if (value === 0) return '0px'
  const pct = value / maxValue.value
  const px = Math.max(4, Math.round(pct * chartHeightPx))
  return `${px}px`
}

function defaultLabel(bucket: string) {
  const d = new Date(bucket)
  return d.toLocaleString(undefined, { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' })
}

function shortLabel(bucket: string) {
  const d = new Date(bucket)
  return d.toLocaleString(undefined, { hour: '2-digit', minute: '2-digit' })
}

const label = computed(() => props.bucketLabel || defaultLabel)
</script>

<template>
  <div class="rounded-lg border bg-card p-4">
    <p class="mb-4 text-sm font-medium text-muted-foreground">Delivery Volume</p>

    <div v-if="!data.length" class="flex h-40 items-center justify-center text-sm text-muted-foreground">
      No delivery data for this period.
    </div>

    <div v-else class="flex h-40 items-end gap-px">
      <TooltipProvider :delay-duration="0">
        <Tooltip v-for="(bucket, i) in data" :key="i">
          <TooltipTrigger as-child>
            <div
              class="relative flex flex-1 flex-col items-stretch justify-end"
              :style="{ minWidth: '4px' }"
            >
              <!-- Failed portion (stacked on top) -->
              <div
                v-if="bucket.failed > 0"
                class="w-full rounded-t-sm bg-destructive/70"
                :style="{ height: barHeight(bucket.failed) }"
              />
              <!-- Succeeded portion -->
              <div
                v-if="bucket.succeeded > 0"
                class="w-full bg-emerald-500"
                :class="bucket.failed > 0 ? '' : 'rounded-t-sm'"
                :style="{ height: barHeight(bucket.succeeded) }"
              />
              <!-- Empty bar if no data -->
              <div
                v-if="bucket.succeeded === 0 && bucket.failed === 0"
                class="w-full rounded-t-sm bg-muted"
                style="height: 2px"
              />
            </div>
          </TooltipTrigger>
          <TooltipContent>
            <p class="font-medium">{{ label(bucket.bucket) }}</p>
            <p class="text-emerald-500">{{ bucket.succeeded }} succeeded</p>
            <p v-if="bucket.failed" class="text-destructive">{{ bucket.failed }} failed</p>
          </TooltipContent>
        </Tooltip>
      </TooltipProvider>
    </div>

    <!-- X-axis labels -->
    <div v-if="data.length" class="mt-2 flex justify-between text-xs text-muted-foreground">
      <span>{{ shortLabel(data[0].bucket) }}</span>
      <span v-if="data.length > 2">{{ shortLabel(data[Math.floor(data.length / 2)].bucket) }}</span>
      <span>{{ shortLabel(data[data.length - 1].bucket) }}</span>
    </div>

    <!-- Legend -->
    <div class="mt-3 flex items-center gap-4 text-xs text-muted-foreground">
      <span class="flex items-center gap-1.5">
        <span class="inline-block h-2 w-2 rounded-sm bg-emerald-500" />
        Succeeded
      </span>
      <span class="flex items-center gap-1.5">
        <span class="inline-block h-2 w-2 rounded-sm bg-destructive/70" />
        Failed
      </span>
    </div>
  </div>
</template>

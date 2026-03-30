<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useToast } from '@/composables/useToast'
import {
  useEndpoints,
  useUpdateEndpoint,
  useDeleteEndpoint,
  useEndpointStats,
  useEventTypes,
  useRotateSigningSecret,
  useRevokeSigningSecret,
} from '@/api/applications'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import PageHeader from '@/components/PageHeader.vue'
import FormField from '@/components/FormField.vue'
import SecretRevealDialog from '@/components/SecretRevealDialog.vue'
import StatCard from '@/components/StatCard.vue'
import DeliveryChart from '@/components/DeliveryChart.vue'
import LoadingState from '@/components/LoadingState.vue'
import {
  ArrowLeft,
  Activity,
  CheckCircle,
  XCircle,
  Skull,
  Pencil,
  Trash2,
  AlertTriangle,
  Clock,
  RotateCcw,
  Key,
} from 'lucide-vue-next'

const route = useRoute()
const router = useRouter()
const toast = useToast()
const appId = computed(() => route.params.id as string)
const epId = computed(() => route.params.epId as string)

const statsPeriod = ref('24h')
const { data: stats, isLoading: statsLoading } = useEndpointStats(appId, epId, statsPeriod)

const { data: endpointsData } = useEndpoints(appId)
const endpoint = computed(() =>
  endpointsData.value?.items.find(ep => ep.id === epId.value)
)

const { data: eventTypesData } = useEventTypes(appId)
const subscribedEventTypes = computed(() =>
  eventTypesData.value?.items.filter(et =>
    endpoint.value?.event_type_ids.includes(et.id)
  ) || []
)

// Edit
const editDialogOpen = ref(false)
const editName = ref('')
const editUrl = ref('')
const editEventTypeIds = ref<string[]>([])
const updateEp = useUpdateEndpoint(appId)

function openEditDialog() {
  if (!endpoint.value) return
  editName.value = endpoint.value.name
  editUrl.value = endpoint.value.url
  editEventTypeIds.value = [...endpoint.value.event_type_ids]
  editDialogOpen.value = true
}

function toggleEditEventType(id: string) {
  const idx = editEventTypeIds.value.indexOf(id)
  if (idx >= 0) editEventTypeIds.value.splice(idx, 1)
  else editEventTypeIds.value.push(id)
}

function handleUpdate() {
  if (!endpoint.value) return
  updateEp.mutate(
    {
      id: epId.value,
      body: {
        url: editUrl.value,
        event_type_ids: editEventTypeIds.value,
        version: endpoint.value.version,
      },
    },
    {
      onSuccess: () => { editDialogOpen.value = false; toast.success('Endpoint updated') },
      onError: (e: Error) => toast.error(e.message),
    },
  )
}

// Signing Secrets
const rotateSecret = useRotateSigningSecret(appId)
const revokeSecret = useRevokeSigningSecret(appId)
const revealedSecret = ref('')
const secretRevealOpen = ref(false)

function handleRotate() {
  rotateSecret.mutate(epId.value, {
    onSuccess: (data) => {
      revealedSecret.value = data.new_secret
      secretRevealOpen.value = true
    },
    onError: (e: Error) => toast.error(e.message),
  })
}

function handleSecretRevealed() {
  secretRevealOpen.value = false
  revealedSecret.value = ''
  toast.success('Signing secret rotated')
}

function handleRevoke(index: number) {
  revokeSecret.mutate(
    { endpointId: epId.value, index },
    {
      onSuccess: () => toast.success('Signing secret revoked'),
      onError: (e: Error) => toast.error(e.message),
    },
  )
}

// Delete
const deleteDialogOpen = ref(false)
const deleteEp = useDeleteEndpoint(appId)

function handleDelete() {
  deleteDialogOpen.value = false
  deleteEp.mutate(epId.value, {
    onSuccess: () => { toast.success('Endpoint deleted'); router.push(`/apps/${appId.value}`) },
    onError: (e) => toast.error(e.message),
  })
}

function lastDeliveryLabel() {
  if (!stats.value?.last_delivery_at) return 'Never'
  return new Date(stats.value.last_delivery_at).toLocaleString()
}
</script>

<template>
  <div class="space-y-6">
    <div>
      <RouterLink
        :to="`/apps/${appId}`"
        class="mb-3 inline-flex items-center gap-1 text-sm text-muted-foreground hover:text-foreground transition-colors"
      >
        <ArrowLeft class="h-3.5 w-3.5" />
        Back to application
      </RouterLink>

      <PageHeader
        :title="endpoint?.name || 'Endpoint'"
        :description="endpoint?.url"
      >
        <template #actions>
          <Badge :variant="endpoint?.enabled ? 'default' : 'destructive'" class="mr-2">
            {{ endpoint?.enabled ? 'Active' : 'Disabled' }}
          </Badge>
          <Button variant="outline" size="sm" @click="openEditDialog">
            <Pencil class="mr-2 h-4 w-4" />
            Edit
          </Button>
          <Button
            variant="outline"
            size="sm"
            class="text-destructive hover:text-destructive"
            @click="deleteDialogOpen = true"
          >
            <Trash2 class="mr-2 h-4 w-4" />
            Delete
          </Button>
        </template>
      </PageHeader>
    </div>

    <!-- Period selector -->
    <div class="flex justify-end">
      <div class="inline-flex rounded-md border">
        <button
          v-for="p in ['24h', '7d', '30d']"
          :key="p"
          class="px-3 py-1.5 text-xs font-medium transition-colors first:rounded-l-md last:rounded-r-md"
          :class="statsPeriod === p
            ? 'bg-primary text-primary-foreground'
            : 'text-muted-foreground hover:text-foreground'"
          @click="statsPeriod = p"
        >
          {{ p }}
        </button>
      </div>
    </div>

    <LoadingState v-if="statsLoading" message="Loading stats..." />

    <template v-else-if="stats">
      <!-- Stat cards -->
      <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-6">
        <StatCard
          title="Pending"
          :value="stats.total_pending"
          :icon="Activity"
          :variant="stats.total_pending > 0 ? 'warning' : 'default'"
        />
        <StatCard title="Succeeded" :value="stats.total_succeeded" :icon="CheckCircle" variant="success" />
        <StatCard
          title="Failed"
          :value="stats.total_failed"
          :icon="XCircle"
          :variant="stats.total_failed > 0 ? 'destructive' : 'default'"
        />
        <StatCard
          title="Dead Lettered"
          :value="stats.total_dead_lettered"
          :icon="Skull"
          :variant="stats.total_dead_lettered > 0 ? 'warning' : 'default'"
        />
        <StatCard
          title="Consecutive Failures"
          :value="stats.consecutive_failures"
          :icon="AlertTriangle"
          :variant="stats.consecutive_failures > 0 ? 'destructive' : 'default'"
        />
        <StatCard
          title="Last Delivery"
          :value="stats.last_status || '—'"
          :icon="Clock"
          :description="lastDeliveryLabel()"
        />
      </div>

      <!-- Success rate -->
      <div v-if="stats.total_attempts > 0" class="rounded-lg border bg-card p-4">
        <div class="flex items-center justify-between">
          <p class="text-sm font-medium text-muted-foreground">Success Rate</p>
          <p class="text-2xl font-semibold" :class="stats.success_rate >= 0.95 ? 'text-emerald-600' : stats.success_rate >= 0.8 ? 'text-amber-600' : 'text-destructive'">
            {{ `${(stats.success_rate * 100).toFixed(1)}%` }}
          </p>
        </div>
        <div class="mt-2 h-2 w-full overflow-hidden rounded-full bg-muted">
          <div
            class="h-full rounded-full transition-all duration-500"
            :class="stats.success_rate >= 0.95 ? 'bg-emerald-500' : stats.success_rate >= 0.8 ? 'bg-amber-500' : 'bg-destructive'"
            :style="{ width: `${stats.success_rate * 100}%` }"
          />
        </div>
      </div>

      <!-- Chart -->
      <DeliveryChart :data="stats.time_series" />

      <!-- Subscribed event types -->
      <div class="space-y-3">
        <h3 class="text-sm font-medium">Subscribed Event Types</h3>
        <div v-if="!subscribedEventTypes.length" class="text-sm text-muted-foreground">
          No event types subscribed.
        </div>
        <div v-else class="flex flex-wrap gap-2">
          <RouterLink
            v-for="et in subscribedEventTypes"
            :key="et.id"
            :to="`/apps/${appId}/event-types/${et.id}`"
          >
            <Badge variant="outline" class="hover:bg-accent transition-colors">
              {{ et.name }}
            </Badge>
          </RouterLink>
        </div>
      </div>

      <!-- Signing Secrets -->
      <div class="space-y-3">
        <div class="flex items-center justify-between">
          <h3 class="text-sm font-medium flex items-center gap-2">
            <Key class="h-4 w-4" />
            Signing Secrets
          </h3>
          <Button variant="outline" size="sm" @click="handleRotate" :disabled="rotateSecret.isPending.value">
            <RotateCcw class="mr-2 h-4 w-4" />
            {{ rotateSecret.isPending.value ? 'Rotating...' : 'Rotate' }}
          </Button>
        </div>
        <div v-if="!endpoint?.signing_secrets_masked.length" class="text-sm text-muted-foreground">
          No signing secrets configured.
        </div>
        <div v-else class="space-y-2">
          <div
            v-for="(masked, index) in endpoint.signing_secrets_masked"
            :key="index"
            class="flex items-center justify-between rounded-md border px-3 py-2"
          >
            <div class="flex items-center gap-2">
              <code class="text-sm font-mono">{{ masked }}</code>
              <Badge v-if="index === 0" variant="default">current</Badge>
              <Badge v-else variant="secondary">previous</Badge>
            </div>
            <Button
              v-if="index > 0"
              variant="ghost"
              size="sm"
              class="text-destructive hover:text-destructive"
              :disabled="revokeSecret.isPending.value"
              @click="handleRevoke(index)"
            >
              Revoke
            </Button>
          </div>
        </div>
      </div>
    </template>

    <!-- Edit dialog -->
    <Dialog v-model:open="editDialogOpen">
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Edit Endpoint</DialogTitle>
          <DialogDescription>Update endpoint configuration.</DialogDescription>
        </DialogHeader>
        <form class="space-y-4" @submit.prevent="handleUpdate">
          <FormField label="Name" html-for="edit-ep-name" required>
            <Input id="edit-ep-name" v-model="editName" />
          </FormField>
          <FormField label="URL" html-for="edit-ep-url" required>
            <Input id="edit-ep-url" v-model="editUrl" />
          </FormField>
          <FormField v-if="eventTypesData?.items.length" label="Event Types" description="Select which events this endpoint receives.">
            <div class="flex flex-wrap gap-2">
              <button
                v-for="et in eventTypesData.items"
                :key="et.id"
                type="button"
                @click="toggleEditEventType(et.id)"
              >
                <Badge :variant="editEventTypeIds.includes(et.id) ? 'default' : 'outline'">
                  {{ et.name }}
                </Badge>
              </button>
            </div>
          </FormField>
          <DialogFooter>
            <Button type="submit" :disabled="updateEp.isPending.value || !editUrl">
              {{ updateEp.isPending.value ? 'Saving...' : 'Save' }}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>

    <!-- Delete dialog -->
    <Dialog v-model:open="deleteDialogOpen">
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Delete endpoint</DialogTitle>
          <DialogDescription>
            Are you sure you want to delete <strong>{{ endpoint?.name }}</strong>?
            This will remove all delivery history for this endpoint.
          </DialogDescription>
        </DialogHeader>
        <DialogFooter>
          <Button variant="outline" @click="deleteDialogOpen = false">Cancel</Button>
          <Button variant="destructive" @click="handleDelete">Delete</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>

    <!-- Secret Reveal Dialog -->
    <SecretRevealDialog
      :open="secretRevealOpen"
      :secret="revealedSecret"
      @confirmed="handleSecretRevealed"
    />
  </div>
</template>

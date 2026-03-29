<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useToast } from '@/composables/useToast'
import {
  useEventTypes,
  useUpdateEventType,
  useDeleteEventType,
  useEventTypeStats,
  useEndpoints,
  type RecentMessage,
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
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import PageHeader from '@/components/PageHeader.vue'
import FormField from '@/components/FormField.vue'
import StatCard from '@/components/StatCard.vue'
import DeliveryChart from '@/components/DeliveryChart.vue'
import LoadingState from '@/components/LoadingState.vue'
import ErrorState from '@/components/ErrorState.vue'
import {
  ArrowLeft,
  Inbox,
  Activity,
  CheckCircle,
  XCircle,
  Skull,
  Globe,
  Pencil,
  Trash2,
} from 'lucide-vue-next'

const route = useRoute()
const router = useRouter()
const toast = useToast()
const appId = computed(() => route.params.id as string)
const etId = computed(() => route.params.etId as string)

const statsPeriod = ref('24h')
const { data: stats, isLoading: statsLoading } = useEventTypeStats(appId, etId, statsPeriod)

// Get the event type name from the list
const { data: eventTypesData } = useEventTypes(appId)
const eventType = computed(() =>
  eventTypesData.value?.items.find(et => et.id === etId.value)
)

// Get subscribed endpoints
const { data: endpointsData } = useEndpoints(appId)
const subscribedEndpoints = computed(() =>
  endpointsData.value?.items.filter(ep =>
    ep.event_type_ids.includes(etId.value)
  ) || []
)

// Edit
const editDialogOpen = ref(false)
const editName = ref('')
const updateEt = useUpdateEventType(appId)

function openEditDialog() {
  if (!eventType.value) return
  editName.value = eventType.value.name
  editDialogOpen.value = true
}

function handleUpdate() {
  if (!eventType.value) return
  updateEt.mutate(
    { id: etId.value, body: { name: editName.value, version: eventType.value.version } },
    {
      onSuccess: () => { editDialogOpen.value = false; toast.success('Event type updated') },
      onError: (e: Error) => toast.error(e.message),
    },
  )
}

// Delete
const deleteDialogOpen = ref(false)
const deleteEt = useDeleteEventType(appId)

function handleDelete() {
  deleteDialogOpen.value = false
  deleteEt.mutate(etId.value, {
    onSuccess: () => { toast.success('Event type deleted'); router.push(`/apps/${appId.value}`) },
    onError: (e) => toast.error(e.message),
  })
}

function messageStatusLabel(msg: RecentMessage) {
  if (msg.attempts_created === 0) return { label: 'No endpoints', variant: 'outline' as const }
  if (msg.dead_lettered > 0) return { label: 'Dead lettered', variant: 'destructive' as const }
  if (msg.succeeded >= msg.attempts_created) return { label: 'Delivered', variant: 'default' as const }
  if (msg.failed > 0) return { label: 'Partial', variant: 'secondary' as const }
  if (msg.succeeded > 0) return { label: 'Delivering', variant: 'secondary' as const }
  return { label: 'Pending', variant: 'secondary' as const }
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
        :title="eventType?.name || 'Event Type'"
        description="Event type delivery metrics and configuration."
      >
        <template #actions>
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
      <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-7">
        <StatCard title="Messages" :value="stats.total_messages" :icon="Inbox" />
        <StatCard
          title="Endpoints"
          :value="stats.subscribed_endpoints"
          :icon="Globe"
        />
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
          title="Success Rate"
          :value="stats.total_attempts > 0 ? `${(stats.success_rate * 100).toFixed(1)}%` : '—'"
          :variant="stats.success_rate >= 0.95 ? 'success' : stats.success_rate >= 0.8 ? 'warning' : 'destructive'"
        />
      </div>

      <!-- Chart -->
      <DeliveryChart :data="stats.time_series" />

      <!-- Subscribed endpoints -->
      <div class="space-y-3">
        <h3 class="text-sm font-medium">Subscribed Endpoints</h3>
        <div v-if="!subscribedEndpoints.length" class="text-sm text-muted-foreground">
          No endpoints subscribe to this event type.
        </div>
        <div v-else class="flex flex-wrap gap-2">
          <RouterLink
            v-for="ep in subscribedEndpoints"
            :key="ep.id"
            :to="`/apps/${appId}/endpoints/${ep.id}`"
          >
            <Badge variant="outline" class="hover:bg-accent transition-colors">
              {{ ep.name }}
            </Badge>
          </RouterLink>
        </div>
      </div>

      <!-- Recent messages -->
      <div class="space-y-3">
        <h3 class="text-sm font-medium">Recent Messages</h3>
        <div v-if="!stats.recent_messages.length" class="text-sm text-muted-foreground">
          No messages of this type have been sent.
        </div>
        <Table v-else>
          <TableHeader>
            <TableRow>
              <TableHead>Idempotency Key</TableHead>
              <TableHead>Status</TableHead>
              <TableHead>Attempts</TableHead>
              <TableHead>Created</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            <TableRow v-for="msg in stats.recent_messages" :key="msg.id">
              <TableCell class="font-mono text-xs text-muted-foreground">
                {{ msg.idempotency_key }}
              </TableCell>
              <TableCell>
                <Badge :variant="messageStatusLabel(msg).variant">
                  {{ messageStatusLabel(msg).label }}
                </Badge>
              </TableCell>
              <TableCell class="text-muted-foreground text-sm">
                {{ msg.succeeded }}/{{ msg.attempts_created }}
              </TableCell>
              <TableCell class="text-muted-foreground">
                {{ new Date(msg.created_at).toLocaleString() }}
              </TableCell>
            </TableRow>
          </TableBody>
        </Table>
      </div>
    </template>

    <!-- Edit dialog -->
    <Dialog v-model:open="editDialogOpen">
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Edit Event Type</DialogTitle>
          <DialogDescription>Update the event type name.</DialogDescription>
        </DialogHeader>
        <form class="space-y-4" @submit.prevent="handleUpdate">
          <FormField label="Name" html-for="edit-et-name">
            <Input id="edit-et-name" v-model="editName" />
          </FormField>
          <DialogFooter>
            <Button type="submit" :disabled="updateEt.isPending.value || !editName">
              {{ updateEt.isPending.value ? 'Saving...' : 'Save' }}
            </Button>
          </DialogFooter>
        </form>
        <ErrorState v-if="updateEt.error.value" :message="updateEt.error.value.message" />
      </DialogContent>
    </Dialog>

    <!-- Delete dialog -->
    <Dialog v-model:open="deleteDialogOpen">
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Delete event type</DialogTitle>
          <DialogDescription>
            Are you sure you want to delete <strong>{{ eventType?.name }}</strong>?
            This will remove all associated messages and delivery history.
          </DialogDescription>
        </DialogHeader>
        <DialogFooter>
          <Button variant="outline" @click="deleteDialogOpen = false">Cancel</Button>
          <Button variant="destructive" @click="handleDelete">Delete</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  </div>
</template>

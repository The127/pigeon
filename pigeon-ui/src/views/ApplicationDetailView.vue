<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useToast } from '@/composables/useToast'
import {
  useApplication,
  useUpdateApplication,
  useDeleteApplication,
  useEventTypes,
  useEndpoints,
  useCreateEventType,
  useDeleteEventType,
  useCreateEndpoint,
  useDeleteEndpoint,
  useSendMessage,
  useMessages,
  useAttempts,
  useDeadLetters,
  useReplayDeadLetter,
  useAppStats,
  useRetriggerMessage,
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
  DialogTrigger,
} from '@/components/ui/dialog'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { Input } from '@/components/ui/input'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import PageHeader from '@/components/PageHeader.vue'
import FormField from '@/components/FormField.vue'
import EmptyState from '@/components/EmptyState.vue'
import LoadingState from '@/components/LoadingState.vue'
import ErrorState from '@/components/ErrorState.vue'
import { Textarea } from '@/components/ui/textarea'
import StatCard from '@/components/StatCard.vue'
import DeliveryChart from '@/components/DeliveryChart.vue'
import { Plus, Zap, Globe, ArrowLeft, MoreHorizontal, Trash2, Pencil, Send, CheckCircle2, Mail, AlertTriangle, RotateCcw, ChevronDown, ChevronRight, Activity, Inbox, CheckCircle, XCircle, Skull } from 'lucide-vue-next'

const route = useRoute()
const router = useRouter()
const toast = useToast()
const appId = computed(() => route.params.id as string)

const { data: app, isLoading, error } = useApplication(appId)
const statsPeriod = ref('24h')
const { data: stats, isLoading: statsLoading } = useAppStats(appId, statsPeriod)
const updateApp = useUpdateApplication(appId)
const deleteApp = useDeleteApplication()
const { data: eventTypesData, isLoading: etLoading } = useEventTypes(appId)
const { data: endpointsData, isLoading: epLoading } = useEndpoints(appId)

// --- Edit Application ---
const editDialogOpen = ref(false)
const editName = ref('')

function openEditDialog() {
  if (!app.value) return
  editName.value = app.value.name
  editDialogOpen.value = true
}

function handleUpdateApp() {
  if (!app.value) return
  updateApp.mutate(
    { name: editName.value, version: app.value.version },
    {
      onSuccess: () => { editDialogOpen.value = false; toast.success('Application updated') },
      onError: (e: Error) => toast.error(e.message),
    },
  )
}

// --- Delete Application ---
const deleteAppOpen = ref(false)

function handleDeleteApp() {
  deleteApp.mutate(appId.value, {
    onSuccess: () => { toast.success('Application deleted'); router.push('/apps') },
    onError: (e) => toast.error(e.message),
  })
}

// --- Create Event Type ---
const etDialogOpen = ref(false)
const etName = ref('')
const createEt = useCreateEventType(appId)

function handleCreateEventType() {
  createEt.mutate(
    { name: etName.value },
    {
      onSuccess: () => { etDialogOpen.value = false; etName.value = ''; toast.success('Event type created') },
      onError: (e: Error) => toast.error(e.message),
    },
  )
}

// --- Delete Event Type ---
const deleteEt = useDeleteEventType(appId)
const deleteEtTarget = ref<{ id: string; name: string } | null>(null)

function handleDeleteEventType() {
  const id = deleteEtTarget.value?.id
  if (!id) return
  deleteEtTarget.value = null
  deleteEt.mutate(id, {
    onSuccess: () => toast.success('Event type deleted'),
    onError: (e) => toast.error(e.message),
  })
}

// --- Create Endpoint ---
const epDialogOpen = ref(false)
const epName = ref('')
const epUrl = ref('')
const epSecret = ref('')
const epEventTypeIds = ref<string[]>([])
const createEp = useCreateEndpoint(appId)

function handleCreateEndpoint() {
  createEp.mutate(
    {
      name: epName.value || undefined,
      url: epUrl.value,
      signing_secret: epSecret.value || undefined,
      event_type_ids: epEventTypeIds.value,
    },
    {
      onSuccess: () => {
        epDialogOpen.value = false
        epName.value = ''
        epUrl.value = ''
        epSecret.value = ''
        epEventTypeIds.value = []
        toast.success('Endpoint created')
      },
      onError: (e: Error) => toast.error(e.message),
    },
  )
}

function toggleEventType(id: string) {
  const idx = epEventTypeIds.value.indexOf(id)
  if (idx >= 0) epEventTypeIds.value.splice(idx, 1)
  else epEventTypeIds.value.push(id)
}

// --- Delete Endpoint ---
const deleteEp = useDeleteEndpoint(appId)
const deleteEpTarget = ref<{ id: string; url: string } | null>(null)

function handleDeleteEndpoint() {
  const id = deleteEpTarget.value?.id
  if (!id) return
  deleteEpTarget.value = null
  deleteEp.mutate(id, {
    onSuccess: () => toast.success('Endpoint deleted'),
    onError: (e) => toast.error(e.message),
  })
}

// --- Send Message ---
const sendMsg = useSendMessage(appId)
const msgEventTypeId = ref('')
const msgPayload = ref('{\n  \n}')
const msgResult = ref<{ id: string; attempts: number; duplicate: boolean } | null>(null)

function handleSendMessage() {
  let payload: unknown
  try {
    payload = JSON.parse(msgPayload.value)
  } catch {
    return
  }
  sendMsg.mutate(
    { event_type_id: msgEventTypeId.value, payload },
    {
      onSuccess: (data) => {
        msgResult.value = {
          id: data.id,
          attempts: data.attempts_created,
          duplicate: data.was_duplicate,
        }
        toast.success(data.was_duplicate ? 'Duplicate message (idempotent)' : `Message sent — ${data.attempts_created} attempt(s)`)
      },
      onError: (e: Error) => toast.error(e.message),
    },
  )
}

const payloadValid = computed(() => {
  try {
    JSON.parse(msgPayload.value)
    return true
  } catch {
    return false
  }
})

// --- Messages ---
const { data: messagesData, isLoading: msgLoading } = useMessages(appId)
const retriggerMsg = useRetriggerMessage(appId)
const expandedMessageId = ref<string | null>(null)
const { data: attemptsData, isLoading: attLoading } = useAttempts(appId, expandedMessageId)

function toggleMessage(id: string) {
  expandedMessageId.value = expandedMessageId.value === id ? null : id
}

function eventTypeName(etId: string) {
  return eventTypesData.value?.items.find(e => e.id === etId)?.name || etId.slice(0, 8)
}

function endpointLabel(epId: string) {
  const ep = endpointsData.value?.items.find(e => e.id === epId)
  return ep?.name || ep?.url || epId.slice(0, 8)
}

function messageStatus(msg: { attempts_created: number; succeeded: number; failed: number; dead_lettered: number }) {
  if (msg.attempts_created === 0) return { label: 'No endpoints', variant: 'outline' as const }
  if (msg.dead_lettered > 0) return { label: 'Dead lettered', variant: 'destructive' as const }
  if (msg.succeeded >= msg.attempts_created) return { label: 'Delivered', variant: 'default' as const }
  if (msg.failed > 0 && msg.succeeded === 0) return { label: 'Failed', variant: 'destructive' as const }
  if (msg.failed > 0) return { label: 'Partial', variant: 'secondary' as const }
  if (msg.succeeded > 0) return { label: 'Delivering', variant: 'secondary' as const }
  return { label: 'Pending', variant: 'secondary' as const }
}

function statusColor(status: string) {
  switch (status) {
    case 'succeeded': return 'default'
    case 'pending': return 'secondary'
    case 'in_flight': return 'secondary'
    case 'failed': return 'destructive'
    default: return 'outline'
  }
}

function handleRetrigger(messageId: string) {
  retriggerMsg.mutate(messageId, {
    onSuccess: () => toast.success('Message retriggered'),
    onError: (e) => toast.error(e.message),
  })
}

// --- Dead Letters ---
const { data: deadLettersData, isLoading: dlLoading } = useDeadLetters(appId)
const replayDl = useReplayDeadLetter(appId)

function handleReplay(deadLetterId: string) {
  replayDl.mutate(deadLetterId, {
    onSuccess: () => toast.success('Dead letter replayed'),
    onError: (e) => toast.error(e.message),
  })
}
</script>

<template>
  <div class="space-y-6">
    <LoadingState v-if="isLoading" message="Loading application..." />
    <ErrorState v-else-if="error" :message="error.message" />

    <template v-else-if="app">
      <div>
        <RouterLink
          to="/apps"
          class="mb-3 inline-flex items-center gap-1 text-sm text-muted-foreground hover:text-foreground transition-colors"
        >
          <ArrowLeft class="h-3.5 w-3.5" />
          Applications
        </RouterLink>

        <PageHeader :title="app.name" :description="`UID: ${app.uid}`">
          <template #actions>
            <Button variant="outline" size="sm" @click="openEditDialog">
              <Pencil class="mr-2 h-4 w-4" />
              Edit
            </Button>
            <Button variant="outline" size="sm" class="text-destructive hover:text-destructive" @click="deleteAppOpen = true">
              <Trash2 class="mr-2 h-4 w-4" />
              Delete
            </Button>
          </template>
        </PageHeader>
      </div>

      <!-- Edit Application Dialog -->
      <Dialog v-model:open="editDialogOpen">
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Edit Application</DialogTitle>
            <DialogDescription>Update the application name.</DialogDescription>
          </DialogHeader>
          <form class="space-y-4" @submit.prevent="handleUpdateApp">
            <FormField label="Name" html-for="edit-name">
              <Input id="edit-name" v-model="editName" />
            </FormField>
            <DialogFooter>
              <Button type="submit" :disabled="updateApp.isPending.value || !editName">
                {{ updateApp.isPending.value ? 'Saving...' : 'Save' }}
              </Button>
            </DialogFooter>
          </form>
          <ErrorState v-if="updateApp.error.value" :message="updateApp.error.value.message" />
        </DialogContent>
      </Dialog>

      <!-- Delete Application Dialog -->
      <Dialog v-model:open="deleteAppOpen">
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Delete application</DialogTitle>
            <DialogDescription>
              Are you sure you want to delete <strong>{{ app.name }}</strong>?
              This will remove all event types, endpoints, and message history.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" @click="deleteAppOpen = false">Cancel</Button>
            <Button variant="destructive" @click="handleDeleteApp">Delete</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Tabs default-value="dashboard">
        <TabsList>
          <TabsTrigger value="dashboard">
            Dashboard
          </TabsTrigger>
          <TabsTrigger value="event-types">
            Event Types
            <Badge v-if="eventTypesData?.total" variant="secondary" class="ml-2">
              {{ eventTypesData.total }}
            </Badge>
          </TabsTrigger>
          <TabsTrigger value="endpoints">
            Endpoints
            <Badge v-if="endpointsData?.total" variant="secondary" class="ml-2">
              {{ endpointsData.total }}
            </Badge>
          </TabsTrigger>
          <TabsTrigger value="messages">
            Messages
            <Badge v-if="messagesData?.total" variant="secondary" class="ml-2">
              {{ messagesData.total }}
            </Badge>
          </TabsTrigger>
          <TabsTrigger value="dead-letters">
            Dead Letters
            <Badge v-if="deadLettersData?.total" variant="secondary" class="ml-2">
              {{ deadLettersData.total }}
            </Badge>
          </TabsTrigger>
          <TabsTrigger value="send">
            Send Message
          </TabsTrigger>
        </TabsList>

        <!-- Event Types Tab -->
        <!-- Dashboard Tab -->
        <TabsContent value="dashboard" class="space-y-6">
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
                title="Messages"
                :value="stats.total_messages"
                :icon="Inbox"
              />
              <StatCard
                title="Pending"
                :value="stats.total_pending"
                :icon="Activity"
                :variant="stats.total_pending > 0 ? 'warning' : 'default'"
              />
              <StatCard
                title="Succeeded"
                :value="stats.total_succeeded"
                :icon="CheckCircle"
                variant="success"
              />
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
            </div>

            <!-- Success rate -->
            <div class="rounded-lg border bg-card p-4">
              <div class="flex items-center justify-between">
                <p class="text-sm font-medium text-muted-foreground">Success Rate</p>
                <p class="text-2xl font-semibold" :class="stats.success_rate >= 0.95 ? 'text-emerald-600' : stats.success_rate >= 0.8 ? 'text-amber-600' : 'text-destructive'">
                  {{ stats.total_attempts > 0 ? `${(stats.success_rate * 100).toFixed(1)}%` : '—' }}
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
          </template>
        </TabsContent>

        <!-- Event Types Tab -->
        <TabsContent value="event-types" class="space-y-4">
          <div class="flex justify-end">
            <Dialog v-model:open="etDialogOpen">
              <DialogTrigger as-child>
                <Button size="sm">
                  <Plus class="mr-2 h-4 w-4" />
                  Add Event Type
                </Button>
              </DialogTrigger>
              <DialogContent>
                <DialogHeader>
                  <DialogTitle>Add Event Type</DialogTitle>
                  <DialogDescription>Define an event type that endpoints can subscribe to.</DialogDescription>
                </DialogHeader>
                <form class="space-y-4" @submit.prevent="handleCreateEventType">
                  <FormField label="Name" html-for="et-name" description="e.g. order.placed, user.created">
                    <Input id="et-name" v-model="etName" placeholder="order.placed" />
                  </FormField>
                  <DialogFooter>
                    <Button type="submit" :disabled="createEt.isPending.value || !etName">
                      {{ createEt.isPending.value ? 'Creating...' : 'Create' }}
                    </Button>
                  </DialogFooter>
                </form>
                <ErrorState v-if="createEt.error.value" :message="createEt.error.value.message" />
              </DialogContent>
            </Dialog>
          </div>

          <LoadingState v-if="etLoading" message="Loading event types..." />

          <EmptyState
            v-else-if="!eventTypesData?.items.length"
            :icon="Zap"
            title="No event types"
            description="Add your first event type to define what kinds of events this application sends."
          >
            <Button size="sm" @click="etDialogOpen = true">
              <Plus class="mr-2 h-4 w-4" />
              Add Event Type
            </Button>
          </EmptyState>

          <Table v-else>
            <TableHeader>
              <TableRow>
                <TableHead>Name</TableHead>
                <TableHead>Created</TableHead>
                <TableHead class="w-12" />
              </TableRow>
            </TableHeader>
            <TableBody>
              <TableRow
                v-for="et in eventTypesData.items"
                :key="et.id"
                class="cursor-pointer"
                @click="$router.push(`/apps/${appId}/event-types/${et.id}`)"
              >
                <TableCell>
                  <span class="font-medium text-primary underline-offset-4 hover:underline">{{ et.name }}</span>
                </TableCell>
                <TableCell class="text-muted-foreground">
                  {{ new Date(et.created_at).toLocaleDateString() }}
                </TableCell>
                <TableCell @click.stop>
                  <DropdownMenu>
                    <DropdownMenuTrigger as-child>
                      <Button variant="ghost" size="icon" class="h-8 w-8">
                        <MoreHorizontal class="h-4 w-4" />
                      </Button>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent align="end">
                      <DropdownMenuItem
                        class="text-destructive"
                        @click="deleteEtTarget = { id: et.id, name: et.name }"
                      >
                        <Trash2 class="mr-2 h-4 w-4" />
                        Delete
                      </DropdownMenuItem>
                    </DropdownMenuContent>
                  </DropdownMenu>
                </TableCell>
              </TableRow>
            </TableBody>
          </Table>
        </TabsContent>

        <!-- Endpoints Tab -->
        <TabsContent value="endpoints" class="space-y-4">
          <div class="flex justify-end">
            <Dialog v-model:open="epDialogOpen">
              <DialogTrigger as-child>
                <Button size="sm">
                  <Plus class="mr-2 h-4 w-4" />
                  Add Endpoint
                </Button>
              </DialogTrigger>
              <DialogContent>
                <DialogHeader>
                  <DialogTitle>Add Endpoint</DialogTitle>
                  <DialogDescription>Configure a URL to receive webhook deliveries.</DialogDescription>
                </DialogHeader>
                <form class="space-y-4" @submit.prevent="handleCreateEndpoint">
                  <FormField label="Name" html-for="ep-name" description="Optional — a friendly name will be auto-generated if left blank.">
                    <Input id="ep-name" v-model="epName" placeholder="e.g. production-webhook" />
                  </FormField>
                  <FormField label="URL" html-for="ep-url">
                    <Input id="ep-url" v-model="epUrl" placeholder="https://example.com/webhook" />
                  </FormField>
                  <FormField label="Signing Secret" html-for="ep-secret" description="Optional — used to sign payloads with HMAC-SHA256. Leave blank to skip signing.">
                    <Input id="ep-secret" v-model="epSecret" placeholder="whsec_..." />
                  </FormField>
                  <FormField v-if="eventTypesData?.items.length" label="Event Types" description="Select which events this endpoint receives.">
                    <div class="flex flex-wrap gap-2">
                      <button
                        v-for="et in eventTypesData.items"
                        :key="et.id"
                        type="button"
                        @click="toggleEventType(et.id)"
                      >
                        <Badge :variant="epEventTypeIds.includes(et.id) ? 'default' : 'outline'">
                          {{ et.name }}
                        </Badge>
                      </button>
                    </div>
                  </FormField>
                  <DialogFooter>
                    <Button type="submit" :disabled="createEp.isPending.value || !epUrl">
                      {{ createEp.isPending.value ? 'Creating...' : 'Create' }}
                    </Button>
                  </DialogFooter>
                </form>
                <ErrorState v-if="createEp.error.value" :message="createEp.error.value.message" />
              </DialogContent>
            </Dialog>
          </div>

          <LoadingState v-if="epLoading" message="Loading endpoints..." />

          <EmptyState
            v-else-if="!endpointsData?.items.length"
            :icon="Globe"
            title="No endpoints"
            description="Add an endpoint URL to start receiving webhook deliveries."
          >
            <Button size="sm" @click="epDialogOpen = true">
              <Plus class="mr-2 h-4 w-4" />
              Add Endpoint
            </Button>
          </EmptyState>

          <Table v-else>
            <TableHeader>
              <TableRow>
                <TableHead>Name</TableHead>
                <TableHead>URL</TableHead>
                <TableHead>Events</TableHead>
                <TableHead>Status</TableHead>
                <TableHead class="w-12" />
              </TableRow>
            </TableHeader>
            <TableBody>
              <TableRow
                v-for="ep in endpointsData.items"
                :key="ep.id"
                class="cursor-pointer"
                @click="$router.push(`/apps/${appId}/endpoints/${ep.id}`)"
              >
                <TableCell>
                  <span class="font-medium text-primary underline-offset-4 hover:underline">{{ ep.name }}</span>
                </TableCell>
                <TableCell class="font-mono text-sm text-muted-foreground">{{ ep.url }}</TableCell>
                <TableCell>
                  <div class="flex flex-wrap gap-1">
                    <Badge v-for="etId in ep.event_type_ids" :key="etId" variant="outline">
                      {{ eventTypesData?.items.find(e => e.id === etId)?.name || etId.slice(0, 8) }}
                    </Badge>
                    <span v-if="!ep.event_type_ids.length" class="text-muted-foreground text-sm">No events subscribed</span>
                  </div>
                </TableCell>
                <TableCell>
                  <Badge :variant="ep.enabled ? 'default' : 'destructive'">
                    {{ ep.enabled ? 'Active' : 'Disabled' }}
                  </Badge>
                </TableCell>
                <TableCell class="text-muted-foreground">
                  {{ new Date(ep.created_at).toLocaleDateString() }}
                </TableCell>
                <TableCell @click.stop>
                  <DropdownMenu>
                    <DropdownMenuTrigger as-child>
                      <Button variant="ghost" size="icon" class="h-8 w-8">
                        <MoreHorizontal class="h-4 w-4" />
                      </Button>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent align="end">
                      <DropdownMenuItem
                        class="text-destructive"
                        @click="deleteEpTarget = { id: ep.id, url: ep.url }"
                      >
                        <Trash2 class="mr-2 h-4 w-4" />
                        Delete
                      </DropdownMenuItem>
                    </DropdownMenuContent>
                  </DropdownMenu>
                </TableCell>
              </TableRow>
            </TableBody>
          </Table>
        </TabsContent>

        <!-- Send Message Tab -->
        <!-- Messages Tab -->
        <TabsContent value="messages" class="space-y-4">
          <LoadingState v-if="msgLoading" message="Loading messages..." />

          <EmptyState
            v-else-if="!messagesData?.items.length"
            :icon="Mail"
            title="No messages"
            description="Send a message from the Send Message tab to see delivery history here."
          />

          <Table v-else>
            <TableHeader>
              <TableRow>
                <TableHead class="w-8" />
                <TableHead>Event Type</TableHead>
                <TableHead>Idempotency Key</TableHead>
                <TableHead>Status</TableHead>
                <TableHead>Attempts</TableHead>
                <TableHead>Created</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              <template v-for="msg in messagesData.items" :key="msg.id">
                <TableRow
                  class="cursor-pointer"
                  @click="toggleMessage(msg.id)"
                >
                  <TableCell class="w-8 pr-0">
                    <ChevronDown v-if="expandedMessageId === msg.id" class="h-4 w-4 text-muted-foreground" />
                    <ChevronRight v-else class="h-4 w-4 text-muted-foreground" />
                  </TableCell>
                  <TableCell>
                    <Badge variant="outline">{{ eventTypeName(msg.event_type_id) }}</Badge>
                  </TableCell>
                  <TableCell class="font-mono text-xs text-muted-foreground">
                    {{ msg.idempotency_key }}
                  </TableCell>
                  <TableCell>
                    <Badge :variant="messageStatus(msg).variant">{{ messageStatus(msg).label }}</Badge>
                  </TableCell>
                  <TableCell class="text-muted-foreground text-sm">
                    {{ msg.succeeded }}/{{ msg.attempts_created }}
                    <span v-if="msg.dead_lettered > 0" class="text-destructive ml-1">({{ msg.dead_lettered }} dead)</span>
                  </TableCell>
                  <TableCell class="text-muted-foreground">
                    {{ new Date(msg.created_at).toLocaleString() }}
                  </TableCell>
                </TableRow>

                <!-- Expanded: Attempts -->
                <TableRow v-if="expandedMessageId === msg.id" class="bg-muted/30">
                  <TableCell :colspan="6" class="p-0">
                    <div class="px-6 py-4 space-y-3">
                      <LoadingState v-if="attLoading" message="Loading attempts..." />
                      <p v-else-if="!attemptsData?.length" class="text-sm text-muted-foreground">
                        No delivery attempts — no endpoints matched this event type when the message was sent.
                      </p>
                      <Table v-else>
                        <TableHeader>
                          <TableRow>
                            <TableHead>Endpoint</TableHead>
                            <TableHead>Status</TableHead>
                            <TableHead>Response</TableHead>
                            <TableHead>Duration</TableHead>
                            <TableHead>Attempted</TableHead>
                          </TableRow>
                        </TableHeader>
                        <TableBody>
                          <TableRow v-for="att in attemptsData" :key="att.id">
                            <TableCell>{{ endpointLabel(att.endpoint_id) }}</TableCell>
                            <TableCell>
                              <Badge :variant="statusColor(att.status)">{{ att.status }}</Badge>
                            </TableCell>
                            <TableCell class="text-muted-foreground">
                              {{ att.response_code ?? '—' }}
                            </TableCell>
                            <TableCell class="text-muted-foreground">
                              {{ att.duration_ms != null ? `${att.duration_ms}ms` : '—' }}
                            </TableCell>
                            <TableCell class="text-muted-foreground">
                              {{ att.attempted_at ? new Date(att.attempted_at).toLocaleString() : 'Pending' }}
                            </TableCell>
                          </TableRow>
                        </TableBody>
                      </Table>
                      <Button
                        size="sm"
                        variant="outline"
                        :disabled="retriggerMsg.isPending.value"
                        @click.stop="handleRetrigger(msg.id)"
                      >
                        <RotateCcw class="mr-2 h-4 w-4" />
                        {{ retriggerMsg.isPending.value ? 'Retriggering...' : 'Retrigger delivery' }}
                      </Button>
                    </div>
                  </TableCell>
                </TableRow>
              </template>
            </TableBody>
          </Table>
        </TabsContent>

        <!-- Dead Letters Tab -->
        <TabsContent value="dead-letters" class="space-y-4">
          <LoadingState v-if="dlLoading" message="Loading dead letters..." />

          <EmptyState
            v-else-if="!deadLettersData?.items.length"
            :icon="AlertTriangle"
            title="No dead letters"
            description="Messages that exhaust all retry attempts will appear here."
          />

          <Table v-else>
            <TableHeader>
              <TableRow>
                <TableHead>Endpoint</TableHead>
                <TableHead>Last Response</TableHead>
                <TableHead>Dead Lettered</TableHead>
                <TableHead>Status</TableHead>
                <TableHead class="w-12" />
              </TableRow>
            </TableHeader>
            <TableBody>
              <TableRow v-for="dl in deadLettersData.items" :key="dl.id">
                <TableCell class="font-mono text-xs">{{ endpointLabel(dl.endpoint_id) }}</TableCell>
                <TableCell class="text-muted-foreground">
                  {{ dl.last_response_code ?? '—' }}
                </TableCell>
                <TableCell class="text-muted-foreground">
                  {{ new Date(dl.dead_lettered_at).toLocaleString() }}
                </TableCell>
                <TableCell>
                  <Badge v-if="dl.replayed_at" variant="secondary">Replayed</Badge>
                  <Badge v-else variant="destructive">Failed</Badge>
                </TableCell>
                <TableCell>
                  <Button
                    v-if="!dl.replayed_at"
                    variant="ghost"
                    size="icon"
                    class="h-8 w-8"
                    :disabled="replayDl.isPending.value"
                    @click="handleReplay(dl.id)"
                  >
                    <RotateCcw class="h-4 w-4" />
                  </Button>
                </TableCell>
              </TableRow>
            </TableBody>
          </Table>
        </TabsContent>

        <!-- Send Message Tab -->
        <TabsContent value="send" class="space-y-4">
          <div class="max-w-lg space-y-4">
            <form class="space-y-4" @submit.prevent="handleSendMessage">
              <FormField label="Event Type" html-for="msg-et">
                <select
                  id="msg-et"
                  v-model="msgEventTypeId"
                  class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
                >
                  <option value="" disabled>Select an event type</option>
                  <option
                    v-for="et in eventTypesData?.items"
                    :key="et.id"
                    :value="et.id"
                  >
                    {{ et.name }}
                  </option>
                </select>
              </FormField>

              <FormField
                label="Payload"
                html-for="msg-payload"
                :description="payloadValid ? 'Valid JSON object.' : undefined"
                :error="!payloadValid && msgPayload.trim() ? 'Invalid JSON.' : undefined"
              >
                <Textarea
                  id="msg-payload"
                  v-model="msgPayload"
                  class="font-mono text-sm min-h-[160px]"
                  placeholder='{ "key": "value" }'
                />
              </FormField>

              <Button
                type="submit"
                :disabled="sendMsg.isPending.value || !msgEventTypeId || !payloadValid"
              >
                <Send class="mr-2 h-4 w-4" />
                {{ sendMsg.isPending.value ? 'Sending...' : 'Send Message' }}
              </Button>
            </form>

            <ErrorState v-if="sendMsg.error.value" :message="sendMsg.error.value.message" />

            <div
              v-if="msgResult"
              class="flex items-start gap-3 rounded-md border border-border bg-muted/30 p-4"
            >
              <CheckCircle2 class="mt-0.5 h-5 w-5 shrink-0 text-emerald-500" />
              <div class="space-y-1 text-sm">
                <p class="font-medium">
                  {{ msgResult.duplicate ? 'Duplicate message (idempotent)' : 'Message sent' }}
                </p>
                <p class="text-muted-foreground">
                  ID: <span class="font-mono">{{ msgResult.id.slice(0, 8) }}</span>
                  &middot; {{ msgResult.attempts }} attempt{{ msgResult.attempts !== 1 ? 's' : '' }} created
                </p>
              </div>
            </div>
          </div>
        </TabsContent>
      </Tabs>

      <!-- Delete Event Type confirmation -->
      <Dialog :open="!!deleteEtTarget" @update:open="(v: boolean) => { if (!v) deleteEtTarget = null }">
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Delete event type</DialogTitle>
            <DialogDescription>
              Are you sure you want to delete <strong>{{ deleteEtTarget?.name }}</strong>?
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" @click="deleteEtTarget = null">Cancel</Button>
            <Button variant="destructive" @click="handleDeleteEventType">Delete</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <!-- Delete Endpoint confirmation -->
      <Dialog :open="!!deleteEpTarget" @update:open="(v: boolean) => { if (!v) deleteEpTarget = null }">
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Delete endpoint</DialogTitle>
            <DialogDescription>
              Are you sure you want to delete <strong class="font-mono">{{ deleteEpTarget?.url }}</strong>?
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" @click="deleteEpTarget = null">Cancel</Button>
            <Button variant="destructive" @click="handleDeleteEndpoint">Delete</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </template>
  </div>
</template>

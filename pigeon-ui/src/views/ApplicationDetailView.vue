<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
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
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog'
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
import { Plus, Zap, Globe, ArrowLeft, MoreHorizontal, Trash2, Pencil, Send, CheckCircle2 } from 'lucide-vue-next'

const route = useRoute()
const router = useRouter()
const appId = computed(() => route.params.id as string)

const { data: app, isLoading, error } = useApplication(appId)
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
    { onSuccess: () => { editDialogOpen.value = false } },
  )
}

// --- Delete Application ---
const deleteAppOpen = ref(false)

function handleDeleteApp() {
  deleteApp.mutate(appId.value, {
    onSuccess: () => router.push('/apps'),
  })
}

// --- Create Event Type ---
const etDialogOpen = ref(false)
const etName = ref('')
const createEt = useCreateEventType(appId)

function handleCreateEventType() {
  createEt.mutate(
    { name: etName.value },
    { onSuccess: () => { etDialogOpen.value = false; etName.value = '' } },
  )
}

// --- Delete Event Type ---
const deleteEt = useDeleteEventType(appId)
const deleteEtTarget = ref<{ id: string; name: string } | null>(null)

function handleDeleteEventType() {
  if (!deleteEtTarget.value) return
  deleteEt.mutate(deleteEtTarget.value.id, {
    onSuccess: () => { deleteEtTarget.value = null },
  })
}

// --- Create Endpoint ---
const epDialogOpen = ref(false)
const epUrl = ref('')
const epSecret = ref('')
const epEventTypeIds = ref<string[]>([])
const createEp = useCreateEndpoint(appId)

function handleCreateEndpoint() {
  createEp.mutate(
    { url: epUrl.value, signing_secret: epSecret.value, event_type_ids: epEventTypeIds.value },
    {
      onSuccess: () => {
        epDialogOpen.value = false
        epUrl.value = ''
        epSecret.value = ''
        epEventTypeIds.value = []
      },
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
  if (!deleteEpTarget.value) return
  deleteEp.mutate(deleteEpTarget.value.id, {
    onSuccess: () => { deleteEpTarget.value = null },
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
      },
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
      <AlertDialog v-model:open="deleteAppOpen">
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Delete application</AlertDialogTitle>
            <AlertDialogDescription>
              Are you sure you want to delete <strong>{{ app.name }}</strong>?
              This will remove all event types, endpoints, and message history.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction
              class="bg-destructive text-destructive-foreground hover:bg-destructive/90"
              @click="handleDeleteApp"
            >
              Delete
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      <Tabs default-value="event-types">
        <TabsList>
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
          <TabsTrigger value="send">
            Send Message
          </TabsTrigger>
        </TabsList>

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
              <TableRow v-for="et in eventTypesData.items" :key="et.id">
                <TableCell class="font-medium">
                  <Badge variant="outline">{{ et.name }}</Badge>
                </TableCell>
                <TableCell class="text-muted-foreground">
                  {{ new Date(et.created_at).toLocaleDateString() }}
                </TableCell>
                <TableCell>
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
                  <FormField label="URL" html-for="ep-url">
                    <Input id="ep-url" v-model="epUrl" placeholder="https://example.com/webhook" />
                  </FormField>
                  <FormField label="Signing Secret" html-for="ep-secret" description="Used to sign payloads with HMAC-SHA256.">
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
                    <Button type="submit" :disabled="createEp.isPending.value || !epUrl || !epSecret">
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
                <TableHead>URL</TableHead>
                <TableHead>Events</TableHead>
                <TableHead>Status</TableHead>
                <TableHead>Created</TableHead>
                <TableHead class="w-12" />
              </TableRow>
            </TableHeader>
            <TableBody>
              <TableRow v-for="ep in endpointsData.items" :key="ep.id">
                <TableCell class="font-medium font-mono text-sm">{{ ep.url }}</TableCell>
                <TableCell>
                  <div class="flex flex-wrap gap-1">
                    <Badge v-for="etId in ep.event_type_ids" :key="etId" variant="outline">
                      {{ eventTypesData?.items.find(e => e.id === etId)?.name || etId.slice(0, 8) }}
                    </Badge>
                    <span v-if="!ep.event_type_ids.length" class="text-muted-foreground text-sm">All events</span>
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
                <TableCell>
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
      <AlertDialog :open="!!deleteEtTarget" @update:open="(v: boolean) => { if (!v) deleteEtTarget = null }">
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Delete event type</AlertDialogTitle>
            <AlertDialogDescription>
              Are you sure you want to delete <strong>{{ deleteEtTarget?.name }}</strong>?
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction
              class="bg-destructive text-destructive-foreground hover:bg-destructive/90"
              @click="handleDeleteEventType"
            >
              Delete
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      <!-- Delete Endpoint confirmation -->
      <AlertDialog :open="!!deleteEpTarget" @update:open="(v: boolean) => { if (!v) deleteEpTarget = null }">
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Delete endpoint</AlertDialogTitle>
            <AlertDialogDescription>
              Are you sure you want to delete <strong class="font-mono">{{ deleteEpTarget?.url }}</strong>?
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction
              class="bg-destructive text-destructive-foreground hover:bg-destructive/90"
              @click="handleDeleteEndpoint"
            >
              Delete
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </template>
  </div>
</template>

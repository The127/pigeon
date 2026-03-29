<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRoute } from 'vue-router'
import {
  useApplication,
  useEventTypes,
  useEndpoints,
  useCreateEventType,
  useCreateEndpoint,
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
import { Plus, Zap, Globe, ArrowLeft } from 'lucide-vue-next'

const route = useRoute()
const appId = computed(() => route.params.id as string)

const { data: app, isLoading, error } = useApplication(appId)
const { data: eventTypesData, isLoading: etLoading } = useEventTypes(appId)
const { data: endpointsData, isLoading: epLoading } = useEndpoints(appId)

// --- Create Event Type ---
const etDialogOpen = ref(false)
const etName = ref('')
const createEt = useCreateEventType(appId)

function handleCreateEventType() {
  createEt.mutate(
    { name: etName.value },
    {
      onSuccess: () => {
        etDialogOpen.value = false
        etName.value = ''
      },
    },
  )
}

// --- Create Endpoint ---
const epDialogOpen = ref(false)
const epUrl = ref('')
const epSecret = ref('')
const epEventTypeIds = ref<string[]>([])
const createEp = useCreateEndpoint(appId)

function handleCreateEndpoint() {
  createEp.mutate(
    {
      url: epUrl.value,
      signing_secret: epSecret.value,
      event_type_ids: epEventTypeIds.value,
    },
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
  if (idx >= 0) {
    epEventTypeIds.value.splice(idx, 1)
  } else {
    epEventTypeIds.value.push(id)
  }
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

        <PageHeader :title="app.name" :description="`UID: ${app.uid}`" />
      </div>

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
                  <DialogDescription>
                    Define an event type that endpoints can subscribe to.
                  </DialogDescription>
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
                  <DialogDescription>
                    Configure a URL to receive webhook deliveries.
                  </DialogDescription>
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
                        <Badge
                          :variant="epEventTypeIds.includes(et.id) ? 'default' : 'outline'"
                        >
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
              </TableRow>
            </TableHeader>
            <TableBody>
              <TableRow v-for="ep in endpointsData.items" :key="ep.id">
                <TableCell class="font-medium font-mono text-sm">{{ ep.url }}</TableCell>
                <TableCell>
                  <div class="flex flex-wrap gap-1">
                    <Badge
                      v-for="etId in ep.event_type_ids"
                      :key="etId"
                      variant="outline"
                    >
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
              </TableRow>
            </TableBody>
          </Table>
        </TabsContent>
      </Tabs>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useApplications, useCreateApplication, useDeleteApplication } from '@/api/applications'
import { useToast } from '@/composables/useToast'
import { Button } from '@/components/ui/button'
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
import { Badge } from '@/components/ui/badge'
import PageHeader from '@/components/PageHeader.vue'
import EmptyState from '@/components/EmptyState.vue'
import FormField from '@/components/FormField.vue'
import LoadingState from '@/components/LoadingState.vue'
import ErrorState from '@/components/ErrorState.vue'
import { Plus, LayoutGrid, MoreHorizontal, Trash2 } from 'lucide-vue-next'

const router = useRouter()
const toast = useToast()
const { data, isLoading, error } = useApplications()
const createApp = useCreateApplication()
const deleteApp = useDeleteApplication()

const dialogOpen = ref(false)
const newName = ref('')
const newUid = ref('')
const uidTouched = ref(false)

const deleteTarget = ref<{ id: string; name: string } | null>(null)

function slugify(value: string): string {
  return value
    .toLowerCase()
    .trim()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '')
}

function onNameInput() {
  if (!uidTouched.value) {
    newUid.value = slugify(newName.value)
  }
}

function handleCreate() {
  createApp.mutate(
    { name: newName.value, uid: newUid.value },
    {
      onSuccess: () => {
        dialogOpen.value = false
        toast.success('Application created')
        newName.value = ''
        newUid.value = ''
        uidTouched.value = false
      },
      onError: (e) => toast.error(e.message),
    },
  )
}

function handleDelete() {
  const id = deleteTarget.value?.id
  if (!id) return
  deleteTarget.value = null
  deleteApp.mutate(id, {
    onSuccess: () => toast.success('Application deleted'),
    onError: (e) => toast.error(e.message),
  })
}
</script>

<template>
  <div class="space-y-6">
    <PageHeader title="Applications" description="Manage your webhook applications.">
      <template #actions>
        <Dialog v-model:open="dialogOpen">
          <DialogTrigger as-child>
            <Button>
              <Plus class="mr-2 h-4 w-4" />
              Create Application
            </Button>
          </DialogTrigger>

          <DialogContent>
            <DialogHeader>
              <DialogTitle>Create Application</DialogTitle>
              <DialogDescription>
                Add a new application to send webhooks from.
              </DialogDescription>
            </DialogHeader>

            <form class="space-y-4" @submit.prevent="handleCreate">
              <FormField label="Name" html-for="name">
                <Input
                  id="name"
                  v-model="newName"
                  placeholder="My Application"
                  @input="onNameInput"
                />
              </FormField>

              <FormField
                label="UID"
                html-for="uid"
                description="A unique identifier for this application."
              >
                <Input
                  id="uid"
                  v-model="newUid"
                  placeholder="my-application"
                  @input="uidTouched = true"
                />
              </FormField>

              <DialogFooter>
                <Button
                  type="submit"
                  :disabled="createApp.isPending.value || !newName || !newUid"
                >
                  {{ createApp.isPending.value ? 'Creating...' : 'Create' }}
                </Button>
              </DialogFooter>
            </form>

            <ErrorState
              v-if="createApp.error.value"
              :message="createApp.error.value.message"
            />
          </DialogContent>
        </Dialog>
      </template>
    </PageHeader>

    <LoadingState v-if="isLoading" message="Loading applications..." />

    <ErrorState
      v-else-if="error"
      :message="`Failed to load applications: ${error.message}`"
    />

    <EmptyState
      v-else-if="!data?.items.length"
      :icon="LayoutGrid"
      title="No applications yet"
      description="Create your first application to start sending webhooks."
    >
      <Button @click="dialogOpen = true">
        <Plus class="mr-2 h-4 w-4" />
        Create Application
      </Button>
    </EmptyState>

    <Table v-else>
      <TableHeader>
        <TableRow>
          <TableHead>Name</TableHead>
          <TableHead>UID</TableHead>
          <TableHead>Created</TableHead>
          <TableHead class="w-12" />
        </TableRow>
      </TableHeader>
      <TableBody>
        <TableRow
          v-for="app in data.items"
          :key="app.id"
          class="cursor-pointer"
          @click="router.push(`/apps/${app.id}`)"
        >
          <TableCell>
            <span class="font-medium text-primary underline-offset-4 hover:underline">{{ app.name }}</span>
          </TableCell>
          <TableCell>
            <Badge variant="secondary">{{ app.uid }}</Badge>
          </TableCell>
          <TableCell class="text-muted-foreground">
            {{ new Date(app.created_at).toLocaleDateString() }}
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
                  @click="deleteTarget = { id: app.id, name: app.name }"
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

    <!-- Delete confirmation -->
    <Dialog :open="!!deleteTarget" @update:open="(v: boolean) => { if (!v) deleteTarget = null }">
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Delete application</DialogTitle>
          <DialogDescription>
            Are you sure you want to delete <strong>{{ deleteTarget?.name }}</strong>?
            This will remove all event types, endpoints, and message history.
          </DialogDescription>
        </DialogHeader>
        <DialogFooter>
          <Button variant="outline" @click="deleteTarget = null">Cancel</Button>
          <Button variant="destructive" @click="handleDelete">Delete</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  </div>
</template>

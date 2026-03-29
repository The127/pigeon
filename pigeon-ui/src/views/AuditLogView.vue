<script setup lang="ts">
import { ref } from 'vue'
import { useAuditLog } from '@/api/applications'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Input } from '@/components/ui/input'
import FilterSelect from '@/components/FilterSelect.vue'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import PageHeader from '@/components/PageHeader.vue'
import LoadingState from '@/components/LoadingState.vue'
import ErrorState from '@/components/ErrorState.vue'
import EmptyState from '@/components/EmptyState.vue'
import { ScrollText, ChevronLeft, ChevronRight } from 'lucide-vue-next'

const limit = ref(25)
const offset = ref(0)
const commandFilter = ref('')
const successFilter = ref('')
const { data, isLoading, error } = useAuditLog(offset, limit, commandFilter, successFilter)

function prevPage() {
  offset.value = Math.max(0, offset.value - limit.value)
}

function nextPage() {
  if (data.value && offset.value + limit.value < data.value.total) {
    offset.value += limit.value
  }
}

function formatCommand(name: string) {
  // "CreateApplication" → "Create Application"
  return name.replace(/([A-Z])/g, ' $1').trim()
}
</script>

<template>
  <div class="space-y-6">
    <PageHeader
      title="Audit Log"
      description="A record of all actions performed in your organization."
    />

    <div class="flex gap-2">
      <Input
        v-model="commandFilter"
        placeholder="Filter by action..."
        class="max-w-[200px]"
      />
      <FilterSelect
        v-model="successFilter"
        placeholder="All results"
        :options="[{ value: 'true', label: 'Success' }, { value: 'false', label: 'Failed' }]"
      />
    </div>

    <LoadingState v-if="isLoading" message="Loading audit log..." />

    <ErrorState v-else-if="error" :message="error.message" />

    <EmptyState
      v-else-if="!data?.items.length"
      :icon="ScrollText"
      title="No audit entries"
      description="Actions will be recorded here as you use Pigeon."
    />

    <template v-else>
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Action</TableHead>
            <TableHead>User</TableHead>
            <TableHead>Result</TableHead>
            <TableHead>Error</TableHead>
            <TableHead>Time</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          <TableRow v-for="entry in data.items" :key="entry.id">
            <TableCell class="font-medium">
              {{ formatCommand(entry.command_name) }}
            </TableCell>
            <TableCell class="font-mono text-xs text-muted-foreground">
              {{ entry.actor }}
            </TableCell>
            <TableCell>
              <Badge :variant="entry.success ? 'default' : 'destructive'">
                {{ entry.success ? 'Success' : 'Failed' }}
              </Badge>
            </TableCell>
            <TableCell class="max-w-xs truncate text-sm text-muted-foreground">
              {{ entry.error_message || '—' }}
            </TableCell>
            <TableCell class="text-muted-foreground whitespace-nowrap">
              {{ new Date(entry.timestamp).toLocaleString() }}
            </TableCell>
          </TableRow>
        </TableBody>
      </Table>

      <!-- Pagination -->
      <div class="flex items-center justify-between text-sm text-muted-foreground">
        <span>
          Showing {{ offset + 1 }}–{{ Math.min(offset + limit, data.total) }} of {{ data.total }}
        </span>
        <div class="flex items-center gap-2">
          <Button
            variant="outline"
            size="sm"
            :disabled="offset === 0"
            @click="prevPage"
          >
            <ChevronLeft class="h-4 w-4" />
          </Button>
          <Button
            variant="outline"
            size="sm"
            :disabled="offset + limit >= data.total"
            @click="nextPage"
          >
            <ChevronRight class="h-4 w-4" />
          </Button>
        </div>
      </div>
    </template>
  </div>
</template>

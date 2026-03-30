<script setup lang="ts">
import { ref } from 'vue'
import { useOidcConfigs, useCreateOidcConfig, useDeleteOidcConfig } from '@/api/oidc-configs'
import type { OidcConfigResponse } from '@/api/oidc-configs'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog'
import PageHeader from '@/components/PageHeader.vue'
import LoadingState from '@/components/LoadingState.vue'
import ErrorState from '@/components/ErrorState.vue'
import EmptyState from '@/components/EmptyState.vue'
import { Shield, Plus, Trash2 } from 'lucide-vue-next'
import { useToast } from '@/composables/useToast'

const { data, isLoading, error } = useOidcConfigs()
const createConfig = useCreateOidcConfig()
const deleteConfig = useDeleteOidcConfig()
const toast = useToast()

const showCreateDialog = ref(false)
const newIssuerUrl = ref('')
const newAudience = ref('')
const newJwksUrl = ref('')

function handleCreate() {
  createConfig.mutate(
    {
      issuer_url: newIssuerUrl.value,
      audience: newAudience.value,
      jwks_url: newJwksUrl.value,
    },
    {
      onSuccess: () => {
        toast.success('OIDC configuration created')
        showCreateDialog.value = false
        newIssuerUrl.value = ''
        newAudience.value = ''
        newJwksUrl.value = ''
      },
      onError: (e) => toast.error(e.message),
    },
  )
}

function handleDelete(config: OidcConfigResponse) {
  if (!confirm(`Delete OIDC config for ${config.audience}?`)) return
  deleteConfig.mutate(config.id, {
    onSuccess: () => toast.success('OIDC configuration deleted'),
    onError: (e) => toast.error(e.message),
  })
}
</script>

<template>
  <div class="space-y-6">
    <PageHeader
      title="OIDC Settings"
      description="Manage OpenID Connect configurations for your organization."
    >
      <template #actions>
        <Dialog v-model:open="showCreateDialog">
          <DialogTrigger as-child>
            <Button size="sm">
              <Plus class="mr-2 h-4 w-4" />
              Add Configuration
            </Button>
          </DialogTrigger>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>Add OIDC Configuration</DialogTitle>
            </DialogHeader>
            <div class="space-y-4 py-4">
              <div class="space-y-2">
                <label class="text-sm font-medium">Issuer URL</label>
                <Input
                  v-model="newIssuerUrl"
                  placeholder="https://auth.example.com"
                />
              </div>
              <div class="space-y-2">
                <label class="text-sm font-medium">Audience</label>
                <Input
                  v-model="newAudience"
                  placeholder="my-api"
                />
              </div>
              <div class="space-y-2">
                <label class="text-sm font-medium">JWKS URL</label>
                <Input
                  v-model="newJwksUrl"
                  placeholder="https://auth.example.com/.well-known/jwks.json"
                />
              </div>
            </div>
            <DialogFooter>
              <Button
                @click="handleCreate"
                :disabled="createConfig.isPending.value || !newIssuerUrl || !newAudience || !newJwksUrl"
              >
                Create
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      </template>
    </PageHeader>

    <LoadingState v-if="isLoading" message="Loading OIDC configurations..." />

    <ErrorState v-else-if="error" :message="error.message" />

    <EmptyState
      v-else-if="!data?.items.length"
      :icon="Shield"
      title="No OIDC configurations"
      description="Add an OpenID Connect provider to enable authentication."
    />

    <Table v-else>
      <TableHeader>
        <TableRow>
          <TableHead>Issuer URL</TableHead>
          <TableHead>Audience</TableHead>
          <TableHead>JWKS URL</TableHead>
          <TableHead>Created</TableHead>
          <TableHead class="w-[60px]" />
        </TableRow>
      </TableHeader>
      <TableBody>
        <TableRow v-for="config in data.items" :key="config.id">
          <TableCell class="font-mono text-sm">
            {{ config.issuer_url }}
          </TableCell>
          <TableCell class="font-mono text-sm">
            {{ config.audience }}
          </TableCell>
          <TableCell class="max-w-xs truncate font-mono text-sm text-muted-foreground">
            {{ config.jwks_url }}
          </TableCell>
          <TableCell class="text-muted-foreground whitespace-nowrap">
            {{ new Date(config.created_at).toLocaleDateString() }}
          </TableCell>
          <TableCell>
            <Button
              variant="ghost"
              size="icon"
              class="h-8 w-8 text-muted-foreground hover:text-destructive"
              :disabled="(data?.items.length ?? 0) <= 1"
              @click="handleDelete(config)"
            >
              <Trash2 class="h-4 w-4" />
            </Button>
          </TableCell>
        </TableRow>
      </TableBody>
    </Table>
  </div>
</template>

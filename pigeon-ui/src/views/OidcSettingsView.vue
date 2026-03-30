<script setup lang="ts">
import { ref, watch } from 'vue'
import { useOidcConfig, useUpdateOidcConfig } from '@/api/oidc-configs'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import PageHeader from '@/components/PageHeader.vue'
import LoadingState from '@/components/LoadingState.vue'
import ErrorState from '@/components/ErrorState.vue'
import { Pencil, X, Save } from 'lucide-vue-next'
import { useToast } from '@/composables/useToast'

const { data: config, isLoading, error } = useOidcConfig()
const updateConfig = useUpdateOidcConfig()
const toast = useToast()

const editing = ref(false)
const editIssuerUrl = ref('')
const editAudience = ref('')
const editJwksUrl = ref('')

function startEditing() {
  if (!config.value) return
  editIssuerUrl.value = config.value.issuer_url
  editAudience.value = config.value.audience
  editJwksUrl.value = config.value.jwks_url
  editing.value = true
}

function cancelEditing() {
  editing.value = false
}

function handleSave() {
  if (!config.value) return
  updateConfig.mutate(
    {
      oldId: config.value.id,
      issuer_url: editIssuerUrl.value,
      audience: editAudience.value,
      jwks_url: editJwksUrl.value,
    },
    {
      onSuccess: () => {
        toast.success('OIDC configuration updated')
        editing.value = false
      },
      onError: (e) => toast.error(e.message),
    },
  )
}

watch(config, () => {
  if (editing.value) return
})
</script>

<template>
  <div class="space-y-6">
    <PageHeader
      title="OIDC Settings"
      description="OpenID Connect configuration for your organization."
    >
      <template #actions>
        <template v-if="config && !editing">
          <Button size="sm" variant="outline" @click="startEditing">
            <Pencil class="mr-2 h-4 w-4" />
            Edit
          </Button>
        </template>
        <template v-if="editing">
          <Button size="sm" variant="ghost" @click="cancelEditing">
            <X class="mr-2 h-4 w-4" />
            Cancel
          </Button>
          <Button
            size="sm"
            @click="handleSave"
            :disabled="updateConfig.isPending.value || !editIssuerUrl || !editAudience || !editJwksUrl"
          >
            <Save class="mr-2 h-4 w-4" />
            Save
          </Button>
        </template>
      </template>
    </PageHeader>

    <LoadingState v-if="isLoading" message="Loading OIDC configuration..." />

    <ErrorState v-else-if="error" :message="error.message" />

    <template v-else-if="config">
      <form @submit.prevent="handleSave" class="rounded-lg border bg-card p-6 space-y-6">
        <!-- Issuer URL -->
        <div class="space-y-1">
          <label class="text-sm font-medium text-muted-foreground">Issuer URL</label>
          <Input
            v-if="editing"
            v-model="editIssuerUrl"
            placeholder="https://auth.example.com"
          />
          <p v-else class="font-mono text-sm">{{ config.issuer_url }}</p>
        </div>

        <!-- Audience -->
        <div class="space-y-1">
          <label class="text-sm font-medium text-muted-foreground">Audience</label>
          <Input
            v-if="editing"
            v-model="editAudience"
            placeholder="my-api"
          />
          <p v-else class="font-mono text-sm">{{ config.audience }}</p>
        </div>

        <!-- JWKS URL -->
        <div class="space-y-1">
          <label class="text-sm font-medium text-muted-foreground">JWKS URL</label>
          <Input
            v-if="editing"
            v-model="editJwksUrl"
            placeholder="https://auth.example.com/.well-known/jwks.json"
          />
          <p v-else class="font-mono text-sm break-all">{{ config.jwks_url }}</p>
        </div>

        <!-- Metadata -->
        <div class="border-t pt-4 text-sm text-muted-foreground">
          Last updated {{ new Date(config.created_at).toLocaleDateString() }}
        </div>
      </form>
    </template>
  </div>
</template>

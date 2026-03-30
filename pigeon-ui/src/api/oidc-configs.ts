import { useQuery, useMutation, useQueryClient } from '@tanstack/vue-query'
import type { OidcConfigResponse } from './generated/types.gen'
import { apiFetch } from './client'

export type { OidcConfigResponse }

export function useOidcConfig() {
  return useQuery<OidcConfigResponse | null>({
    queryKey: ['oidc-config'],
    queryFn: async () => {
      const result = await apiFetch<{ items: OidcConfigResponse[] }>('/oidc-configs?limit=1')
      return result.items[0] ?? null
    },
  })
}

export function useUpdateOidcConfig() {
  const queryClient = useQueryClient()

  return useMutation<
    OidcConfigResponse,
    Error,
    { oldId: string; issuer_url: string; audience: string; jwks_url: string }
  >({
    mutationFn: async ({ oldId, issuer_url, audience, jwks_url }) => {
      // OIDC configs are immutable — replace by creating new then deleting old
      const created = await apiFetch<OidcConfigResponse>('/oidc-configs', {
        method: 'POST',
        body: JSON.stringify({ issuer_url, audience, jwks_url }),
      })
      await apiFetch(`/oidc-configs/${oldId}`, { method: 'DELETE' })
      return created
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['oidc-config'] })
    },
  })
}

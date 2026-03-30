import { useQuery, useMutation, useQueryClient } from '@tanstack/vue-query'
import type {
  CreateOidcConfigRequest,
  OidcConfigResponse,
} from './generated/types.gen'
import { apiFetch } from './client'

export type { OidcConfigResponse, CreateOidcConfigRequest }

export function useOidcConfigs() {
  return useQuery({
    queryKey: ['oidc-configs'],
    queryFn: () =>
      apiFetch<{ items: OidcConfigResponse[]; total: number; offset: number; limit: number }>(
        '/oidc-configs?limit=100',
      ),
  })
}

export function useCreateOidcConfig() {
  const queryClient = useQueryClient()

  return useMutation<OidcConfigResponse, Error, CreateOidcConfigRequest>({
    mutationFn: (body) =>
      apiFetch('/oidc-configs', {
        method: 'POST',
        body: JSON.stringify(body),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['oidc-configs'] })
    },
  })
}

export function useDeleteOidcConfig() {
  const queryClient = useQueryClient()

  return useMutation<void, Error, string>({
    mutationFn: (id) =>
      apiFetch(`/oidc-configs/${id}`, { method: 'DELETE' }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['oidc-configs'] })
    },
  })
}

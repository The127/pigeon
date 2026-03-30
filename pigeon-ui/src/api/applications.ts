import { useQuery, useMutation, useQueryClient } from '@tanstack/vue-query'
import type { Ref } from 'vue'
import type {
  ApplicationResponse,
  AppStatsResponse,
  AttemptResponse,
  AuditLogResponse,
  CreateApplicationRequest,
  CreateEndpointRequest,
  CreateEventTypeRequest,
  DeadLetterResponse,
  EndpointResponse,
  EndpointStatsResponse,
  EventTypeResponse,
  EventTypeStatsResponse,
  MessageResponse,
  PaginatedApplicationResponse,
  SendMessageRequest,
  SendMessageResponse,
  UpdateApplicationRequest,
  UpdateEndpointRequest,
  UpdateEventTypeRequest,
} from './generated/types.gen'
import { apiFetch } from './client'

// Re-export generated types used by views
export type {
  ApplicationResponse,
  AppStatsResponse,
  AttemptResponse,
  AuditLogResponse,
  CreateApplicationRequest,
  CreateEndpointRequest,
  CreateEventTypeRequest,
  DeadLetterResponse,
  EndpointResponse,
  EndpointStatsResponse,
  EventTypeResponse,
  EventTypeStatsResponse,
  MessageResponse,
  SendMessageRequest,
  SendMessageResponse,
  UpdateApplicationRequest,
  UpdateEndpointRequest,
  UpdateEventTypeRequest,
}
// CreateEndpointResponse is exported from where it's defined below

// Aliases for generated types used by components under shorter names
export type { TimeBucketResponse as TimeBucket } from './generated/types.gen'
export type { RecentMessageResponse as RecentMessage } from './generated/types.gen'

// --- Applications ---

export function useApplications(search: Ref<string>) {
  return useQuery<PaginatedApplicationResponse>({
    queryKey: ['applications', search],
    queryFn: () => {
      const params = new URLSearchParams({ limit: '100' })
      if (search.value) params.set('search', search.value)
      return apiFetch(`/applications?${params}`)
    },
  })
}

export function useApplication(id: Ref<string>) {
  return useQuery<ApplicationResponse>({
    queryKey: ['applications', id],
    queryFn: () => apiFetch(`/applications/${id.value}`),
  })
}

export function useCreateApplication() {
  const queryClient = useQueryClient()

  return useMutation<ApplicationResponse, Error, CreateApplicationRequest>({
    mutationFn: (body) =>
      apiFetch('/applications', {
        method: 'POST',
        body: JSON.stringify(body),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['applications'] })
    },
  })
}

export function useUpdateApplication(id: Ref<string>) {
  const queryClient = useQueryClient()

  return useMutation<ApplicationResponse, Error, UpdateApplicationRequest>({
    mutationFn: (body) =>
      apiFetch(`/applications/${id.value}`, {
        method: 'PUT',
        body: JSON.stringify(body),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['applications'] })
    },
  })
}

export function useDeleteApplication() {
  const queryClient = useQueryClient()

  return useMutation<void, Error, string>({
    mutationFn: (id) =>
      apiFetch(`/applications/${id}`, { method: 'DELETE' }),
    onSuccess: (_data, id) => {
      queryClient.removeQueries({ queryKey: ['applications', id] })
      queryClient.invalidateQueries({ queryKey: ['applications'] })
    },
  })
}

// --- Stats ---

export function useAppStats(appId: Ref<string>, period: Ref<string>) {
  return useQuery<AppStatsResponse>({
    queryKey: ['applications', appId, 'stats', period],
    queryFn: () =>
      apiFetch(`/applications/${appId.value}/stats?period=${period.value}`),
  })
}

// --- Event Types ---

export function useEventTypes(appId: Ref<string>) {
  return useQuery({
    queryKey: ['applications', appId, 'event-types'],
    queryFn: () =>
      apiFetch<{ items: EventTypeResponse[]; total: number }>(
        `/applications/${appId.value}/event-types?limit=100`,
      ),
  })
}

export function useCreateEventType(appId: Ref<string>) {
  const queryClient = useQueryClient()

  return useMutation<EventTypeResponse, Error, CreateEventTypeRequest>({
    mutationFn: (body) =>
      apiFetch(`/applications/${appId.value}/event-types`, {
        method: 'POST',
        body: JSON.stringify(body),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: ['applications', appId, 'event-types'],
      })
    },
  })
}

export function useUpdateEventType(appId: Ref<string>) {
  const queryClient = useQueryClient()

  return useMutation<EventTypeResponse, Error, { id: string; body: UpdateEventTypeRequest }>({
    mutationFn: ({ id, body }) =>
      apiFetch(`/applications/${appId.value}/event-types/${id}`, {
        method: 'PUT',
        body: JSON.stringify(body),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: ['applications', appId, 'event-types'],
      })
    },
  })
}

export function useDeleteEventType(appId: Ref<string>) {
  const queryClient = useQueryClient()

  return useMutation<void, Error, string>({
    mutationFn: (id) =>
      apiFetch(`/applications/${appId.value}/event-types/${id}`, {
        method: 'DELETE',
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['applications', appId, 'event-types'] })
      queryClient.invalidateQueries({ queryKey: ['applications', appId, 'stats'] })
    },
  })
}

// --- Event Type Stats ---

export function useEventTypeStats(
  appId: Ref<string>,
  eventTypeId: Ref<string>,
  period: Ref<string>,
) {
  return useQuery<EventTypeStatsResponse>({
    queryKey: ['applications', appId, 'event-types', eventTypeId, 'stats', period],
    queryFn: () =>
      apiFetch(
        `/applications/${appId.value}/event-types/${eventTypeId.value}/stats?period=${period.value}`,
      ),
  })
}

// --- Endpoints ---

export function useEndpoints(appId: Ref<string>) {
  return useQuery({
    queryKey: ['applications', appId, 'endpoints'],
    queryFn: () =>
      apiFetch<{ items: EndpointResponse[]; total: number }>(
        `/applications/${appId.value}/endpoints?limit=100`,
      ),
  })
}

export type CreateEndpointResponse = EndpointResponse & { signing_secret: string }

export function useCreateEndpoint(appId: Ref<string>) {
  const queryClient = useQueryClient()

  return useMutation<CreateEndpointResponse, Error, CreateEndpointRequest>({
    mutationFn: (body) =>
      apiFetch(`/applications/${appId.value}/endpoints`, {
        method: 'POST',
        body: JSON.stringify(body),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['applications', appId, 'endpoints'] })
      queryClient.invalidateQueries({ queryKey: ['applications', appId, 'event-types'] })
    },
  })
}

export function useUpdateEndpoint(appId: Ref<string>) {
  const queryClient = useQueryClient()

  return useMutation<EndpointResponse, Error, { id: string; body: UpdateEndpointRequest }>({
    mutationFn: ({ id, body }) =>
      apiFetch(`/applications/${appId.value}/endpoints/${id}`, {
        method: 'PUT',
        body: JSON.stringify(body),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['applications', appId, 'endpoints'] })
      queryClient.invalidateQueries({ queryKey: ['applications', appId, 'event-types'] })
    },
  })
}

export function useDeleteEndpoint(appId: Ref<string>) {
  const queryClient = useQueryClient()

  return useMutation<void, Error, string>({
    mutationFn: (id) =>
      apiFetch(`/applications/${appId.value}/endpoints/${id}`, {
        method: 'DELETE',
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['applications', appId, 'endpoints'] })
      queryClient.invalidateQueries({ queryKey: ['applications', appId, 'event-types'] })
      queryClient.invalidateQueries({ queryKey: ['applications', appId, 'stats'] })
    },
  })
}

// --- Signing Secret Rotation ---

export interface RotateSigningSecretResponse {
  new_secret: string
  signing_secrets_masked: string[]
}

export function useRotateSigningSecret(appId: Ref<string>) {
  const queryClient = useQueryClient()

  return useMutation<RotateSigningSecretResponse, Error, string>({
    mutationFn: (endpointId) =>
      apiFetch(`/applications/${appId.value}/endpoints/${endpointId}/rotate`, {
        method: 'POST',
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['applications', appId, 'endpoints'] })
    },
  })
}

export function useRevokeSigningSecret(appId: Ref<string>) {
  const queryClient = useQueryClient()

  return useMutation<void, Error, { endpointId: string; index: number }>({
    mutationFn: ({ endpointId, index }) =>
      apiFetch(`/applications/${appId.value}/endpoints/${endpointId}/secrets/${index}`, {
        method: 'DELETE',
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['applications', appId, 'endpoints'] })
    },
  })
}

// --- Endpoint Stats ---

export function useEndpointStats(
  appId: Ref<string>,
  endpointId: Ref<string>,
  period: Ref<string>,
) {
  return useQuery<EndpointStatsResponse>({
    queryKey: ['applications', appId, 'endpoints', endpointId, 'stats', period],
    queryFn: () =>
      apiFetch(
        `/applications/${appId.value}/endpoints/${endpointId.value}/stats?period=${period.value}`,
      ),
  })
}

// --- Messages ---

export function useSendMessage(appId: Ref<string>) {
  const queryClient = useQueryClient()

  return useMutation<SendMessageResponse, Error, SendMessageRequest>({
    mutationFn: (body) =>
      apiFetch(`/applications/${appId.value}/messages`, {
        method: 'POST',
        body: JSON.stringify(body),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['applications', appId, 'messages'] })
      queryClient.invalidateQueries({ queryKey: ['applications', appId, 'stats'] })
    },
  })
}

export function useRetriggerMessage(appId: Ref<string>) {
  const queryClient = useQueryClient()

  return useMutation<unknown, Error, string>({
    mutationFn: (messageId) =>
      apiFetch(`/applications/${appId.value}/messages/${messageId}/retrigger`, {
        method: 'POST',
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['applications', appId, 'messages'] })
      queryClient.invalidateQueries({ queryKey: ['applications', appId, 'stats'] })
    },
  })
}

export function useMessages(appId: Ref<string>, eventTypeId: Ref<string>, status: Ref<string>) {
  return useQuery({
    queryKey: ['applications', appId, 'messages', eventTypeId, status],
    queryFn: () => {
      const params = new URLSearchParams({ limit: '50' })
      if (eventTypeId.value) params.set('event_type_id', eventTypeId.value)
      if (status.value) params.set('status', status.value)
      return apiFetch<{ items: MessageResponse[]; total: number }>(
        `/applications/${appId.value}/messages?${params}`,
      )
    },
  })
}

export function useAttempts(appId: Ref<string>, messageId: Ref<string | null>) {
  return useQuery({
    queryKey: ['applications', appId, 'messages', messageId, 'attempts'],
    queryFn: () =>
      apiFetch<AttemptResponse[]>(
        `/applications/${appId.value}/messages/${messageId.value}/attempts`,
      ),
    enabled: () => !!messageId.value,
  })
}

// --- Dead Letters ---

export function useDeadLetters(appId: Ref<string>, endpointId: Ref<string>, replayed: Ref<string>) {
  return useQuery({
    queryKey: ['applications', appId, 'dead-letters', endpointId, replayed],
    queryFn: () => {
      const params = new URLSearchParams({ limit: '50' })
      if (endpointId.value) params.set('endpoint_id', endpointId.value)
      if (replayed.value) params.set('replayed', replayed.value)
      return apiFetch<{ items: DeadLetterResponse[]; total: number }>(
        `/applications/${appId.value}/dead-letters?${params}`,
      )
    },
  })
}

export function useReplayDeadLetter(appId: Ref<string>) {
  const queryClient = useQueryClient()

  return useMutation<unknown, Error, string>({
    mutationFn: (id) =>
      apiFetch(`/applications/${appId.value}/dead-letters/${id}/replay`, {
        method: 'POST',
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['applications', appId, 'dead-letters'] })
      queryClient.invalidateQueries({ queryKey: ['applications', appId, 'messages'] })
      queryClient.invalidateQueries({ queryKey: ['applications', appId, 'stats'] })
    },
  })
}

// --- Audit Log ---

export function useAuditLog(
  offset: Ref<number>,
  limit: Ref<number>,
  commandFilter: Ref<string>,
  successFilter: Ref<string>,
) {
  return useQuery({
    queryKey: ['audit-log', offset, limit, commandFilter, successFilter],
    queryFn: () => {
      const params = new URLSearchParams({
        offset: String(offset.value),
        limit: String(limit.value),
      })
      if (commandFilter.value) params.set('command', commandFilter.value)
      if (successFilter.value) params.set('success', successFilter.value)
      return apiFetch<{ items: AuditLogResponse[]; total: number; offset: number; limit: number }>(
        `/audit-log?${params}`,
      )
    },
  })
}

import { useQuery, useMutation, useQueryClient } from '@tanstack/vue-query'
import type { Ref } from 'vue'
import type {
  ApplicationResponse,
  CreateApplicationRequest,
  UpdateApplicationRequest,
  PaginatedApplicationResponse,
  EventTypeResponse,
  CreateEventTypeRequest,
  UpdateEventTypeRequest,
  EndpointResponse,
  CreateEndpointRequest,
  UpdateEndpointRequest,
  SendMessageRequest,
  SendMessageResponse,
} from './generated/types.gen'

// --- Stats ---

export interface TimeBucket {
  bucket: string
  succeeded: number
  failed: number
}

export interface AppStatsResponse {
  total_messages: number
  total_attempts: number
  total_succeeded: number
  total_failed: number
  total_dead_lettered: number
  success_rate: number
  time_series: TimeBucket[]
}

export function useAppStats(appId: Ref<string>, period: Ref<string>) {
  return useQuery<AppStatsResponse>({
    queryKey: ['applications', appId, 'stats', period],
    queryFn: () =>
      apiFetch(`/applications/${appId.value}/stats?period=${period.value}`),
  })
}

// --- Types not yet in generated client (pending server restart) ---

// Endpoint types with name field (pending server restart + client regen)
export interface EndpointWithName {
  id: string
  app_id: string
  name: string
  url: string
  enabled: boolean
  event_type_ids: string[]
  created_at: string
  version: number
}

export interface MessageResponse {
  id: string
  app_id: string
  event_type_id: string
  payload: unknown
  idempotency_key: string
  created_at: string
  attempts_created: number
  succeeded: number
  failed: number
  dead_lettered: number
}

export interface AttemptResponse {
  id: string
  message_id: string
  endpoint_id: string
  status: string
  response_code: number | null
  response_body: string | null
  attempted_at: string | null
  next_attempt_at: string | null
  attempt_number: number
  duration_ms: number | null
}

export interface DeadLetterResponse {
  id: string
  message_id: string
  endpoint_id: string
  app_id: string
  last_response_code: number | null
  last_response_body: string | null
  dead_lettered_at: string
  replayed_at: string | null
}
import { apiFetch } from './client'

// --- Applications ---

export function useApplications() {
  return useQuery<PaginatedApplicationResponse>({
    queryKey: ['applications'],
    queryFn: () => apiFetch('/applications?limit=100'),
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
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['applications'] })
    },
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
      queryClient.invalidateQueries({
        queryKey: ['applications', appId, 'event-types'],
      })
    },
  })
}

// --- Endpoints ---

export function useEndpoints(appId: Ref<string>) {
  return useQuery({
    queryKey: ['applications', appId, 'endpoints'],
    queryFn: () =>
      apiFetch<{ items: EndpointWithName[]; total: number }>(
        `/applications/${appId.value}/endpoints?limit=100`,
      ),
  })
}

export function useCreateEndpoint(appId: Ref<string>) {
  const queryClient = useQueryClient()

  return useMutation<EndpointWithName, Error, CreateEndpointRequest & { name?: string }>({
    mutationFn: (body) =>
      apiFetch(`/applications/${appId.value}/endpoints`, {
        method: 'POST',
        body: JSON.stringify(body),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: ['applications', appId, 'endpoints'],
      })
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
      queryClient.invalidateQueries({
        queryKey: ['applications', appId, 'endpoints'],
      })
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
      queryClient.invalidateQueries({
        queryKey: ['applications', appId, 'endpoints'],
      })
    },
  })
}

// --- Messages ---

export function useSendMessage(appId: Ref<string>) {
  return useMutation<SendMessageResponse, Error, SendMessageRequest>({
    mutationFn: (body) =>
      apiFetch(`/applications/${appId.value}/messages`, {
        method: 'POST',
        body: JSON.stringify(body),
      }),
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
      queryClient.invalidateQueries({
        queryKey: ['applications', appId, 'messages'],
      })
    },
  })
}

export function useMessages(appId: Ref<string>) {
  return useQuery({
    queryKey: ['applications', appId, 'messages'],
    queryFn: () =>
      apiFetch<{ items: MessageResponse[]; total: number }>(
        `/applications/${appId.value}/messages?limit=50`,
      ),
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

export function useDeadLetters(appId: Ref<string>) {
  return useQuery({
    queryKey: ['applications', appId, 'dead-letters'],
    queryFn: () =>
      apiFetch<{ items: DeadLetterResponse[]; total: number }>(
        `/applications/${appId.value}/dead-letters?limit=50`,
      ),
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
      queryClient.invalidateQueries({
        queryKey: ['applications', appId, 'dead-letters'],
      })
    },
  })
}

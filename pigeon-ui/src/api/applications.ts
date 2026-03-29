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
} from './generated/types.gen'
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
      apiFetch<{ items: EndpointResponse[]; total: number }>(
        `/applications/${appId.value}/endpoints?limit=100`,
      ),
  })
}

export function useCreateEndpoint(appId: Ref<string>) {
  const queryClient = useQueryClient()

  return useMutation<EndpointResponse, Error, CreateEndpointRequest>({
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

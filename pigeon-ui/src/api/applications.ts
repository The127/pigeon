import { useQuery, useMutation, useQueryClient } from '@tanstack/vue-query'
import type {
  ApplicationResponse,
  CreateApplicationRequest,
  PaginatedApplicationResponse,
} from './generated/types.gen'
import { apiFetch } from './client'

export function useApplications() {
  return useQuery<PaginatedApplicationResponse>({
    queryKey: ['applications'],
    queryFn: () => apiFetch('/applications?limit=100'),
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

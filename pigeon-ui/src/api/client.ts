import { useAuth } from '@/auth'

const BASE_URL = '/api/v1'

async function getAuthHeaders(): Promise<Record<string, string>> {
  const { user } = useAuth()
  const token = user.value?.access_token
  if (!token) return {}
  return { Authorization: `Bearer ${token}` }
}

export async function apiFetch<T>(
  path: string,
  options: RequestInit = {},
): Promise<T> {
  const authHeaders = await getAuthHeaders()
  const res = await fetch(`${BASE_URL}${path}`, {
    ...options,
    headers: {
      'Content-Type': 'application/json',
      ...authHeaders,
      ...options.headers,
    },
  })

  if (!res.ok) {
    const body = await res.json().catch(() => ({ error: res.statusText }))
    throw new ApiError(res.status, body.error || res.statusText)
  }

  if (res.status === 204) return undefined as T
  return res.json()
}

export class ApiError extends Error {
  constructor(
    public status: number,
    message: string,
  ) {
    super(message)
    this.name = 'ApiError'
  }
}

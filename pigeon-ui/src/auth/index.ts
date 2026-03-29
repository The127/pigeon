import { UserManager, WebStorageStateStore, type User } from 'oidc-client-ts'
import { ref } from 'vue'

let userManager: UserManager | null = null
const currentUser = ref<User | null>(null)

interface AuthConfig {
  issuer_url: string
  audience: string
}

async function fetchAuthConfig(): Promise<AuthConfig> {
  const res = await fetch('/api/v1/auth/config')
  if (!res.ok) throw new Error('Failed to fetch auth config')
  return res.json()
}

function buildUserManager(config: AuthConfig): UserManager {
  const origin = window.location.origin
  return new UserManager({
    authority: config.issuer_url,
    client_id: config.audience,
    redirect_uri: `${origin}/auth/callback`,
    post_logout_redirect_uri: origin,
    response_type: 'code',
    scope: 'openid profile email',
    userStore: new WebStorageStateStore({ store: window.localStorage }),
  })
}

async function ensureUserManager(): Promise<UserManager> {
  if (!userManager) {
    const config = await fetchAuthConfig()
    userManager = buildUserManager(config)
  }
  return userManager
}

export async function initAuth(): Promise<void> {
  const um = await ensureUserManager()
  currentUser.value = await um.getUser()
}

export function isAuthenticated(): boolean {
  const user = currentUser.value
  return !!user && !user.expired
}

export function useAuth() {
  const login = async () => (await ensureUserManager()).signinRedirect()
  const logout = async () => (await ensureUserManager()).signoutRedirect()

  return {
    user: currentUser,
    isAuthenticated,
    login,
    logout,
  }
}

export async function handleCallback(): Promise<User> {
  const um = await ensureUserManager()
  const user = await um.signinRedirectCallback()
  currentUser.value = user
  return user
}

import { createRouter, createWebHistory } from 'vue-router'
import { isAuthenticated } from '@/auth'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/login',
      name: 'login',
      component: () => import('@/views/LoginView.vue'),
      meta: { public: true },
    },
    {
      path: '/auth/callback',
      name: 'auth-callback',
      component: () => import('@/views/AuthCallbackView.vue'),
      meta: { public: true },
    },
    {
      path: '/',
      redirect: '/apps',
    },
    {
      path: '/audit-log',
      name: 'audit-log',
      component: () => import('@/views/AuditLogView.vue'),
    },
    {
      path: '/apps',
      name: 'applications',
      component: () => import('@/views/ApplicationsView.vue'),
    },
    {
      path: '/apps/:id',
      name: 'application-detail',
      component: () => import('@/views/ApplicationDetailView.vue'),
    },
    {
      path: '/apps/:id/event-types/:etId',
      name: 'event-type-detail',
      component: () => import('@/views/EventTypeDetailView.vue'),
    },
    {
      path: '/apps/:id/endpoints/:epId',
      name: 'endpoint-detail',
      component: () => import('@/views/EndpointDetailView.vue'),
    },
  ],
})

router.beforeEach((to) => {
  if (to.meta.public) return true
  if (!isAuthenticated()) {
    return { name: 'login', query: { redirect: to.fullPath } }
  }
  return true
})

export default router

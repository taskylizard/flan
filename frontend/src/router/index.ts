import { createRouter, createWebHistory } from 'vue-router'
import { useUserStore } from '../stores/user'
import HomeView from '../views/HomeView.vue'
import LoginView from '../views/LoginView.vue'

declare module 'vue-router' {
  interface RouteMeta {
    requiresAuth?: boolean
    guestOnly?: boolean
  }
}

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      component: HomeView,
      meta: { requiresAuth: true },
    },
    {
      path: '/login',
      component: LoginView,
      meta: { guestOnly: true },
    },
  ],
})

router.beforeEach((to, from, next) => {
  const userStore = useUserStore()
  const isAuthenticated = userStore.isAuthenticated

  if (to.meta.requiresAuth && !isAuthenticated) {
    next('/login')
    return
  }

  if (to.meta.guestOnly && isAuthenticated) {
    next('/')
    return
  }

  next()
})

export default router

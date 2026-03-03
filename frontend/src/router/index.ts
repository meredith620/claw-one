import { createRouter, createWebHistory } from 'vue-router'
import ConfigView from '../views/ConfigView.vue'
import StatusView from '../views/StatusView.vue'
import SafeModeView from '../views/SafeModeView.vue'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      redirect: '/status',
    },
    {
      path: '/status',
      name: 'Status',
      component: StatusView,
    },
    {
      path: '/config',
      name: 'Config',
      component: ConfigView,
    },
    {
      path: '/safe-mode',
      name: 'SafeMode',
      component: SafeModeView,
    },
  ],
})

export default router

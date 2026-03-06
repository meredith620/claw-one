import { createRouter, createWebHistory } from 'vue-router'
import ConfigView from '../views/ConfigView.vue'
import StatusView from '../views/StatusView.vue'
import SafeModeView from '../views/SafeModeView.vue'
import SetupWizard from '../views/SetupWizard.vue'
import ProviderConfigView from '../views/ProviderConfigView.vue'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      redirect: '/setup',
    },
    {
      path: '/setup',
      name: 'Setup',
      component: SetupWizard,
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
      path: '/config/provider',
      name: 'ProviderConfig',
      component: ProviderConfigView,
    },
    {
      path: '/safe-mode',
      name: 'SafeMode',
      component: SafeModeView,
    },
  ],
})

export default router

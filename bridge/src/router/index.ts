import { createRouter, createWebHistory } from 'vue-router'
import ConfigView from '../views/ConfigView.vue'
import StatusView from '../views/StatusView.vue'
import SafeModeView from '../views/SafeModeView.vue'
import SetupWizard from '../views/SetupWizard.vue'
import ProviderConfigView from '../views/ProviderConfigView.vue'
import ConfigLayout from '../views/ConfigLayout.vue'
import ProviderModule from '../views/ProviderModule.vue'
import AgentModule from '../views/AgentModule.vue'
import MemoryModule from '../views/MemoryModule.vue'
import ChannelModule from '../views/ChannelModule.vue'

import SnapshotsView from '../views/SnapshotsView.vue'

import SnapshotsView from '../views/SnapshotsView.vue'

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
      path: '/config-old',
      name: 'ConfigOld',
      component: ConfigView,
    },
    {
      path: '/config',
      component: ConfigLayout,
      redirect: '/config/provider',
      children: [
        {
          path: 'provider',
          name: 'ProviderConfig',
          component: ProviderModule,
        },
        {
          path: 'agent',
          name: 'AgentConfig',
          component: AgentModule,
        },
        {
          path: 'memory',
          name: 'MemoryConfig',
          component: MemoryModule,
        },
        {
          path: 'channel',
          name: 'ChannelConfig',
          component: ChannelModule,
        },
      ],
    },
    {
      path: '/config/provider-old',
      name: 'ProviderConfigOld',
      component: ProviderConfigView,
    },
    {
      path: '/safe-mode',
      name: 'SafeMode',
      component: SafeModeView,
    },
    {
      path: '/snapshots',
      name: 'Snapshots',
      component: SnapshotsView,
    },
    {
      path: '/snapshots',
      name: 'Snapshots',
      component: SnapshotsView,
    },
  ],
})

export default router

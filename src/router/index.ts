import { createRouter, createWebHistory } from 'vue-router';

const routes = [
  {
    path: '/',
    name: 'Todo',
    component: () => import('../views/TodoList.vue'),
  },
  {
    path: '/weekly',
    name: 'WeeklyReport',
    component: () => import('../views/WeeklyReport.vue'),
  },
  {
    path: '/projects',
    name: 'Projects',
    component: () => import('../views/ProjectList.vue'),
  },
  {
    path: '/project/:id',
    name: 'ProjectDetail',
    component: () => import('../views/ProjectDetail.vue'),
  },
  {
    path: '/servers',
    name: 'Servers',
    component: () => import('../components/server/ServerManager.vue'),
  },
  {
    path: '/database',
    name: 'Database',
    component: () => import('../views/DBManager.vue'),
  },
  {
    path: '/lan',
    name: 'LAN',
    component: () => import('../views/LanUsers.vue'),
  },
  {
    path: '/cicd',
    name: 'CICD',
    component: () => import('../views/CiCdConfig.vue'),
  },
  {
    path: '/deploy',
    name: 'Deploy',
    component: () => import('../components/cicd/DeployPanel.vue'),
  },
  {
    path: '/git',
    name: 'Git',
    component: () => import('../views/GitManager.vue'),
  },
  {
    path: '/notes',
    name: 'Notes',
    component: () => import('../views/NoteManager.vue'),
  },
  {
    path: '/logs',
    name: 'Logs',
    component: () => import('../views/LogAggregator.vue'),
  },
  {
    path: '/vpn',
    name: 'VPN',
    component: () => import('../views/VPNManager.vue'),
  },
  {
    path: '/backup',
    name: 'Backup',
    component: () => import('../views/DataBackup.vue'),
  },
  {
    path: '/mfa',
    name: 'MFA',
    component: () => import('../views/MfaManager.vue'),
  },
  {
    path: '/accounting',
    name: 'Accounting',
    component: () => import('../views/AccountingBook.vue'),
  },
  {
    path: '/report',
    name: 'Report',
    component: () => import('../views/TodoReport.vue'),
  },
  {
    path: '/devtools',
    name: 'DevTools',
    component: () => import('../components/devtools/DevTools.vue'),
  },
  {
    path: '/settings',
    name: 'Settings',
    component: () => import('../views/SettingsView.vue'),
  },
];

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes,
});

export default router;

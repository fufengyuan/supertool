import { createRouter, createWebHistory } from 'vue-router';
import MainLayout from '../layouts/MainLayout.vue';

const routes = [
  {
    path: '/',
    component: MainLayout,
    children: [
      {
        path: '',
        name: 'Todo',
        component: () => import('../components/todo/TodoList.vue'),
      },
      {
        path: 'weekly',
        name: 'WeeklyReport',
        component: () => import('../components/WeeklyReport.vue'),
      },
      {
        path: 'projects',
        name: 'Projects',
        component: () => import('../components/ProjectList.vue'),
      },
      {
        path: 'project/:id',
        name: 'ProjectDetail',
        component: () => import('../components/ProjectDetail.vue'),
      },
      {
        path: 'servers',
        name: 'Servers',
        component: () => import('../components/server/ServerManager.vue'),
      },
      {
        path: 'database',
        name: 'Database',
        component: () => import('../components/db/DBManager.vue'),
      },
      {
        path: 'cicd',
        name: 'CICD',
        component: () => import('../components/cicd/CiCdConfig.vue'),
      },
      {
        path: 'git',
        name: 'Git',
        component: () => import('../components/GitRepoList.vue'),
      },
      {
        path: 'notes',
        name: 'Notes',
        component: () => import('../components/NoteManager.vue'),
      },
      {
        path: 'logs',
        name: 'Logs',
        component: () => import('../components/LogAggregator.vue'),
      },
      {
        path: 'vpn',
        name: 'VPN',
        component: () => import('../components/VPNManager.vue'),
      },
      {
        path: 'backup',
        name: 'Backup',
        component: () => import('../components/DataBackup.vue'),
      },
      {
        path: 'mfa',
        name: 'MFA',
        component: () => import('../components/MfaManager.vue'),
      },
      {
        path: 'accounting',
        name: 'Accounting',
        component: () => import('../components/AccountingBook.vue'),
      },
      {
        path: 'report',
        name: 'Report',
        component: () => import('../components/TodoReport.vue'),
      },
      {
        path: 'devtools',
        name: 'DevTools',
        component: () => import('../components/devtools/DevTools.vue'),
      },
      {
        path: 'settings',
        name: 'Settings',
        component: () => import('../views/SettingsView.vue'),
      },
    ],
  },
];

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes,
});

export default router;

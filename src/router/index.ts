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
        component: () => import('../views/todo/TodoList.vue'),
      },
      {
        path: 'weekly',
        name: 'WeeklyReport',
        component: () => import('../views/WeeklyReport.vue'),
      },
      {
        path: 'projects',
        name: 'Projects',
        component: () => import('../views/ProjectList.vue'),
      },
      {
        path: 'project/:id',
        name: 'ProjectDetail',
        component: () => import('../views/ProjectDetail.vue'),
      },
      {
        path: 'servers',
        name: 'Servers',
        component: () => import('../views/server/ServerManager.vue'),
      },
      {
        path: 'database',
        name: 'Database',
        component: () => import('../views/db/DBManager.vue'),
      },
      {
        path: 'cicd',
        name: 'CICD',
        component: () => import('../views/cicd/CiCdConfig.vue'),
      },
      {
        path: 'git',
        name: 'Git',
        component: () => import('../views/GitRepoList.vue'),
      },
      {
        path: 'notes',
        name: 'Notes',
        component: () => import('../views/NoteManager.vue'),
      },
      {
        path: 'logs',
        name: 'Logs',
        component: () => import('../views/LogAggregator.vue'),
      },
      {
        path: 'vpn',
        name: 'VPN',
        component: () => import('../views/VPNManager.vue'),
      },
      {
        path: 'backup',
        name: 'Backup',
        component: () => import('../views/DataBackup.vue'),
      },
      {
        path: 'mfa',
        name: 'MFA',
        component: () => import('../views/MfaManager.vue'),
      },
      {
        path: 'accounting',
        name: 'Accounting',
        component: () => import('../views/AccountingBook.vue'),
      },
      {
        path: 'report',
        name: 'Report',
        component: () => import('../views/TodoReport.vue'),
      },
      {
        path: 'devtools',
        name: 'DevTools',
        component: () => import('../views/devtools/DevTools.vue'),
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

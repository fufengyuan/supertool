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
        component: () => import('../views/weekly/WeeklyReport.vue'),
      },
      {
        path: 'projects',
        name: 'Projects',
        component: () => import('../views/projects/ProjectList.vue'),
      },
      {
        path: 'project/:id',
        name: 'ProjectDetail',
        component: () => import('../views/projects/ProjectDetail.vue'),
        props: true,
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
        path: 'alert',
        name: 'Alert',
        component: () => import('../views/alert/AlertView.vue'),
      },
      {
        path: 'cicd',
        name: 'CICD',
        component: () => import('../views/cicd/CiCdConfig.vue'),
      },
      {
        path: 'git',
        name: 'Git',
        component: () => import('../views/git/GitRepoList.vue'),
      },
      {
        path: 'notes',
        name: 'Notes',
        component: () => import('../views/notes/NoteManager.vue'),
      },
      {
        path: 'logs',
        name: 'Logs',
        component: () => import('../views/logs/LogAggregator.vue'),
      },
      {
        path: 'nginx',
        name: 'Nginx',
        component: () => import('../views/nginx/NginxManager.vue'),
      },
      {
        path: 'vpn',
        name: 'VPN',
        component: () => import('../views/vpn/VPNManager.vue'),
      },
      {
        path: 'backup',
        name: 'Backup',
        component: () => import('../views/backup/DataBackup.vue'),
      },
      {
        path: 'disk-cleaner',
        name: 'DiskCleaner',
        component: () => import('../components/DiskCleaner.vue'),
      },
      {
        path: 'mfa',
        name: 'MFA',
        component: () => import('../views/mfa/MfaManager.vue'),
      },
      {
        path: 'accounting',
        name: 'Accounting',
        component: () => import('../views/accounting/AccountingBook.vue'),
      },
      {
        path: 'report',
        name: 'Report',
        component: () => import('../views/reports/TodoReport.vue'),
      },
      {
        path: 'devtools',
        name: 'DevTools',
        component: () => import('../views/devtools/DevTools.vue'),
      },
      {
        path: 'agent',
        name: 'Agent',
        component: () => import('../views/agent/AgentManager.vue'),
      },
      {
        path: 'agent/chat',
        name: 'AgentChat',
        component: () => import('../views/agent/HermesChat.vue'),
      },
      {
        path: 'settings',
        name: 'Settings',
        component: () => import('../views/settings/SettingsView.vue'),
      },
    ],
  },
];

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes,
});

export default router;

import { createRouter, createWebHistory } from 'vue-router';
import MainLayout from '../layouts/MainLayout.vue';

const routes = [
  {
    path: '/',
    component: MainLayout,
    children: [
      {
        path: '',
        name: 'Dashboard',
        component: () => import('../views/dashboard/Dashboard.vue'),
      },
      {
        path: 'todo',
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
        redirect: '/agent/sessions',
      },
      {
        path: 'agent/chat',
        name: 'AgentChat',
        component: () => import('../views/agent/chat/Chat.vue'),
      },
      {
        path: 'agent/profiles',
        name: 'AgentProfiles',
        component: () => import('../views/agent/AgentProfiles.vue'),
      },
      {
        path: 'agent/tools',
        name: 'AgentTools',
        component: () => import('../views/agent/ToolsManager.vue'),
      },
      {
        path: 'agent/cron',
        name: 'AgentCron',
        component: () => import('../views/agent/CronManager.vue'),
      },
      {
        path: 'agent/skills',
        name: 'AgentSkills',
        component: () => import('../views/agent/SkillsBrowser.vue'),
      },
      {
        path: 'agent/memory',
        name: 'AgentMemory',
        component: () => import('../views/agent/MemoryManager.vue'),
      },
      {
        path: 'agent/sessions',
        name: 'AgentSessions',
        component: () => import('../views/agent/SessionsPage.vue'),
      },
      {
        path: 'agent/settings',
        name: 'AgentSettings',
        component: () => import('../views/agent/SettingsPage.vue'),
      },
      {
        path: 'agent/providers',
        name: 'AgentProviders',
        component: () => import('../views/agent/ProviderManager.vue'),
      },
      {
        path: 'agent/models',
        name: 'AgentModels',
        component: () => import('../views/agent/ModelsPage.vue'),
      },
      {
        path: 'kanban',
        name: 'Kanban',
        component: () => import('../components/kanban/KanbanBoard.vue'),
      },
      {
        path: 'image',
        name: 'ImageProcessor',
        meta: { title: '图像处理' },
        component: () => import('../views/image/ImageProcessor.vue'),
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

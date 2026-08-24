<template>
  <div class="flex flex-col gap-4 p-4 lg:p-6">
    <!-- 顶部欢迎 -->
    <div class="flex items-center justify-between">
      <div>
        <h1 class="text-xl font-semibold text-base-content">综合看板</h1>
        <p class="text-sm text-base-content/50 mt-0.5">{{ greeting }}</p>
      </div>
      <div class="text-right text-sm text-base-content/40">
        <span>{{ currentDate }}</span>
        <span class="ml-2">{{ currentTime }}</span>
      </div>
    </div>

    <!-- 核心数据卡片 -->
    <div class="grid grid-cols-2 lg:grid-cols-4 gap-3">
      <!-- 待办统计 -->
      <div 
        class="bg-base-100 rounded-xl border border-base-content/10 p-4 cursor-pointer hover:border-primary/50 hover:bg-base-200/50 transition-all"
        @click="navigateTo('/todo')"
      >
        <div class="flex items-center gap-2 mb-3">
          <SvgIcon name="checklist" size="16" class="text-primary" />
          <span class="text-sm font-medium text-base-content/70">待办事项</span>
        </div>
        <div class="flex items-end justify-between">
          <div>
            <span class="text-2xl font-bold text-base-content">{{ todoStats.active }}</span>
            <span class="text-xs text-base-content/50 ml-1">进行中</span>
          </div>
          <div class="text-right">
            <span class="text-sm text-success">{{ todoStats.completed }}</span>
            <span class="text-xs text-base-content/50 ml-1">已完成</span>
          </div>
        </div>
        <div class="mt-3 h-1.5 bg-base-200 rounded-full overflow-hidden">
          <div 
            class="h-full bg-primary rounded-full transition-all duration-500"
            :style="{ width: todoStats.progressPercent + '%' }"
          ></div>
        </div>
      </div>

      <!-- 服务器状态 -->
      <div 
        class="bg-base-100 rounded-xl border border-base-content/10 p-4 cursor-pointer hover:border-primary/50 hover:bg-base-200/50 transition-all"
        @click="navigateTo('/servers')"
      >
        <div class="flex items-center gap-2 mb-3">
          <SvgIcon name="serverRack" size="16" class="text-info" />
          <span class="text-sm font-medium text-base-content/70">服务器</span>
        </div>
        <div class="flex items-end justify-between">
          <div>
            <span class="text-2xl font-bold text-success">{{ serverStats.total }}</span>
            <span class="text-xs text-base-content/50 ml-1">已配置</span>
          </div>
          <div class="text-right">
            <span class="text-xs text-base-content/40" v-if="serverStats.total > 0">未做连通性检测</span>
          </div>
        </div>
        <div class="mt-3 text-xs text-base-content/40">{{ serverStats.total }} 台服务器</div>
      </div>

      <!-- 项目进度 -->
      <div 
        class="bg-base-100 rounded-xl border border-base-content/10 p-4 cursor-pointer hover:border-primary/50 hover:bg-base-200/50 transition-all"
        @click="navigateTo('/projects')"
      >
        <div class="flex items-center gap-2 mb-3">
          <SvgIcon name="folder" size="16" class="text-warning" />
          <span class="text-sm font-medium text-base-content/70">项目</span>
        </div>
        <div class="flex items-end justify-between">
          <div>
            <span class="text-2xl font-bold text-base-content">{{ projectStats.active }}</span>
            <span class="text-xs text-base-content/50 ml-1">活跃</span>
          </div>
        </div>
        <div class="mt-3 space-y-1" v-if="projectStats.topProjects.length > 0">
          <div 
            v-for="p in projectStats.topProjects.slice(0, 2)" 
            :key="p.id"
            class="flex items-center gap-1.5 text-xs"
          >
            <span class="w-1.5 h-1.5 rounded-full" :style="{ backgroundColor: p.color || '#6366f1' }"></span>
            <span class="text-base-content/60 truncate flex-1">{{ p.name }}</span>
            <span class="text-base-content/40">{{ p.progress }}%</span>
          </div>
        </div>
      </div>

      <!-- 告警 -->
      <div 
        class="bg-base-100 rounded-xl border border-base-content/10 p-4 cursor-pointer hover:border-primary/50 hover:bg-base-200/50 transition-all"
        :class="{ 'border-error/30': alertStats.today > 0 }"
        @click="navigateTo('/alert')"
      >
        <div class="flex items-center gap-2 mb-3">
          <SvgIcon name="bell" size="16" :class="alertStats.today > 0 ? 'text-error animate-pulse' : 'text-base-content/50'" />
          <span class="text-sm font-medium text-base-content/70">告警</span>
        </div>
        <div class="flex items-end justify-between">
          <div>
            <span class="text-2xl font-bold" :class="alertStats.today > 0 ? 'text-error' : 'text-base-content'">{{ alertStats.today }}</span>
            <span class="text-xs text-base-content/50 ml-1">今日告警</span>
          </div>
        </div>
        <div class="mt-3 text-xs text-base-content/40" v-if="alertStats.total > 0">
          共 {{ alertStats.total }} 条告警记录
        </div>
      </div>
    </div>

    <!-- 快速操作 -->
    <div class="bg-base-100 rounded-xl border border-base-content/10 p-4">
      <div class="flex items-center gap-2 mb-3">
        <SvgIcon name="zap" size="16" class="text-warning" />
        <span class="text-sm font-medium text-base-content/70">快速操作</span>
      </div>
      <div class="flex flex-wrap gap-2">
        <button class="btn btn-sm btn-primary gap-1.5" @click="openQuickTodo">
          <SvgIcon name="plus" size="14" />
          添加待办
        </button>
        <button class="btn btn-sm btn-ghost gap-1.5" @click="navigateTo('/cicd')">
          <SvgIcon name="rocket" size="14" />
          CI/CD
        </button>
        <button class="btn btn-sm btn-ghost gap-1.5" @click="navigateTo('/logs')">
          <SvgIcon name="fileText" size="14" />
          日志搜索
        </button>
        <button class="btn btn-sm btn-ghost gap-1.5" @click="navigateTo('/database')">
          <SvgIcon name="database" size="14" />
          数据库
        </button>
        <button class="btn btn-sm btn-ghost gap-1.5" @click="navigateTo('/notes')">
          <SvgIcon name="file" size="14" />
          笔记
        </button>
      </div>
    </div>

    <!-- 最近活动 -->
    <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
      <!-- 最近待办 -->
      <div class="bg-base-100 rounded-xl border border-base-content/10 p-4">
        <div class="flex items-center justify-between mb-3">
          <div class="flex items-center gap-2">
            <SvgIcon name="checklist" size="14" class="text-primary" />
            <span class="text-sm font-medium text-base-content/70">最近待办</span>
          </div>
          <button class="btn btn-ghost btn-xs" @click="navigateTo('/todo')">查看全部</button>
        </div>
        <div class="space-y-2" v-if="recentTodos.length > 0">
          <div 
            v-for="todo in recentTodos" 
            :key="todo.id"
            class="flex items-center gap-2 px-2 py-1.5 rounded-lg hover:bg-base-200/50 transition-colors"
          >
            <span 
              class="w-2 h-2 rounded-full shrink-0"
              :class="todo.completed ? 'bg-success' : (todo.priority === 'high' ? 'bg-error' : todo.priority === 'medium' ? 'bg-warning' : 'bg-base-content/30')"
            ></span>
            <span 
              class="text-sm truncate flex-1"
              :class="todo.completed ? 'text-base-content/40 line-through' : 'text-base-content'"
            >{{ todo.text }}</span>
            <span v-if="todo.dueDate" class="text-xs text-base-content/40">{{ formatDueDate(todo.dueDate) }}</span>
          </div>
        </div>
        <div class="text-center py-6 text-base-content/40" v-else>
          <SvgIcon name="inbox" size="24" class="mb-2 opacity-50" />
          <p class="text-sm">暂无待办</p>
        </div>
      </div>

      <!-- 最近部署 -->
      <div class="bg-base-100 rounded-xl border border-base-content/10 p-4">
        <div class="flex items-center justify-between mb-3">
          <div class="flex items-center gap-2">
            <SvgIcon name="rocket" size="14" class="text-info" />
            <span class="text-sm font-medium text-base-content/70">最近部署</span>
          </div>
          <button class="btn btn-ghost btn-xs" @click="navigateTo('/cicd')">查看全部</button>
        </div>
        <div class="space-y-2" v-if="recentDeployments.length > 0">
          <div 
            v-for="deploy in recentDeployments" 
            :key="deploy.id"
            class="flex items-center gap-2 px-2 py-1.5 rounded-lg hover:bg-base-200/50 transition-colors"
          >
            <span 
              class="w-2 h-2 rounded-full shrink-0"
              :class="deploy.status === 'success' ? 'bg-success' : deploy.status === 'failed' ? 'bg-error' : 'bg-warning'"
            ></span>
            <span class="text-sm truncate flex-1 text-base-content">{{ deploy.configName || '部署' }}</span>
            <span class="text-xs text-base-content/40">{{ formatTime(deploy.createdAt) }}</span>
          </div>
        </div>
        <div class="text-center py-6 text-base-content/40" v-else>
          <SvgIcon name="rocket" size="24" class="mb-2 opacity-50" />
          <p class="text-sm">暂无部署记录</p>
        </div>
      </div>
    </div>

    <!-- 万年历（放在最底部） -->
    <PerpetualCalendar />

    <!-- 快速添加待办弹窗 -->

    <Modal v-model="showQuickTodo" title="添加待办">
      <div class="flex flex-col gap-3">
        <input
          ref="quickTodoInputRef"
          v-model="quickTodoText"
          class="input input-bordered"
          placeholder="输入待办内容..."
          @keyup.enter="submitQuickTodo"
        />
        <div class="flex items-center gap-2">
          <select v-model="quickTodoPriority" class="select select-bordered select-sm">
            <option value="low">低优先级</option>
            <option value="medium">中优先级</option>
            <option value="high">高优先级</option>
          </select>
          <select v-model="quickTodoProjectId" class="select select-bordered select-sm flex-1">
            <option value="">无项目</option>
            <option v-for="p in projectStore.projects" :key="p.id" :value="p.id">{{ p.name }}</option>
          </select>
        </div>
      </div>
      <template #footer>
        <button class="btn btn-ghost btn-sm" @click="showQuickTodo = false">取消</button>
        <button class="btn btn-primary btn-sm" @click="submitQuickTodo" :disabled="!quickTodoText.trim()">添加</button>
      </template>
    </Modal>
  </div>
</template>

<script setup lang="ts">
defineOptions({ name: 'Dashboard' })
import { ref, computed, onMounted, onUnmounted, nextTick } from 'vue';
import { useRouter } from 'vue-router';
import { invoke } from '@tauri-apps/api/core';
import { useTodoStore } from '../../stores/todoStore';
import { useProjectStore } from '../../stores/projectStore';
import { useToast } from '../../composables/useToast';
import SvgIcon from '../../components/ui/SvgIcon.vue';
import Modal from '../../components/ui/Modal.vue';
import PerpetualCalendar from './components/PerpetualCalendar.vue';
import type { Todo, Project } from '../../types';

const router = useRouter();
const todoStore = useTodoStore();
const projectStore = useProjectStore();

// 时间显示
const currentTime = ref('');
const currentDate = ref('');
let timeInterval: number | null = null;

const updateTime = () => {
  const now = new Date();
  currentTime.value = now.toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' });
  currentDate.value = now.toLocaleDateString('zh-CN', { weekday: 'long', month: 'long', day: 'numeric' });
};

const greeting = computed(() => {
  const hour = new Date().getHours();
  if (hour < 6) {return '夜深了，注意休息';}
  if (hour < 9) {return '早上好，新的一天开始了';}
  if (hour < 12) {return '上午好';}
  if (hour < 14) {return '中午好';}
  if (hour < 18) {return '下午好';}
  if (hour < 22) {return '晚上好';}
  return '夜深了，注意休息';
});

// 导航
const navigateTo = (path: string) => {
  router.push(path);
};

// ===== 数据加载 =====

// 待办统计
const todoStats = computed(() => {
  const todos = todoStore.todos || [];
  const active = todos.filter(t => !t.completed).length;
  const completed = todos.filter(t => t.completed).length;
  const total = todos.length;
  return {
    active,
    completed,
    total,
    progressPercent: total > 0 ? Math.round((completed / total) * 100) : 0
  };
});

// 最近待办（前10条未完成的）
const recentTodos = computed(() => {
  return todoStore.todos
    .filter(t => !t.completed)
    .sort((a, b) => {
      // 按优先级和创建时间排序
      const priorityOrder = { high: 0, medium: 1, low: 2 };
      const pa = priorityOrder[a.priority as keyof typeof priorityOrder] ?? 2;
      const pb = priorityOrder[b.priority as keyof typeof priorityOrder] ?? 2;
      if (pa !== pb) {return pa - pb;}
      return new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime();
    })
    .slice(0, 10);
});

// 服务器统计：只展示已配置数量。当前无连通性健康检查，不再虚构在线/离线数字
const serverStats = ref({ total: 0, online: 0, offline: 0 });
const loadServerStats = async () => {
  try {
    const servers = await invoke<{ id: string; name: string; host: string }[]>('get_all_servers');
    serverStats.value.total = servers.length;
    serverStats.value.online = 0;
    serverStats.value.offline = 0;
  } catch (e) {
    console.error('加载服务器统计失败:', e);
    toast.error('加载服务器统计失败');
  }
};

// 项目统计
const projectStats = computed(() => {
  const projects = projectStore.projects || [];
  const active = projects.filter(p => !p.archived).length;
  const topProjects = projects
    .filter(p => p.stats && p.stats.progress > 0)
    .sort((a, b) => (b.stats?.progress ?? 0) - (a.stats?.progress ?? 0))
    .slice(0, 3)
    .map(p => ({
      id: p.id,
      name: p.name,
      color: p.color,
      progress: p.stats?.progress ?? 0
    }));
  return { active, total: projects.length, topProjects };
});

// 告警统计
const alertStats = ref({ total: 0, today: 0 });
const loadAlertStats = async () => {
  try {
    const alerts = await invoke<{ id: string; alertType: string; sentAt: string }[]>('get_alert_history');
    const today = new Date().toDateString();
    alertStats.value.total = alerts.length;
    alertStats.value.today = alerts.filter(a => new Date(a.sentAt).toDateString() === today).length;
  } catch (e) {
    console.error('加载告警统计失败:', e);
    toast.error('加载告警统计失败');
  }
};

// 最近部署（deploy_history 已废弃，改读 deploy_logs）
const recentDeployments = ref<{ id: string; status: string; configName?: string; createdAt: string }[]>([]);
const loadRecentDeployments = async () => {
  try {
    const logs = await invoke<{ id: string; status: string; configId: string; configName?: string; createdAt: string }[]>('get_all_deploy_logs', { limit: 10 });
    recentDeployments.value = logs.map(h => ({
      id: h.id,
      status: h.status,
      configName: h.configName || '未知配置',
      createdAt: h.createdAt
    }));
  } catch (e) {
    console.error('加载部署历史失败:', e);
    toast.error('加载部署历史失败');
  }
};

// ===== 快速添加待办 =====
const showQuickTodo = ref(false);
const quickTodoText = ref('');
const quickTodoPriority = ref<'low' | 'medium' | 'high'>('medium');
const quickTodoProjectId = ref('');
const quickTodoInputRef = ref<HTMLInputElement | null>(null);
const toast = useToast();

function openQuickTodo() {
  showQuickTodo.value = true;
  nextTick(() => quickTodoInputRef.value?.focus());
}

const submitQuickTodo = async () => {
  if (!quickTodoText.value.trim()) {return;}
  try {
    await todoStore.addTodo({
      text: quickTodoText.value.trim(),
      priority: quickTodoPriority.value,
      projectId: quickTodoProjectId.value || undefined,
      completed: false,
      createdAt: new Date().toISOString()
    });
    quickTodoText.value = '';
    quickTodoPriority.value = 'medium';
    quickTodoProjectId.value = '';
    showQuickTodo.value = false;
  } catch (e) {
    console.error('添加待办失败:', e);
  }
};

// ===== 格式化函数 =====
const formatDueDate = (date: string) => {
  const d = new Date(date);
  const today = new Date();
  const diffDays = Math.ceil((d.getTime() - today.getTime()) / (1000 * 60 * 60 * 24));
  if (diffDays === 0) {return '今天';}
  if (diffDays === 1) {return '明天';}
  if (diffDays < 0) {return '已过期';}
  return `${diffDays}天后`;
};

const formatTime = (dateStr: string) => {
  const d = new Date(dateStr);
  const now = new Date();
  const diffMs = now.getTime() - d.getTime();
  const diffMinutes = Math.floor(diffMs / (1000 * 60));
  const diffHours = Math.floor(diffMs / (1000 * 60 * 60));
  const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));
  
  if (diffMinutes < 1) {return '刚刚';}
  if (diffMinutes < 60) {return `${diffMinutes}分钟前`;}
  if (diffHours < 24) {return `${diffHours}小时前`;}
  if (diffDays < 7) {return `${diffDays}天前`;}
  return d.toLocaleDateString('zh-CN');
};

// ===== 生命周期 =====
onMounted(async () => {
  updateTime();
  timeInterval = window.setInterval(updateTime, 1000);

  // 并行加载数据（各函数内部已捕获错误，互不影响）
  await Promise.all([
    todoStore.loadTodos(),
    projectStore.loadProjects(),
    loadServerStats(),
    loadAlertStats(),
    loadRecentDeployments(),
  ]);
});

onUnmounted(() => {
  if (timeInterval) {
    clearInterval(timeInterval);
  }
});
</script>
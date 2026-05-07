<template>
  <div class="project-card" :style="{ borderColor: project.color }" @click="$emit('select', project)">
    <!-- 左侧：颜色+核心信息 -->
    <div class="card-left">
      <div class="color-dot" :style="{ backgroundColor: project.color }"></div>
      <div class="card-main">
        <!-- 标题行 -->
        <div class="title-row">
          <h3 class="project-name">{{ project.name }}</h3>
          <span v-if="project.category" class="category-badge" :class="'category-' + project.category">{{ categoryLabel(project.category) }}</span>
          <span v-if="project.archived" class="archived-badge">已归档</span>
        </div>
        <!-- 描述 -->
        <p v-if="project.description" class="project-description">{{ project.description }}</p>
        <!-- 元信息行 -->
        <div class="meta-row">
          <span class="meta-item" v-if="project.createdAt">
            <span class="meta-icon">📅 创建于 {{ formatDate(project.createdAt) }}</span>
          </span>
          <span class="meta-item" v-if="project.updatedAt">
            <span class="meta-icon">✏️ 更新于 {{ formatDate(project.updatedAt) }}</span>
          </span>
        </div>
        <!-- Git 仓库 -->
        <div class="git-repos" v-if="hasGitRepos">
          <div v-if="project.repoPath" class="git-repo">
            <span class="git-icon">📂</span>
            <span class="git-url">{{ project.repoPath.split('/').pop() }}</span>
            <span v-if="project.branch" class="branch-badge">{{ project.branch }}</span>
          </div>
          <div v-if="project.repoPath2" class="git-repo">
            <span class="git-icon">📂</span>
            <span class="git-url">{{ project.repoPath2.split('/').pop() }}</span>
            <span v-if="project.branch2" class="branch-badge">{{ project.branch2 }}</span>
          </div>
          <div v-if="project.gitUrl1" class="git-repo">
            <span class="git-icon">🌐</span>
            <span class="git-url">{{ project.gitUrl1 }}</span>
          </div>
          <div v-if="project.gitUrl2" class="git-repo">
            <span class="git-icon">🌐</span>
            <span class="git-url">{{ project.gitUrl2 }}</span>
          </div>
        </div>
      </div>
    </div>
    <!-- 右侧：统计+操作 -->
    <div class="card-right">
      <div class="stats-section">
        <div class="stats-numbers">
          <div class="stat"><span class="stat-val">{{ stats?.total || 0 }}</span><span class="stat-lbl">总任务</span></div>
          <div class="stat completed"><span class="stat-val">{{ stats?.completed || 0 }}</span><span class="stat-lbl">已完成</span></div>
          <div class="stat active"><span class="stat-val">{{ (stats?.total || 0) - (stats?.completed || 0) }}</span><span class="stat-lbl">进行中</span></div>
        </div>
        <div class="progress-section">
          <div class="progress-bar">
            <div class="progress-fill" :style="{ width: (stats?.progress || 0) + '%', backgroundColor: project.color }"></div>
          </div>
          <span class="progress-pct">{{ stats?.progress || 0 }}%</span>
        </div>
      </div>
      <div class="card-actions">
        <button class="action-btn" @click.stop="$emit('toggle-archive', project)" :title="project.archived ? '取消归档' : '归档'">
          {{ project.archived ? '↩️' : '📁' }}
        </button>
        <button class="action-btn primary" @click.stop="$emit('edit', project)" title="编辑">✏️</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';

const props = defineProps({
  project: { type: Object, required: true },
  stats: { type: Object, default: () => ({ total: 0, completed: 0, progress: 0 }) },
});

defineEmits(['select', 'edit', 'toggle-archive']);

const categoryMap: Record<string, string> = {
  'frontend': '前端',
  'backend': '后端',
  'infrastructure': '基础设施',
  'other': '其他',
};

const categoryLabel = (cat: string) => categoryMap[cat] || cat;

const hasGitRepos = computed(() =>
  props.project.repoPath || props.project.repoPath2 || props.project.gitUrl1 || props.project.gitUrl2
);

const formatDate = (dateStr: string) => {
  if (!dateStr) return '';
  return new Date(dateStr).toLocaleDateString('zh-CN', { month: '2-digit', day: '2-digit' });
};
</script>

<style scoped>
.project-card {
  display: flex;
  align-items: stretch;
  gap: 16px;
  padding: 16px 20px;
  border: 1.5px solid var(--border-color);
  border-left: 4px solid var(--border-color);
  border-radius: 12px;
  background: var(--card-bg);
  cursor: pointer;
  transition: all 0.15s ease;
  box-shadow: var(--card-shadow);
}
.project-card:hover {
  border-color: var(--primary-color);
  box-shadow: 0 2px 12px rgba(0,0,0,0.1);
  transform: translateY(-1px);
}

.card-left { display: flex; gap: 14px; flex: 1; min-width: 0; }
.color-dot { width: 12px; height: 12px; border-radius: 50%; flex-shrink: 0; margin-top: 6px; }
.card-main { flex: 1; min-width: 0; }

.title-row { display: flex; align-items: center; gap: 10px; flex-wrap: wrap; margin-bottom: 6px; }
.project-name { margin: 0; color: var(--main-text); font-size: 18px; font-weight: 700; }
.category-badge { display: inline-block; padding: 2px 10px; border-radius: 12px; font-size: 12px; font-weight: 500; background: var(--primary-light); color: var(--primary-color); }
.archived-badge { display: inline-block; padding: 2px 10px; border-radius: 12px; font-size: 11px; font-weight: 500; background: var(--warning-light, #fef3c7); color: var(--warning-color, #d97706); }

.project-description { margin: 0 0 8px 0; color: var(--main-text-secondary); font-size: 13px; line-height: 1.5; display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical; overflow: hidden; }

.meta-row { display: flex; gap: 16px; margin-bottom: 8px; flex-wrap: wrap; }
.meta-item { font-size: 12px; color: var(--main-text-secondary); display: flex; align-items: center; gap: 4px; }
.meta-icon { font-size: 13px; }

.git-repos { display: flex; flex-direction: column; gap: 4px; }
.git-repo { display: flex; align-items: center; gap: 6px; font-size: 12px; color: var(--main-text-secondary); }
.git-icon { flex-shrink: 0; font-size: 14px; }
.git-url { flex: 1; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; font-family: 'SF Mono', monospace; font-size: 11px; }
.branch-badge { padding: 1px 8px; background: var(--primary-light); color: var(--primary-color); border-radius: 10px; font-size: 11px; font-weight: 600; white-space: nowrap; }

/* 右侧统计+操作 */
.card-right { display: flex; flex-direction: column; align-items: flex-end; justify-content: space-between; flex-shrink: 0; gap: 12px; min-width: 200px; }

.stats-section { display: flex; flex-direction: column; gap: 8px; width: 100%; }
.stats-numbers { display: flex; gap: 16px; }
.stat { display: flex; flex-direction: column; align-items: center; }
.stat-val { font-size: 18px; font-weight: 700; color: var(--main-text); }
.stat-lbl { font-size: 11px; color: var(--main-text-secondary); }
.stat.completed .stat-val { color: #10b981; }
.stat.active .stat-val { color: #f59e0b; }

.progress-section { display: flex; align-items: center; gap: 8px; }
.progress-bar { flex: 1; height: 8px; background: var(--input-bg); border-radius: 4px; overflow: hidden; min-width: 100px; }
.progress-fill { height: 100%; transition: width 0.3s ease; border-radius: 4px; }
.progress-pct { font-size: 13px; font-weight: 600; color: var(--main-text); white-space: nowrap; min-width: 36px; text-align: right; }

.card-actions { display: flex; gap: 6px; }
.action-btn { padding: 6px 10px; border: 1px solid var(--border-color); border-radius: 6px; background: var(--card-bg); color: var(--main-text); cursor: pointer; font-size: 14px; transition: all 0.15s; }
.action-btn:hover { border-color: var(--primary-color); color: var(--primary-color); }
.action-btn.primary:hover { background: var(--primary-color); color: white; border-color: var(--primary-color); }
</style>

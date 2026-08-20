<template>
  <div class="flex-1 overflow-y-auto bg-base-200">
    <div class="max-w-[760px] mx-auto px-6 py-8">
      <!-- Header -->
      <div class="flex items-center justify-between mb-6">
        <div>
          <h3 class="m-0 text-xl font-bold text-base-content flex items-center gap-2">
            <SvgIcon name="rocket" :size="20" /> 新建部署配置
          </h3>
          <p class="m-0 mt-1 text-sm text-base-content/60">按步骤快速创建；多环境、健康检查等高级能力可在创建后继续配置</p>
        </div>
        <button class="btn btn-ghost btn-sm" @click="emit('cancel')">取消</button>
      </div>

      <!-- Steps indicator -->
      <div class="flex items-center gap-0 mb-6 bg-base-100 border border-base-content/10 rounded-xl p-4">
        <template v-for="(s, i) in steps" :key="s.key">
          <div class="flex items-center gap-2 cursor-pointer select-none" :class="i <= step ? 'opacity-100' : 'opacity-40 hover:opacity-70'" @click="goStep(i)">
            <span class="flex items-center justify-center w-7 h-7 rounded-full text-xs font-bold border-2 transition-all"
              :class="i < step ? 'bg-primary border-primary text-white' : i === step ? 'border-primary text-primary' : 'border-base-content/20 text-base-content/50'">
              <SvgIcon v-if="i < step" name="check" :size="14" />
              <template v-else>{{ i + 1 }}</template>
            </span>
            <span class="text-sm font-medium" :class="i === step ? 'text-primary' : ''">{{ s.title }}</span>
          </div>
          <div v-if="i < steps.length - 1" class="flex-1 h-px mx-3 transition-colors duration-300" :class="i < step ? 'bg-primary' : 'bg-base-content/10'" />
        </template>
      </div>

      <!-- Step panels -->
      <div class="bg-base-100 border border-base-content/10 rounded-xl p-6 min-h-[320px]">

        <!-- Step 1: 项目与分支 -->
        <div v-if="step === 0" class="flex flex-col gap-4">
          <div>
            <label class="block mb-1 text-xs font-medium text-base-content/60 uppercase tracking-wider">Git 仓库 <span class="text-error normal-case tracking-normal">*</span></label>
            <select v-model="draft.gitRepoId" class="select select-bordered w-full bg-base-200 text-sm" @change="onRepoChange">
              <option value="">选择 Git 仓库...</option>
              <option v-for="repo in gitRepos" :key="repo.id" :value="repo.id">{{ repo.name }} — {{ repo.path }}</option>
            </select>
          </div>
          <div class="grid grid-cols-2 gap-4">
            <div>
              <label class="block mb-1 text-xs font-medium text-base-content/60 uppercase tracking-wider">配置名称 <span class="text-error normal-case tracking-normal">*</span></label>
              <input v-model="draft.name" class="input input-bordered w-full bg-base-200 text-sm" placeholder="例如：用户中心后端" />
            </div>
            <div>
              <label class="block mb-1 text-xs font-medium text-base-content/60 uppercase tracking-wider">分组</label>
              <select v-model="draft.groupName" class="select select-bordered w-full bg-base-200 text-sm">
                <option v-for="g in groups" :key="g" :value="g">{{ g }}</option>
              </select>
            </div>
          </div>
          <div>
            <label class="block mb-1 text-xs font-medium text-base-content/60 uppercase tracking-wider">部署分支</label>
            <div class="flex gap-1.5">
              <select v-model="draft.deployBranch" class="select select-bordered w-full bg-base-200 text-sm flex-1">
                <option value="main">main</option>
                <option value="master">master</option>
                <option v-for="b in branches" :key="b" :value="b">{{ b }}</option>
              </select>
              <button class="btn btn-ghost btn-sm" :disabled="!draft.gitRepoId || loadingBranches" @click="loadBranches" title="刷新分支列表">
                <SvgIcon name="refresh" :size="14" :class="{ 'animate-spin': loadingBranches }" />
              </button>
            </div>
          </div>
          <div>
            <label class="block mb-1 text-xs font-medium text-base-content/60 uppercase tracking-wider">本地代码目录</label>
            <div class="flex gap-1.5">
              <input :value="draft.localPath" readonly class="input input-bordered w-full bg-base-200 text-sm font-mono" placeholder="用于扫描构建工具与模块（默认取 Git 仓库根目录）" />
              <button class="btn btn-ghost btn-sm whitespace-nowrap" @click="pickLocalDir" title="选择实际代码目录（如 src/xxx，不在仓库根目录时）">
                <SvgIcon name="folderOpen" :size="14" /> 选择目录
              </button>
              <button v-if="draft.localPath" class="btn btn-ghost btn-sm" @click="scanProject(draft.localPath)" title="重新扫描" :disabled="scanningProj">
                <SvgIcon name="refresh" :size="14" :class="{ 'animate-spin': scanningProj }" />
              </button>
            </div>
          </div>
          <div class="flex items-start gap-2 px-3 py-2.5 rounded-lg bg-primary/5 border border-primary/15 text-xs text-base-content/70">
            <SvgIcon name="lightbulb" :size="14" class="shrink-0 mt-0.5" />
            <span>默认按 Git 仓库根目录扫描；若代码在子目录（如 <code class="bg-base-200 px-1 rounded">src/xxx</code>），请用「选择目录」定位实际代码位置，以正确识别构建工具与多模块{{ scanningProj ? '，正在扫描...' : '' }}</span>
          </div>
        </div>

        <!-- Step 2: 构建配置 -->
        <div v-else-if="step === 1" class="flex flex-col gap-4">
          <div>
            <label class="block mb-1 text-xs font-medium text-base-content/60 uppercase tracking-wider">构建工具 <span class="text-error normal-case tracking-normal">*</span></label>
            <div class="grid grid-cols-6 gap-2">
              <div v-for="tool in buildTools" :key="tool.key"
                class="flex flex-col items-center px-2 py-3 border-2 rounded-xl cursor-pointer transition-all duration-150 relative hover:border-primary"
                :class="{ 'border-primary bg-primary/10': draft.buildTool === tool.key, 'opacity-40': !tool.available && tool.key !== 'cargo' }"
                :title="tool.available ? tool.name : `${tool.name}（未安装）`"
                @click="draft.buildTool = tool.key">
                <span class="text-2xl mb-1">{{ tool.icon }}</span>
                <span class="text-xs font-semibold text-base-content">{{ tool.name }}</span>
                <span v-if="tool.version" class="text-[10px] text-base-content/60 mt-0.5">{{ tool.version.split(' ')[0] }}</span>
              </div>
            </div>
          </div>
          <div v-if="draft.buildTool === 'maven'" class="grid grid-cols-2 gap-4">
            <div>
              <label class="block mb-1 text-xs font-medium text-base-content/60 uppercase tracking-wider">Maven Profile</label>
              <input v-model="draft.mavenProfile" class="input input-bordered w-full bg-base-200 text-sm" placeholder="prod" />
            </div>
            <div>
              <label class="block mb-1 text-xs font-medium text-base-content/60 uppercase tracking-wider">重启脚本</label>
              <input v-model="draft.restartScript" class="input input-bordered w-full bg-base-200 text-sm" placeholder="./restart.sh" />
            </div>
          </div>
          <div v-else-if="['npm', 'pnpm', 'yarn'].includes(draft.buildTool)">
            <label class="block mb-1 text-xs font-medium text-base-content/60 uppercase tracking-wider">构建脚本</label>
            <select v-model="draft.npmScript" class="select select-bordered w-full bg-base-200 text-sm">
              <option value="build">build</option>
              <option value="build:prod">build:prod</option>
              <option value="custom">自定义...</option>
            </select>
            <input v-if="draft.npmScript === 'custom'" v-model="draft.npmCustomScript" class="input input-bordered w-full bg-base-200 text-sm mt-2" placeholder="脚本名称" />
          </div>

          <!-- 多模块识别区（仅多模块项目显示） -->
          <div v-if="isMultiModule" class="border border-primary/20 rounded-xl overflow-hidden">
            <div class="flex items-center gap-2 px-4 py-3 bg-primary/5 border-b border-primary/10">
              <SvgIcon name="layers" :size="15" class="text-primary flex-shrink-0" />
              <span class="text-sm font-semibold text-base-content">部署模块</span>
              <span v-if="scanningProj" class="ml-1 loading loading-spinner loading-xs text-primary" />
              <span v-else class="ml-1 text-xs text-base-content/60">识别到 {{ modules.length }} 个模块{{ selectedModules.length ? `，已勾选 ${selectedModules.length} 个` : '' }}</span>
              <span class="ml-auto text-[10px] text-base-content/50">父 POM 统一构建，每个模块部署到独立远程目录</span>
            </div>
            <div class="p-2 max-h-56 overflow-y-auto flex flex-col">
              <label
                v-for="m in modules" :key="m.moduleName"
                class="flex items-center gap-2.5 px-3 py-2 rounded-lg cursor-pointer select-none hover:bg-base-200/60 transition-colors"
              >
                <input v-model="m.checked" type="checkbox" class="checkbox checkbox-primary checkbox-sm" />
                <span class="text-sm font-medium text-base-content">{{ m.moduleName }}</span>
                <span class="ml-auto text-xs text-base-content/40 font-mono">{{ m.modulePath }}</span>
              </label>
              <div v-if="!selectedModules.length" class="px-3 py-2 text-xs text-amber-600">
                未勾选任何模块，将不部署子模块
              </div>
            </div>
          </div>
          <div class="flex items-start gap-2 px-3 py-2.5 rounded-lg bg-primary/5 border border-primary/15 text-xs text-base-content/70">
            <SvgIcon name="lightbulb" :size="14" class="shrink-0 mt-0.5" />
            <span>构建工具路径（Maven/JDK/Node 等）会自动检测填充，创建后在「构建配置」分组中可查看和修改</span>
          </div>
        </div>

        <!-- Step 3: 部署目标 -->
        <div v-else-if="step === 2" class="flex flex-col gap-4">
          <div>
            <label class="block mb-1 text-xs font-medium text-base-content/60 uppercase tracking-wider">目标服务器 <span class="text-error normal-case tracking-normal">*</span></label>
            <GroupedServerSelector :servers="servers" :groups="serverGroups" v-model="selectedServerIds" mode="multi" />
            <span class="block text-xs text-base-content/60 mt-1.5">已选 {{ selectedServerIds.length }} 台，部署时按顺序逐台上传</span>
          </div>
          <div>
            <label class="block mb-1 text-xs font-medium text-base-content/60 uppercase tracking-wider">部署路径 <span class="text-error normal-case tracking-normal">*</span></label>
            <input v-model="draft.deployPath" class="input input-bordered w-full bg-base-200 text-sm font-mono" :placeholder="suggestedDeployPath" />
            <span class="block text-xs text-base-content/60 mt-1">服务器上的目标目录{{ draft.buildTool === 'maven' ? '，如 /opt/apphome' : '，如 /home/nginxWebUI/ui' }}</span>
          </div>
          <div class="flex items-start gap-2 px-3 py-2.5 rounded-lg bg-primary/5 border border-primary/15 text-xs text-base-content/70">
            <SvgIcon name="lightbulb" :size="14" class="shrink-0 mt-0.5" />
            <span>增量上传默认开启：只传输变更文件；健康检查与失败自动回滚可在创建后于「部署保障」分组配置</span>
          </div>
        </div>

        <!-- Step 4: 确认创建 -->
        <div v-else class="flex flex-col gap-4">
          <div class="text-sm font-semibold text-base-content mb-1">确认配置摘要</div>
          <div class="border border-base-content/10 rounded-xl overflow-hidden text-sm">
            <div class="grid grid-cols-[110px_1fr]">
              <template v-for="row in summaryRows" :key="row.label">
                <div class="px-4 py-2.5 bg-base-200 text-base-content/60 border-b border-base-content/5">{{ row.label }}</div>
                <div class="px-4 py-2.5 text-base-content border-b border-base-content/5 break-all">{{ row.value || '—' }}</div>
              </template>
            </div>
          </div>
          <div v-if="missingKeys.length" class="px-3 py-2.5 rounded-lg bg-amber-500/10 border border-amber-500/20 text-xs text-amber-600">
            <span class="font-semibold">以下必填项缺失：</span>{{ missingKeys.join('、') }}
          </div>
        </div>
      </div>

      <!-- Footer buttons -->
      <div class="flex items-center justify-between mt-5">
        <button class="btn btn-ghost" :disabled="step === 0" @click="step--">
          <SvgIcon name="chevronLeft" :size="14" /> 上一步
        </button>
        <div class="flex gap-2">
          <button v-if="step < steps.length - 1" class="btn btn-primary" :disabled="!stepValid" @click="step++">
            下一步 <SvgIcon name="chevronRight" :size="14" />
          </button>
          <button v-else class="btn btn-primary" :disabled="!allValid || creating" @click="finish">
            <span v-if="creating" class="loading loading-spinner loading-xs" />
            <SvgIcon v-else name="check" :size="14" />
            {{ creating ? '创建中...' : '创建配置' }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, watch } from 'vue'
import SvgIcon from '@/components/ui/SvgIcon.vue'
import GroupedServerSelector from '../server/GroupedServerSelector.vue'
import { getTauriAPI } from '../../utils/tauri-api'
import type { Server } from '../../types'

interface GitRepoEntry { id: string; name: string; path?: string; branch?: string }
interface ServerGroupEntry { id: string; name: string; color: string; parentId: string | null }
interface BuildToolOption { key: string; name: string; icon: string; version?: string; available: boolean }
interface ModuleItem { moduleName: string; modulePath: string; checked: boolean }

const props = defineProps<{
  gitRepos: GitRepoEntry[]
  groups: string[]
  servers: Server[]
  serverGroups: ServerGroupEntry[]
  buildTools: BuildToolOption[]
}>()

const emit = defineEmits<{
  complete: [payload: Record<string, unknown>]
  cancel: []
}>()

const steps = [
  { key: 'project', title: '项目与分支' },
  { key: 'build', title: '构建配置' },
  { key: 'deploy', title: '部署目标' },
  { key: 'confirm', title: '确认创建' },
]

const step = ref(0)
const creating = ref(false)
const branches = ref<string[]>([])
const loadingBranches = ref(false)
const selectedServerIds = ref<string[]>([])

const draft = reactive({
  name: '',
  gitRepoId: '',
  groupName: '未分组',
  deployBranch: 'main',
  buildTool: '',
  mavenProfile: 'prod',
  npmScript: 'build',
  npmCustomScript: '',
  restartScript: './restart.sh',
  deployPath: '',
  // 实际代码目录（可能不在 git 仓库根目录，如 src/xxx 子模块），用于扫描识别构建工具与模块
  localPath: '',
})

// 多模块：扫描识别出 moduleNames 后生成的勾选列表；无多模块则为空数组
const modules = ref<ModuleItem[]>([])

// 默认选中第一个可用构建工具
watch(() => props.buildTools, (tools) => {
  if (!draft.buildTool) {
    const first = tools.find(t => t.available)
    if (first) { draft.buildTool = first.key }
  }
}, { immediate: true })

// 选中仓库后自动填充名称/分支并加载分支列表
watch(() => draft.gitRepoId, (id) => { if (id) { onRepoChange() } })

async function onRepoChange() {
  const repo = props.gitRepos.find(r => r.id === draft.gitRepoId)
  if (!repo) {return;}
  if (!draft.name) {draft.name = repo.name;}
  if (repo.branch) {draft.deployBranch = repo.branch;}
  if (!draft.localPath) {draft.localPath = repo.path || '';}
  loadBranches();
  scanProject(draft.localPath);
}

// 手动选择实际代码目录（模块可能不在 git 仓库根目录，如 src/xxx），并重新扫描
async function pickLocalDir() {
  const { getTauriAPI } = await import('../../utils/tauri-api')
  try {
    const result = await getTauriAPI().showOpenDialogForDirs()
    const dir = result?.filePaths?.[0]
    if (dir) {
      draft.localPath = dir
      scanProject(dir)
      // 未命名时用选中目录名兜底
      if (!draft.name) {
        const name = dir.split(/[\\/]/).filter(Boolean).pop() || ''
        draft.name = name
      }
    }
  } catch { /* 静默 */ }
}

async function loadBranches() {
  const repo = props.gitRepos.find(r => r.id === draft.gitRepoId)
  if (!repo?.path) {return;}
  loadingBranches.value = true;
  try {
    const result = await getTauriAPI().getGitBranches(repo.path);
    branches.value = (result?.branches || result || []).map((b: unknown) => typeof b === 'string' ? b : (b as { name: string }).name);
  } catch { branches.value = []; }
  finally { loadingBranches.value = false; }
}

// 扫描项目自动识别构建工具与推荐部署路径；多模块项目同时识别子模块
const scanned = ref<Record<string, unknown>>({})
const scanningProj = ref(false)
async function scanProject(path: string) {
  if (!path) {return;}
  scanningProj.value = true;
  try {
    const result = await getTauriAPI().scanProject(path);
    if (result && typeof result === 'object') {
      const r = result as Record<string, unknown>;
      scanned.value = r;
      if (r.buildTool) {draft.buildTool = r.buildTool as string;}
      if (r.currentBranch && !branches.value.length) {draft.deployBranch = r.currentBranch as string;}
      if (r.recommendedScript) {draft.npmScript = r.recommendedScript as string;}
      // 多模块识别：填充勾选列表（默认全选，父 POM 统一构建）
      if (r.isMultiModule && Array.isArray(r.moduleNames)) {
        const names = (r.moduleNames as string[]).filter(Boolean);
        if (names.length) {
          modules.value = names.map(n => ({ moduleName: n, modulePath: n, checked: true }));
          scanned.value.isMultiModule = true;
        }
      }
    }
  } finally { scanningProj.value = false; }
}

const suggestedDeployPath = computed(() =>
  (scanned.value.suggestedDeployPath as string) || (draft.buildTool === 'maven' ? '/opt/apphome' : '/home/nginxWebUI/ui'))

// 扫描出推荐部署路径后自动填入（仅当用户未修改时）
watch(suggestedDeployPath, (p) => { if (p && !draft.deployPath) {draft.deployPath = p;} }, { immediate: true })

const stepValid = computed(() => {
  if (step.value === 0) {return !!draft.gitRepoId && !!draft.name.trim();}
  if (step.value === 1) {return !!draft.buildTool;}
  if (step.value === 2) {return selectedServerIds.value.length > 0 && !!draft.deployPath.trim();}
  return true;
})

const missingKeys = computed(() => {
  const missing: string[] = [];
  if (!draft.gitRepoId) {missing.push('Git 仓库');}
  if (!draft.name.trim()) {missing.push('配置名称');}
  if (!draft.buildTool) {missing.push('构建工具');}
  if (!selectedServerIds.value.length) {missing.push('目标服务器');}
  if (!draft.deployPath.trim()) {missing.push('部署路径');}
  return missing;
})

const allValid = computed(() => missingKeys.value.length === 0)

const repoName = computed(() => props.gitRepos.find(r => r.id === draft.gitRepoId)?.name || '')
const serverNames = computed(() =>
  selectedServerIds.value.map(id => props.servers.find(s => s.id === id)?.name).filter(Boolean).join('、'))
// 已勾选要部署的模块
const selectedModules = computed(() => modules.value.filter(m => m.checked))
const isMultiModule = computed(() => modules.value.length > 0)
const moduleSummary = computed(() => selectedModules.value.map(m => m.moduleName).join('、'))

const summaryRows = computed(() => {
  const rows: { label: string; value: string }[] = [
    { label: '配置名称', value: draft.name },
    { label: 'Git 仓库', value: repoName.value },
    { label: '部署分支', value: draft.deployBranch },
    { label: '分组', value: draft.groupName },
    { label: '构建工具', value: props.buildTools.find(t => t.key === draft.buildTool)?.name || draft.buildTool },
    { label: '目标服务器', value: serverNames.value },
    { label: '部署路径', value: draft.deployPath },
  ];
  if (isMultiModule.value) {
    rows.push({ label: '部署模块', value: moduleSummary.value || '—' });
  }
  return rows;
})

function goStep(i: number) {
  // 只允许回退或前进一步（前进需当前步校验通过）
  if (i <= step.value || (i === step.value + 1 && stepValid.value)) {step.value = i;}
}

async function finish() {
  if (!allValid.value || creating.value) {return;}
  creating.value = true;
  try {
    const serverEntries = selectedServerIds.value.map(id => {
      const s = props.servers.find(srv => srv.id === id)
      return { serverId: id, label: s?.name || '', deployDir: '' }
    })
    emit('complete', {
      ...draft,
      servers: serverEntries,
      modules: selectedModules.value.map(m => ({
        moduleName: m.moduleName, modulePath: m.modulePath, artifactName: '',
      })),
    })
  } finally {
    creating.value = false;
  }
}
</script>

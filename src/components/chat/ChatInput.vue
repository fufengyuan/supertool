<template>
  <!-- 输入区域 -->
  <div class="border-t border-base-content/10 px-4 py-3 bg-base-100">
    <!-- Hermes 未安装提示 -->
    <div v-if="!hermesAvailable" class="flex items-center justify-center gap-2 py-2">
      <SvgIcon name="warning" size="14" class="text-warning" />
      <span class="text-xs text-base-content/60">Hermes 未安装或不可用</span>
      <button class="btn btn-ghost btn-xs" @click="$emit('checkHermes')">检测</button>
    </div>

    <!-- 正常输入 -->
    <div v-else class="space-y-2">
      <!-- 模型选择、工具集和引用消息显示 -->
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-2">
          <!-- 附件按钮 -->
          <div class="relative">
            <button
              class="btn btn-ghost btn-xs btn-square"
              @click="showAttachMenu = !showAttachMenu"
              title="添加文件/文件夹/Git仓库路径"
              :disabled="isStreaming"
            >
              <SvgIcon name="paperclip" size="14" />
            </button>
            <!-- 下拉菜单 -->
            <div 
              v-if="showAttachMenu" 
              class="absolute left-0 bottom-full mb-1 bg-base-100 border border-base-content/20 rounded-lg shadow-lg z-50 min-w-[200px]"
            >
              <!-- 常用文件夹 -->
              <div v-if="favoriteFolders.length > 0" class="border-b border-base-content/10">
                <div class="px-3 py-1.5 text-xs text-base-content/50 font-medium flex items-center justify-between">
                  <span>常用文件夹</span>
                </div>
                <div 
                  v-for="folder in favoriteFolders" 
                  :key="folder"
                  class="flex items-center gap-1 px-2 py-1 text-xs hover:bg-base-200 group"
                >
                  <span class="truncate flex-1 text-base-content/70" :title="folder">{{ folder.split('/').pop() || folder }}</span>
                  <button 
                    class="btn btn-ghost btn-xs btn-square opacity-0 group-hover:opacity-100" 
                    @click.stop="selectFromFavorite(folder, 'file')"
                    title="从此文件夹选择文件"
                  >
                    <SvgIcon name="file" size="10" class="text-base-content/60" />
                  </button>
                  <button 
                    class="btn btn-ghost btn-xs btn-square opacity-0 group-hover:opacity-100" 
                    @click.stop="selectFromFavorite(folder, 'folder')"
                    title="从此文件夹选择子文件夹"
                  >
                    <SvgIcon name="folder" size="10" class="text-base-content/60" />
                  </button>
                  <button 
                    class="btn btn-ghost btn-xs btn-square opacity-0 group-hover:opacity-100 hover:text-error" 
                    @click.stop="$emit('removeFavoriteFolder', folder)"
                    title="移除"
                  >
                    <SvgIcon name="close" size="10" />
                  </button>
                </div>
              </div>
              <!-- 文件/文件夹选择 -->
              <button class="flex items-center gap-2 w-full px-3 py-2 text-xs hover:bg-base-200" @click="selectFile()">
                <SvgIcon name="file" size="14" class="text-base-content/60" />
                <span>选择文件</span>
              </button>
              <button class="flex items-center gap-2 w-full px-3 py-2 text-xs hover:bg-base-200" @click="selectFolder()">
                <SvgIcon name="folder" size="14" class="text-base-content/60" />
                <span>选择文件夹</span>
              </button>
              <!-- Git 仓库列表 -->
              <div v-if="gitRepos.length > 0" class="border-t border-base-content/10">
                <div class="px-3 py-1.5 text-xs text-base-content/50 font-medium">Git 仓库</div>
                <button 
                  v-for="repo in gitRepos" 
                  :key="repo.id" 
                  class="flex items-center gap-2 w-full px-3 py-1.5 text-xs hover:bg-base-200 rounded-b-lg"
                  @click="selectGitRepo(repo)"
                >
                  <SvgIcon name="github" size="12" class="text-base-content/60" />
                  <span class="truncate">{{ repo.name }}</span>
                </button>
              </div>
            </div>
          </div>
          <!-- 模型选择 -->
          <div class="relative flex items-center gap-1.5 model-dropdown-container">
            <!-- 模型选择按钮 -->
            <button
              class="select select-bordered select-xs max-w-[240px] flex items-center justify-between"
              :disabled="isStreaming"
              @click="showModelDropdown = !showModelDropdown"
            >
              <span class="truncate">
                {{ selectedModel ? parseModelName(selectedModel).name : (defaultModel ? parseModelName(defaultModel).name || defaultModel : '默认模型') }}
              </span>
              <SvgIcon name="chevronDown" size="12" class="ml-1 shrink-0" />
            </button>
            <!-- 下拉菜单 -->
            <div
              v-if="showModelDropdown"
              class="absolute left-0 bottom-full mb-1 bg-base-100 border border-base-content/20 rounded-lg shadow-lg z-50 w-[320px] max-h-[400px] overflow-hidden"
              @click.stop
            >
              <!-- 搜索框 -->
              <div class="p-2 border-b border-base-content/10">
                <input
                  ref="modelSearchRef"
                  v-model="modelSearchQuery"
                  type="text"
                  class="input input-bordered input-xs w-full"
                  placeholder="搜索模型..."
                  @keydown.esc="showModelDropdown = false"
                />
              </div>
              <!-- 模型列表 -->
              <div class="overflow-y-auto max-h-[340px]">
                <!-- 默认模型 -->
                <button
                  class="flex items-center gap-2 w-full px-3 py-2 text-xs hover:bg-base-200"
                  :class="{ 'bg-primary/10': !selectedModel }"
                  @click="setModel(''); showModelDropdown = false"
                >
                  <span class="text-base-content/60">默认</span>
                  <span class="truncate">{{ defaultModel ? parseModelName(defaultModel).name || defaultModel : '系统默认' }}</span>
                </button>
                <!-- 分组（可折叠） -->
                <template v-for="group in filteredModelGroups" :key="group.provider">
                  <div
                    class="flex items-center gap-2 px-3 py-1.5 text-xs font-medium text-base-content/60 cursor-pointer hover:bg-base-200"
                    @click="toggleModelGroup(group.provider)"
                  >
                    <SvgIcon :name="expandedModelGroups[group.provider] ? 'chevronDown' : 'chevronRight'" size="10" />
                    <span>{{ group.label }}</span>
                    <span class="text-base-content/40">({{ group.models.length }})</span>
                  </div>
                  <template v-if="expandedModelGroups[group.provider]">
                    <div v-for="m in group.models.slice(0, 20)" :key="m" class="group/model relative">
                      <button
                        class="flex items-center gap-2 w-full px-3 pl-6 py-1.5 text-xs hover:bg-base-200"
                        :class="{ 'bg-primary/10': selectedModel === m }"
                        @click="setModel(m); showModelDropdown = false"
                      >
                        <span class="truncate flex-1">{{ parseModelName(m).name }}</span>
                      </button>
                      <button
                        v-if="!isDefaultModel(m)"
                        class="absolute right-1 top-1/2 -translate-y-1/2 opacity-0 group-hover/model:opacity-100 btn btn-ghost btn-xs btn-square"
                        @click.stop="deleteModel(m)"
                        title="删除模型"
                      >
                        <SvgIcon name="close" size="10" class="text-error/60 hover:text-error" />
                      </button>
                    </div>
                    <button
                      v-if="group.models.length > 20"
                      class="flex items-center gap-2 w-full px-3 pl-6 py-1.5 text-xs text-base-content/60 hover:bg-base-200"
                      @click="toggleModelGroupFull(group.provider)"
                    >
                      <span>{{ expandedModelGroupsFull[group.provider] ? '收起' : `展开全部 ${group.models.length} 个` }}</span>
                    </button>
                    <template v-if="expandedModelGroupsFull[group.provider]">
                      <div v-for="m in group.models.slice(20)" :key="m" class="group/model relative">
                        <button
                          class="flex items-center gap-2 w-full px-3 pl-6 py-1.5 text-xs hover:bg-base-200"
                          :class="{ 'bg-primary/10': selectedModel === m }"
                          @click="setModel(m); showModelDropdown = false"
                        >
                          <span class="truncate flex-1">{{ parseModelName(m).name }}</span>
                        </button>
                        <button
                          v-if="!isDefaultModel(m)"
                          class="absolute right-1 top-1/2 -translate-y-1/2 opacity-0 group-hover/model:opacity-100 btn btn-ghost btn-xs btn-square"
                          @click.stop="deleteModel(m)"
                          title="删除模型"
                        >
                          <SvgIcon name="close" size="10" class="text-error/60 hover:text-error" />
                        </button>
                      </div>
                    </template>
                  </template>
                </template>
              </div>
            </div>
            <!-- 供应商标签 -->
            <span v-if="currentProviderLabel"
              class="badge badge-ghost badge-xs text-[10px] text-base-content/50 shrink-0 max-w-[80px] truncate"
              :title="currentProviderLabel"
            >{{ currentProviderLabel }}</span>
          </div>
          <!-- 添加模型按钮 -->
          <button
            class="btn btn-ghost btn-xs btn-square"
            @click="showAddModelDialog = true"
            :disabled="isStreaming"
            title="添加模型"
          >
            <SvgIcon name="plus" size="14" />
          </button>
        </div>
      </div>
      <!-- 已选择路径徽章 -->
      <div v-if="attachedPaths.length > 0" class="flex flex-wrap gap-1.5 px-1">
        <div
          v-for="(item, idx) in attachedPaths"
          :key="idx"
          class="group flex items-center gap-1 px-2 py-1 rounded-md text-xs border cursor-default transition-all"
          :class="item.type === 'folder'
            ? 'bg-warning/5 border-warning/20 text-warning/80'
            : 'bg-info/5 border-info/20 text-info/80'"
          :title="item.path"
        >
          <img v-if="item.previewUrl" :src="item.previewUrl" class="w-6 h-6 rounded object-cover shrink-0" />
          <SvgIcon v-else :name="item.type === 'folder' ? 'folder' : 'file'" size="12" />
          <span class="max-w-[160px] truncate">{{ item.name }}</span>
          <button
            class="ml-0.5 opacity-40 group-hover:opacity-100 hover:!opacity-100 transition-opacity rounded-full hover:bg-base-content/10 p-0.5"
            :class="item.type === 'folder' ? 'hover:text-warning' : 'hover:text-info'"
            @click.stop="removeAttachedPath(idx)"
            title="移除"
          >
            <SvgIcon name="close" size="10" />
          </button>
        </div>
      </div>
      <!-- 输入框 -->
      <div class="flex gap-2 relative">
        <!-- 斜杠命令自动补全菜单 -->
        <div
          v-if="slash.isSlashMenuVisible.value && slash.filteredSlashCommands.value.length > 0"
          class="absolute left-0 bottom-full mb-1 bg-base-100 border border-base-content/20 rounded-lg shadow-xl z-50 w-[380px] max-h-[320px] overflow-hidden"
          @click.stop
        >
          <!-- 菜单头部 -->
          <div class="px-3 py-1.5 bg-base-200/50 border-b border-base-content/10">
            <span class="text-[11px] font-medium text-base-content/50">Slash Commands</span>
          </div>
          <!-- 命令列表 -->
          <div class="overflow-y-auto max-h-[280px] py-1">
            <template v-for="(cmd, idx) in slash.filteredSlashCommands.value" :key="cmd.name">
              <!-- 类别分隔符 -->
              <div
                v-if="idx === 0 || cmd.category !== slash.filteredSlashCommands.value[idx - 1]?.category"
                class="px-3 pt-2 pb-1 text-[10px] font-semibold text-base-content/40 uppercase tracking-wider"
              >
                {{ CATEGORY_LABELS[cmd.category] || cmd.category }}
              </div>
              <!-- 命令项 -->
              <button
                class="flex items-center gap-2.5 w-full px-3 py-2 text-left hover:bg-base-200 transition-colors"
                :class="{
                  'bg-primary/10 hover:bg-primary/15': slash.slashMenuIndex.value === idx,
                }"
                @click="selectSlashCommand(cmd)"
                @mouseenter="slash.slashMenuIndex.value = idx"
              >
                <SvgIcon
                  :name="CATEGORY_ICONS[cmd.category] || 'command'"
                  :size="14"
                  :class="slash.slashMenuIndex.value === idx ? 'text-primary' : 'text-base-content/50'"
                />
                <div class="flex-1 min-w-0">
                  <div class="flex items-center gap-2">
                    <code class="text-xs font-mono font-medium" :class="cmd.local ? 'text-base-content' : 'text-primary'">
                      {{ cmd.name }}
                    </code>
                    <span v-if="cmd.local" class="badge badge-ghost badge-xs text-[9px] px-1 text-base-content/40">local</span>
                  </div>
                  <div class="text-[11px] text-base-content/50 truncate">{{ cmd.description }}</div>
                </div>
              </button>
            </template>
          </div>
        </div>
        <textarea
          ref="inputRef"
          v-model="inputText"
          class="textarea w-full resize-none text-sm transition-colors"
          :class="isStreaming ? 'textarea-warning border-warning/30 bg-warning/5' : 'textarea-bordered'"
          style="min-height: 40px;"
          :placeholder="isStreaming ? '正在处理中，输入新消息将打断当前任务...' : '输入消息...'"
          @input="autoResize"
          @keydown="handleKeydown"
          @compositionstart="isComposing = true"
          @compositionend="isComposing = false"
          @paste="handlePaste"
          autocapitalize="off"
          autocomplete="off"
        ></textarea>
        <!-- 发送按钮 -->
        <button
          class="btn self-end transition-colors"
          :class="isStreaming ? 'btn-warning' : 'btn-primary'"
          :disabled="!inputText.trim() && !isStreaming"
          @click="handleSend"
          :title="isStreaming ? '发送新消息将打断当前处理' : '发送'"
        >
          <SvgIcon v-if="isStreaming" name="send" size="14" class="animate-pulse" />
          <SvgIcon v-else name="send" size="14" />
        </button>
      </div>
    </div>
  </div>

  <!-- 添加模型对话框 -->
  <div v-if="showAddModelDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
    <div class="bg-base-100 rounded-lg p-4 w-80 shadow-xl">
      <h3 class="text-sm font-medium mb-3">添加模型</h3>
      <input
        v-model="newModelName"
        type="text"
        class="input input-bordered input-sm w-full"
        placeholder="输入模型名称"
        @keyup.enter="addModel"
      />
      <div class="flex justify-end gap-2 mt-3">
        <button class="btn btn-ghost btn-sm" @click="showAddModelDialog = false; newModelName = ''">取消</button>
        <button class="btn btn-primary btn-sm" @click="addModel" :disabled="!newModelName.trim()">添加</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted, onUnmounted, nextTick, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { getTauriAPI } from '@/utils/tauri-api';
import type { GitRepo } from '@/types';
import SvgIcon from '@/components/ui/SvgIcon.vue';
import { filesFromClipboard, processFiles, type Attachment } from '@/composables/useAttachmentProcessor';
import {
  useSlashCommands,
  CATEGORY_ICONS,
  CATEGORY_LABELS,
  SLASH_COMMANDS,
  type SlashCommand,
} from '@/composables/useSlashCommands';

// Props
const props = defineProps<{
  isStreaming: boolean;
  currentSession: { id: string; model: string } | null;
  favoriteFolders: string[];
  gitRepos: GitRepo[];
  hermesAvailable: boolean;
  onNewChat?: () => void;
  onClear?: () => void;
  usageStats?: { inputTokens: number; outputTokens: number; totalTokens: number } | null;
}>();

// Events
const emit = defineEmits<{
  send: [text: string, paths: PathItem[], model: string];
  paste: [event: ClipboardEvent];
  pasteError: [errors: string[]];
  checkHermes: [];
  removeFavoriteFolder: [folder: string];
  modelChanged: [model: string];
  pathsChanged: [paths: PathItem[]];
  commandMessage: [content: string];
}>();

// Types
interface PathItem {
  path: string;
  type: 'file' | 'folder';
  name: string;
  previewUrl?: string;
}

// Refs
const inputRef = ref<HTMLTextAreaElement | null>(null);
const inputText = ref('');
const attachedPaths = ref<PathItem[]>([]);
const showAttachMenu = ref(false);
const isComposing = ref(false);
const inputHistory = ref<string[]>([]);
const historyIndex = ref(-1);
const savedDraft = ref('');

// 斜杠命令
const slash = useSlashCommands({
  onNewChat: () => props.onNewChat?.(),
  onClear: () => props.onClear?.(),
  addAgentMessage: (content: string) => emit('commandMessage', content),
  usageStats: props.usageStats ?? undefined,
});

/** 选中非本地命令填入输入框后，暂时阻止菜单重新弹出 */
const suppressSlashMenu = ref(false);

// 监听输入文本，触发斜杠命令实时过滤
watch(inputText, (val) => {
  if (suppressSlashMenu.value) {
    slash.hideSlashMenu();
    return;
  }
  slash.updateInputText(val);
});

/** 将选中命令填入输入框（用于非本地命令） */
function setInputWithCommand(text: string) {
  suppressSlashMenu.value = true;
  inputText.value = text;
  slash.hideSlashMenu();
  nextTick(() => {
    suppressSlashMenu.value = false;
    autoResize();
    inputRef.value?.focus();
  });
}

/** 点击选择斜杠命令 */
function selectSlashCommand(cmd: SlashCommand) {
  if (cmd.local) {
    slash.executeLocal(cmd.name);
  } else {
    setInputWithCommand(cmd.name + ' ');
  }
  slash.hideSlashMenu();
}

// 模型选择
const selectedModel = ref('');
const availableModels = ref<string[]>([]);
const defaultModel = ref<string>('');
const activeProvider = ref<string>('');

// 模型选择器下拉状态
const showModelDropdown = ref(false);
const modelSearchQuery = ref('');
const modelSearchRef = ref<HTMLInputElement | null>(null);
const expandedModelGroups = reactive<Record<string, boolean>>({});
const expandedModelGroupsFull = reactive<Record<string, boolean>>({});
const showAddModelDialog = ref(false);
const newModelName = ref('');

// 切换分组展开
function toggleModelGroup(provider: string) {
  expandedModelGroups[provider] = !expandedModelGroups[provider];
  if (!expandedModelGroups[provider]) {
    expandedModelGroupsFull[provider] = false;
  }
}

// 切换分组完全展开
function toggleModelGroupFull(provider: string) {
  expandedModelGroupsFull[provider] = !expandedModelGroupsFull[provider];
}

// 供应商展示名称映射
const PROVIDER_LABELS: Record<string, string> = {
  'openai': 'OpenAI',
  'anthropic': 'Anthropic',
  'google': 'Google Gemini',
  'gemini': 'Google Gemini',
  'deepseek': 'DeepSeek',
  'meta': 'Meta',
  'mistral': 'Mistral AI',
  'cohere': 'Cohere',
  'x-ai': 'xAI (Grok)',
  'xai': 'xAI (Grok)',
  'zai': 'Z.AI / GLM',
  'z-ai': 'Z.AI / GLM',
  'stepfun': 'StepFun',
  'minimax': 'MiniMax',
  'alibaba': 'Alibaba Cloud',
  'qwen': 'Qwen',
  'nous': 'Nous Portal',
  'openrouter': 'OpenRouter',
  'copilot': 'GitHub Copilot',
  'huggingface': 'Hugging Face',
  'nvidia': 'NVIDIA NIM',
  'ai-gateway': 'Vercel AI Gateway',
  'opencode-go': 'OpenCode Go',
  'opencode-zen': 'OpenCode Zen',
  'tencent': 'Tencent',
  'moonshot': 'Moonshot / Kimi',
  'kimi': 'Kimi',
  'kimi-coding': 'Kimi',
  'xiaomi': 'Xiaomi MiMo',
  'inclusionai': 'Inclusion AI',
  'minimax-oauth': 'MiniMax (OAuth)',
  'minimax-cn': 'MiniMax (China)',
};

// 解析模型名中的供应商前缀
function parseModelName(fullName: string): { provider: string | null; name: string } {
  const slashIdx = fullName.indexOf('/');
  if (slashIdx > 0) {
    return { provider: fullName.substring(0, slashIdx), name: fullName.substring(slashIdx + 1) };
  }
  return { provider: null, name: fullName };
}

// 获取供应商显示名
function providerLabel(provider: string | null): string {
  if (!provider) {return '其他';}
  return PROVIDER_LABELS[provider] || provider;
}

// 模型分组
interface ModelGroup {
  provider: string;
  label: string;
  models: string[];
}

const modelGroups = computed<ModelGroup[]>(() => {
  const groups = new Map<string, string[]>();
  const allModels = [...availableModels.value];
  if (defaultModel.value && !allModels.includes(defaultModel.value)) {
    allModels.unshift(defaultModel.value);
  }
  for (const m of allModels) {
    const { provider } = parseModelName(m);
    const key = provider || '__other__';
    if (!groups.has(key)) {groups.set(key, []);}
    groups.get(key)!.push(m);
  }
  const result: ModelGroup[] = [];
  for (const [provider, models] of groups) {
    models.sort();
    result.push({
      provider: provider === '__other__' ? '' : provider,
      label: providerLabel(provider === '__other__' ? null : provider),
      models,
    });
  }
  result.sort((a, b) => a.label.localeCompare(b.label, 'zh-CN'));
  return result;
});

// 当前选中模型的供应商标签
const currentProviderLabel = computed(() => {
  if (!selectedModel.value && !defaultModel.value) {return '';}
  const modelName = selectedModel.value || defaultModel.value || '';
  const { provider } = parseModelName(modelName);
  return providerLabel(provider);
});

// 搜索过滤后的模型分组
const filteredModelGroups = computed<ModelGroup[]>(() => {
  const query = modelSearchQuery.value.toLowerCase();
  const groups = modelGroups.value;
  
  if (!query) {return groups;}
  
  return groups.map(group => {
    const matchingModels = group.models.filter(m =>
      m.toLowerCase().includes(query) ||
      parseModelName(m).name.toLowerCase().includes(query)
    );
    if (matchingModels.length > 0) {
      expandedModelGroups[group.provider] = true;
    }
    return { ...group, models: matchingModels };
  }).filter(group => group.models.length > 0);
});

// 加载模型列表
const loadModels = async () => {
  try {
    const result = await invoke<{ customModels: string[]; defaultModel: string | null; activeProvider: string | null; providerModels: string[] }>('agent_get_models');
    const customModels = result.customModels || [];
    const predefinedModels = result.providerModels || [];
    const mergedModels = [...new Set([...predefinedModels, ...customModels])];
    availableModels.value = mergedModels;
    defaultModel.value = result.defaultModel || '';
    activeProvider.value = result.activeProvider || '';
    if (!selectedModel.value && defaultModel.value) {
      selectedModel.value = defaultModel.value;
    }
  } catch (e) {
    console.error('Failed to load models:', e);
    availableModels.value = [];
  }
};

// 切换模型
const setModel = async (modelName: string) => {
  selectedModel.value = modelName;
  emit('modelChanged', modelName);
  try {
    await invoke('agent_set_model', { model: modelName });
    if (props.currentSession?.id) {
      await invoke('agent_clear_cache', { sessionId: props.currentSession.id });
    }
  } catch (e) {
    console.error('Failed to persist model:', e);
  }
};

// 添加模型
const addModel = async () => {
  if (!newModelName.value.trim()) {return;}
  try {
    const result = await invoke<{ success: boolean; customModels: string[] }>('agent_add_model', {
      model: newModelName.value.trim(),
    });
    if (result.success) {
      availableModels.value = result.customModels;
      newModelName.value = '';
      showAddModelDialog.value = false;
    }
  } catch (e) {
    console.error('Failed to add model:', e);
  }
};

// 删除模型
const deleteModel = async (model: string) => {
  try {
    const result = await invoke<{ success: boolean; customModels: string[] }>('agent_remove_model', {
      model,
    });
    if (result.success) {
      availableModels.value = result.customModels;
      if (selectedModel.value === model) {
        selectedModel.value = '';
        emit('modelChanged', '');
      }
    }
  } catch (e) {
    console.error('Failed to remove model:', e);
  }
};

// 检查是否为默认模型（不允许删除）
const isDefaultModel = (m: string) => m === defaultModel.value;

// 自动调整输入框高度
const autoResize = () => {
  const el = inputRef.value;
  if (!el) {return;}
  el.style.height = 'auto';
  el.style.height = `${Math.min(el.scrollHeight, 120)}px`;
};

// 选择文件
const selectFile = async (defaultPath?: string) => {
  try {
    const selected = await open({
      multiple: false,
      title: '选择文件',
      defaultPath: defaultPath || undefined,
    });
    if (selected) {
      const path = Array.isArray(selected) ? selected[0] : selected;
      const name = path.split('/').pop() || path;
      attachedPaths.value.push({ path, type: 'file', name });
      emit('pathsChanged', [...attachedPaths.value]);
      nextTick(() => autoResize());
    }
  } catch (e) {
    console.error('选择文件失败:', e);
  }
  showAttachMenu.value = false;
};

// 选择文件夹
const selectFolder = async (defaultPath?: string) => {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: '选择文件夹',
      defaultPath: defaultPath || undefined,
    });
    if (selected) {
      const path = Array.isArray(selected) ? selected[0] : selected;
      const name = path.split('/').pop() || path;
      attachedPaths.value.push({ path, type: 'folder', name });
      emit('pathsChanged', [...attachedPaths.value]);
      nextTick(() => autoResize());
    }
  } catch (e) {
    console.error('选择文件夹失败:', e);
  }
  showAttachMenu.value = false;
};

// 选择 Git 仓库
const selectGitRepo = (repo: GitRepo) => {
  const name = repo.path.split('/').pop() || repo.name || repo.path;
  attachedPaths.value.push({ path: repo.path, type: 'folder', name });
  emit('pathsChanged', [...attachedPaths.value]);
  showAttachMenu.value = false;
  nextTick(() => autoResize());
};

// 从常用文件夹打开文件选择
const selectFromFavorite = (folder: string, type: 'file' | 'folder') => {
  if (type === 'file') {
    selectFile(folder);
  } else {
    selectFolder(folder);
  }
};

// 移除已选择的路径
const removeAttachedPath = (idx: number) => {
  const item = attachedPaths.value[idx];
  if (item?.previewUrl) {URL.revokeObjectURL(item.previewUrl);}
  attachedPaths.value.splice(idx, 1);
  emit('pathsChanged', [...attachedPaths.value]);
  nextTick(() => autoResize());
};

// 发送消息
const handleSend = () => {
  if (!inputText.value.trim()) {return;}
  
  const text = inputText.value.trim();
  const paths = [...attachedPaths.value];
  const model = selectedModel.value || defaultModel.value || '';
  
  // 记录输入历史
  inputHistory.value.push(text);
  historyIndex.value = -1;
  savedDraft.value = '';
  
  // 清空输入
  inputText.value = '';
  
  // 释放图片预览的 object URL
  for (const item of attachedPaths.value) {
    if (item.previewUrl) {URL.revokeObjectURL(item.previewUrl);}
  }
  attachedPaths.value = [];
  emit('pathsChanged', []);
  
  emit('send', text, paths, model);
};

// 处理粘贴事件（处理图片和文件粘贴）
const handlePaste = async (e: ClipboardEvent) => {
  const files = filesFromClipboard(e);
  if (files.length === 0) {return;} // normal text paste, let it through
  e.preventDefault();
  const { attachments, errors } = await processFiles(files);
  if (errors.length > 0) {
    emit('pasteError', errors);
  }
  // Convert attachments to PathItem format and add to attachedPaths
  let hasTextContent = false;
  for (const att of attachments) {
    if (att.path) {
      attachedPaths.value.push({ path: att.path, type: 'file', name: att.name });
    } else if (att.dataUrl) {
      // Image: show as preview in attached paths
      attachedPaths.value.push({ path: att.name, type: 'file', name: att.name, previewUrl: att.dataUrl });
    } else if (att.text) {
      // Text file: prepend content to message
      inputText.value += (inputText.value ? '\n\n' : '') + att.text;
      hasTextContent = true;
    }
  }
  if (hasTextContent) {
    nextTick(() => autoResize());
  }
};

// 文本框快捷键处理（输入历史导航 + 斜杠命令）
const handleKeydown = (e: KeyboardEvent) => {
  // 斜杠菜单打开时，优先交给菜单处理
  if (slash.isSlashMenuVisible.value && !isComposing.value) {
    const consumed = slash.handleSlashKeydown(e, setInputWithCommand);
    if (consumed) { return; }
  }

  if (e.key === 'ArrowUp' && !e.shiftKey) {
    const el = inputRef.value;
    if (el && el.selectionStart === 0) {
      e.preventDefault();
      if (historyIndex.value === -1 && inputText.value) {
        savedDraft.value = inputText.value;
      }
      const next = historyIndex.value === -1 ? inputHistory.value.length - 1 : Math.max(0, historyIndex.value - 1);
      historyIndex.value = next;
      inputText.value = inputHistory.value[next] || '';
      nextTick(() => { el.selectionStart = el.selectionEnd = 0; });
    }
  } else if (e.key === 'ArrowDown' && !e.shiftKey) {
    const el = inputRef.value;
    if (el && el.selectionStart === el.value.length) {
      e.preventDefault();
      if (historyIndex.value === -1) {return;}
      const next = historyIndex.value + 1;
      if (next >= inputHistory.value.length) {
        historyIndex.value = -1;
        inputText.value = savedDraft.value;
      } else {
        historyIndex.value = next;
        inputText.value = inputHistory.value[next] || '';
      }
      nextTick(() => { el.selectionStart = el.selectionEnd = el.value.length; });
    }
  }
};

// 全局快捷键处理
const handleGlobalKeydown = (e: KeyboardEvent) => {
  if (e.key === 'Escape') {
    if (slash.isSlashMenuVisible.value) {
      slash.hideSlashMenu();
      e.preventDefault();
      return;
    }
    if (showAttachMenu.value) {
      showAttachMenu.value = false;
      return;
    }
    if (showModelDropdown.value) {
      showModelDropdown.value = false;
      return;
    }
  }
};

// 全局点击监听 - 关闭下拉菜单
const handleGlobalClick = (e: Event) => {
  const target = (e.target as Element | null);
  if (showModelDropdown.value && !target?.closest('.model-dropdown-container')) {
    showModelDropdown.value = false;
  }
};

// Lifecycle
onMounted(async () => {
  document.addEventListener('keydown', handleGlobalKeydown);
  document.addEventListener('click', handleGlobalClick);
  await loadModels();
});

onUnmounted(() => {
  document.removeEventListener('keydown', handleGlobalKeydown);
  document.removeEventListener('click', handleGlobalClick);
});

// Expose methods for parent component
defineExpose({
  inputRef,
  attachedPaths,
  focus: () => inputRef.value?.focus(),
  clear: () => {
    inputText.value = '';
    attachedPaths.value = [];
  },
  setInputText: (text: string) => {
    inputText.value = text;
    nextTick(() => autoResize());
  },
  closeDropdowns: () => {
    showAttachMenu.value = false;
    showModelDropdown.value = false;
  },
  closeDropdownsOnOutsideClick: (target: Element | null) => {
    // 关闭模型下拉菜单（如果点击在外部）
    if (showModelDropdown.value && !target?.closest('.model-dropdown-container')) {
      showModelDropdown.value = false;
    }
    // 关闭附件菜单（如果点击在外部）
    if (showAttachMenu.value && !target?.closest('.attach-menu-container')) {
      showAttachMenu.value = false;
    }
  },
  setModel,
  loadModels,
  autoResize,
  inputHistory,
  isComposing,
});
</script>
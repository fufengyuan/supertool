<template>
  <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-[1000]" @click="$emit('close')">
    <div class="bg-base-100 rounded-2xl w-[90%] max-w-[640px] max-h-[85vh] overflow-y-auto shadow-2xl" @click.stop>
      <div class="flex items-center justify-between px-6 py-5 border-b border-base-content/10">
        <h3 class="m-0 text-lg font-semibold text-base-content">{{ isEditing ? '✏️ 编辑服务器' : '🖥️ 添加服务器' }}</h3>
        <button @click="$emit('close')" class="btn btn-ghost btn-sm btn-square text-xl text-base-content/60 hover:text-base-content">×</button>
      </div>
      <div class="p-6">
        <div class="grid grid-cols-2 gap-4">
          <div class="mb-4">
            <label class="block mb-1.5 text-xs font-medium text-base-content/60">服务器名称 <span class="text-error">*</span></label>
            <input v-model="localForm.name" class="input input-bordered w-full" placeholder="生产服务器" />
          </div>
          <div class="mb-4">
            <label class="block mb-1.5 text-xs font-medium text-base-content/60">端口</label>
            <input v-model.number="localForm.port" type="number" class="input input-bordered w-full" placeholder="22" />
          </div>
        </div>

        <div class="mb-4">
          <label class="block mb-1.5 text-xs font-medium text-base-content/60">主机地址</label>
          <input v-model="localForm.host" class="input input-bordered w-full" placeholder="192.168.1.100" />
        </div>

        <div class="mb-4">
          <label class="block mb-1.5 text-xs font-medium text-base-content/60">用户名</label>
          <input v-model="localForm.username" class="input input-bordered w-full" placeholder="root" />
        </div>

        <div class="bg-base-200 rounded-xl p-4 mb-4 border border-base-content/10">
          <div class="flex items-center gap-2 text-sm font-semibold text-base-content mb-4">
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="shrink-0"><path d="M21 2l-2 2m-7.61 7.61a5.5 5.5 0 1 1-7.778 7.778 5.5 5.5 0 0 1 7.777-7.777zm0 0L15.5 7.5m0 0l3 3L22 7l-3-3m-3.5 3.5L19 4"/></svg>
            <span>认证方式</span>
          </div>

          <div class="mb-4">
            <label class="block mb-1.5 text-xs font-medium text-base-content/60">SSH Key 路径</label>
            <input v-model="localForm.sshKeyPath" class="input input-bordered w-full" placeholder="~/.ssh/id_rsa" />
            <small class="block mt-1 text-xs text-base-content/40">推荐使用 SSH Key 认证，更安全</small>
          </div>

          <div class="mb-4">
            <label class="block mb-1.5 text-xs font-medium text-base-content/60">密码</label>
            <input
              v-model="localForm.password"
              type="password"
              class="input input-bordered w-full"
              autocomplete="off"
              :placeholder="isEditing ? '留空则保留原密码' : '留空则使用 Key 认证'"
            />
          </div>
        </div>

        <div class="mb-4">
          <label class="block mb-1.5 text-xs font-medium text-base-content/60">分组</label>
          <div ref="treeSelectRef" class="relative">
            <div class="input input-bordered flex items-center justify-between cursor-pointer min-h-[42px]" @click="showTreeSelect = !showTreeSelect">
              <span v-if="localForm.groupId" class="flex items-center gap-2 text-sm text-base-content">
                <span class="w-2.5 h-2.5 rounded-full flex-shrink-0" :style="{ background: selectedGroupColor }"></span>
                {{ selectedGroupName }}
              </span>
              <span v-else class="text-base-content/60 text-sm opacity-60">选择分组...</span>
              <svg class="text-base-content/60 transition-transform flex-shrink-0" :class="{ 'rotate-180': showTreeSelect }" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2.5">
                <polyline points="6 9 12 15 18 9"/>
              </svg>
            </div>
            <Transition
              enter-active-class="transition-all duration-200 ease-[cubic-bezier(0.4,0,0.2,1)]"
              leave-active-class="transition-all duration-200 ease-[cubic-bezier(0.4,0,0.2,1)]"
              enter-from-class="opacity-0 -translate-y-1.5"
              leave-to-class="opacity-0 -translate-y-1.5"
            >
              <div v-if="showTreeSelect" class="absolute top-full left-0 right-0 mt-1 bg-base-100 border border-base-content/10 rounded-xl shadow-lg z-[100] max-h-72 overflow-y-auto p-1.5">
                <div class="flex items-center gap-2 px-3 py-2 rounded-lg cursor-pointer text-xs text-base-content transition-colors select-none hover:bg-base-200"
                  :class="{ 'bg-primary/10 text-primary font-medium': !localForm.groupId }"
                  @click="selectGroup(null)">
                  无分组
                </div>
                <div v-for="g in sortedGroups" :key="g.id"
                  class="flex items-center gap-2 px-3 py-2 rounded-lg cursor-pointer text-xs text-base-content transition-colors select-none hover:bg-base-200"
                  :class="{ 'bg-primary/10 text-primary font-medium': localForm.groupId === g.id }"
                  :style="{ paddingLeft: `${12 + g.depth * 20}px` }"
                  @click="selectGroup(g.id)">
                  <span v-if="g.depth > 0" class="text-base-content/40 text-xs flex-shrink-0 w-3.5 text-center">└</span>
                  <span class="w-2 h-2 rounded-full flex-shrink-0" :style="{ background: g.color || '#6c63ff' }"></span>
                  <span class="flex-1 truncate">{{ g.name }}</span>
                </div>
              </div>
            </Transition>
          </div>
        </div>

        <div class="mb-4">
          <label class="block mb-1.5 text-xs font-medium text-base-content/60">标签（逗号分隔）</label>
          <input v-model="localForm.tagsInput" class="input input-bordered w-full" placeholder="生产, Web, API" />
        </div>

        <div class="mb-4">
          <label class="block mb-1.5 text-xs font-medium text-base-content/60">描述</label>
          <textarea
            v-model="localForm.description"
            class="textarea textarea-bordered w-full"
            placeholder="服务器描述..."
            rows="2"
          ></textarea>
        </div>

        <div class="bg-base-200 rounded-xl p-4 mb-4 border border-warning/30 bg-warning/5">
          <div class="flex items-center gap-2 text-sm font-semibold text-base-content mb-4">
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="shrink-0"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/></svg>
            <span>安全管控</span>
          </div>
          <div class="flex items-center justify-between gap-3">
            <div class="flex-1">
              <div class="text-sm font-semibold text-warning mb-1 flex items-center gap-1.5"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="11" width="18" height="11" rx="2" ry="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/></svg> 执行审核</div>
              <div class="text-xs text-base-content/60 leading-relaxed">开启后，CLI 无法在此服务器执行命令，GUI 执行需人工确认</div>
            </div>
            <input type="checkbox" class="toggle" v-model="localForm.requiresApproval" />
          </div>
        </div>
      </div>

      <div class="flex gap-3 justify-end px-6 py-4 border-t border-base-content/10">
        <button @click="$emit('test-connection')" class="btn btn-ghost btn-sm gap-1.5">
          <svg
            viewBox="0 0 24 24"
            width="14"
            height="14"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
          >
            <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14" />
            <polyline points="22 4 12 14.01 9 11.01" />
          </svg>
          测试连接
        </button>
        <button @click="$emit('close')" class="btn btn-ghost btn-sm">取消</button>
        <button @click="$emit('save')" class="btn btn-primary btn-sm">保存</button>
      </div>

      <div
        v-if="testResult"
        class="mx-6 mb-5 p-3 rounded-lg text-sm"
        :class="testResult.success ? 'bg-success/15 text-success' : 'bg-error/15 text-error'"
      >
        {{ testResult.success ? '✅ 连接成功！' : '❌ 连接失败: ' + testResult.error }}
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, computed, onMounted, onUnmounted } from 'vue';

interface ServerGroup {
  id: string;
  name: string;
  color: string;
  parentId?: string | null;
}

interface TestResult {
  success: boolean;
  error?: string;
}

const props = defineProps<{
  form: Record<string, unknown> & { name?: string; host?: string; port?: number; username?: string; password?: string; sshKeyPath?: string; tagsInput?: string; description?: string; groupId?: string | null; tags?: string[]; requiresApproval?: boolean };
  isEditing?: boolean;
  testResult: TestResult | null;
  groups: ServerGroup[];
}>();

const emit = defineEmits(['close', 'test-connection', 'save', 'update:form']);

// Local reactive wrapper to avoid mutating props directly
const localForm = ref({ ...props.form });

// Sync localForm changes back to parent — batch rapid input events
let emitTimer: ReturnType<typeof setTimeout> | null = null;
watch(localForm, (newVal) => {
  if (emitTimer) clearTimeout(emitTimer);
  emitTimer = setTimeout(() => {
    emit('update:form', { ...newVal });
  }, 16);
}, { deep: true });

// Sync prop changes back to localForm (e.g., when parent resets form)
watch(() => props.form, (newVal) => {
  localForm.value = { ...newVal };
});

// Tree select state
const showTreeSelect = ref(false);
const treeSelectRef = ref<HTMLElement | null>(null);

// Close dropdown when clicking outside
function handleClickOutside(e: MouseEvent) {
  const target = e.target as HTMLElement;
  if (treeSelectRef.value && !treeSelectRef.value.contains(target)) {
    showTreeSelect.value = false;
  }
}
onMounted(() => document.addEventListener('click', handleClickOutside));
onUnmounted(() => document.removeEventListener('click', handleClickOutside));

// Build sorted flat list with depth for tree display
const sortedGroups = computed(() => {
  const result: Array<ServerGroup & { depth: number }> = [];
  function walk(parentId: string | null, depth: number) {
    const children = props.groups.filter(g => (g.parentId || null) === parentId);
    for (const child of children) {
      result.push({ ...child, depth });
      walk(child.id, depth + 1);
    }
  }
  walk(null, 0);
  return result;
});

const selectedGroupName = computed(() => {
  const g = props.groups.find(g => g.id === localForm.value.groupId);
  return g?.name || '';
});

const selectedGroupColor = computed(() => {
  const g = props.groups.find(g => g.id === localForm.value.groupId);
  return g?.color || '#6c63ff';
});

function selectGroup(groupId: string | null) {
  localForm.value.groupId = groupId;
  showTreeSelect.value = false;
}
</script>

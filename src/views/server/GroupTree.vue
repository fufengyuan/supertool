<template>
  <!-- IDEA 风格：分组标题栏 + 服务器卡片平铺 -->
  <div class="group-tree mb-2">
    <BaseTree
      v-if="treeData.length > 0"
      ref="treeRef"
      v-model="treeDataModel"
      :children-key="childrenKey"
      :text-key="textKey"
      :indent="0"
      :default-open="defaultOpen"
      v-slot="{ stat }"
      @open:node="handleOpenNode"
      @click:node="handleClickNode"
    >
      <!-- 分组节点 -->
      <div
        v-if="stat.data.type === 'group'"
        class="group-node"
      >
        <!-- 分组标题栏 -->
        <div
          class="group-header flex items-center gap-2 px-3 py-1.5 rounded cursor-pointer select-none transition-colors"
          :style="stat.open ? { backgroundColor: getGroupBgColor(stat.data.color) } : {}"
          :class="stat.open ? '' : 'hover:bg-base-100/50'"
        >
          <!-- 展开/折叠箭头 -->
          <SvgIcon
            class="text-base-content/50 transition-transform flex-shrink-0"
            :class="{ 'rotate-180': stat.open }"
            name="chevronDown"
            size="12"
            strokeWidth="2.5"
          />
          <!-- 分组颜色标记 -->
          <span
            class="w-2 h-2 rounded-full flex-shrink-0"
            :style="{ backgroundColor: stat.data.color || '#6c63ff' }"
          ></span>
          <!-- 分组名称 -->
          <span class="font-medium text-[11px] text-base-content">{{ stat.data.name }}</span>
          <!-- 服务器数量 -->
          <span class="text-[10px] px-1.5 py-0 rounded bg-base-200 text-base-content/60 leading-tight">
            {{ stat.data.serverCount }}
          </span>
          <!-- 在线数量 -->
          <span
            v-if="stat.data.onlineCount > 0"
            class="flex items-center gap-1 text-[10px] text-success ml-auto"
          >
            <span class="w-1 h-1 rounded-full bg-success"></span>
            {{ stat.data.onlineCount }}
          </span>
        </div>

        <!-- 服务器卡片 grid 布局 -->
        <Transition
          enter-active-class="transition-all duration-200 ease-out"
          leave-active-class="transition-all duration-200 ease-in"
          enter-from-class="opacity-0 max-h-0"
          leave-to-class="opacity-0 max-h-0"
        >
          <div v-show="stat.open" class="mt-1 px-3">
            <!-- 直接子分组的服务器 -->
            <div
              v-if="stat.data.servers && stat.data.servers.length > 0"
              class="grid grid-cols-[repeat(auto-fill,minmax(200px,1fr))] gap-1.5"
            >
              <ServerItem
                v-for="server in stat.data.servers"
                :key="server.id"
                :server="server"
                :connection-status="connectionStatusMap[server.id] || 'offline'"
                @terminal="$emit('terminal', server)"
                @sftp="$emit('sftp', server)"
                @edit="$emit('edit', server)"
                @delete="$emit('delete', server.id)"
              />
            </div>
            <!-- 空状态 -->
            <div
              v-if="(!stat.data.servers || stat.data.servers.length === 0) && (!stat.children || stat.children.length === 0)"
              class="text-center py-2 text-base-content/50 text-[11px]"
            >
              暂无服务器
            </div>
          </div>
        </Transition>
      </div>
    </BaseTree>

    <!-- 空状态 -->
    <div
      v-if="treeData.length === 0"
      class="text-center py-2 text-base-content/50 text-[11px]"
    >
      暂无服务器
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { BaseTree } from '@he-tree/vue'
import '@he-tree/vue/style/default.css'
import SvgIcon from '@/components/ui/SvgIcon.vue'
import ServerItem from './ServerItem.vue'

interface GroupNode {
  id: string
  name: string
  color: string
  parentId: string | null
}

interface ServerNode {
  id: string
  name: string
  groupId: string | null
}

interface TreeNode {
  id: string
  name: string
  type: 'group'
  color?: string
  serverCount?: number
  onlineCount?: number
  servers?: ServerNode[]
  children?: TreeNode[]
}

const props = defineProps<{
  group: GroupNode
  groups: GroupNode[]
  depth: number
  expandedGroups: Set<string | null>
  servers: ServerNode[]
  connectionStatusMap: Record<string, string>
}>()

const emit = defineEmits(['toggle', 'terminal', 'sftp', 'edit', 'delete'])

const treeRef = ref<InstanceType<typeof BaseTree> | null>(null)
const childrenKey = 'children'
const textKey = 'name'

// 默认是否展开
const defaultOpen = computed(() => props.expandedGroups.has(props.group.id as string | null))

// 构建树数据 - 只包含分组层级，服务器放在每个分组节点的 servers 字段
const treeData = computed<TreeNode[]>(() => {
  return buildTreeData(props.group, props.groups, props.servers, props.connectionStatusMap)
})

// v-model 需要 mutable 数据
const treeDataModel = ref<TreeNode[]>([])
watch(treeData, (newData) => {
  treeDataModel.value = newData
}, { immediate: true })

function buildTreeData(
  group: GroupNode,
  allGroups: GroupNode[],
  allServers: ServerNode[],
  statusMap: Record<string, string>
): TreeNode[] {
  // 获取当前分组下的子分组
  const childGroups = allGroups.filter(g => g.parentId === group.id)

  // 获取当前分组下的服务器
  const serversInGroup = allServers.filter(s => s.groupId === group.id)
  const onlineCount = serversInGroup.filter(s => statusMap[s.id] === 'online').length

  // 构建子节点
  const children: TreeNode[] = []

  // 添加子分组节点（递归）
  childGroups.forEach(childGroup => {
    const childServers = allServers.filter(s => s.groupId === childGroup.id)
    const childOnlineCount = childServers.filter(s => statusMap[s.id] === 'online').length

    children.push({
      id: childGroup.id,
      name: childGroup.name,
      type: 'group',
      color: childGroup.color,
      serverCount: childServers.length,
      onlineCount: childOnlineCount,
      servers: childServers,
      children: buildTreeData(childGroup, allGroups, allServers, statusMap)
    })
  })

  // 根节点
  if (group.id === null || group.id === undefined) {
    // 根节点直接返回子分组
    return children
  }

  // 当前分组节点
  return [{
    id: group.id,
    name: group.name,
    type: 'group',
    color: group.color,
    serverCount: serversInGroup.length,
    onlineCount,
    servers: serversInGroup,
    children
  }]
}

// 将 hex 颜色转为 rgba（10% 透明度）
function getGroupBgColor(hex: string): string {
  const color = hex || '#6b7280'
  const r = parseInt(color.slice(1, 3), 16)
  const g = parseInt(color.slice(3, 5), 16)
  const b = parseInt(color.slice(5, 7), 16)
  return `rgba(${r}, ${g}, ${b}, 0.1)`
}

// 处理节点展开
function handleOpenNode(stat: any) {
  if (stat.data?.type === 'group') {
    emit('toggle', stat.data.id)
  }
}

// 处理节点点击
function handleClickNode(stat: any) {
  if (stat.data?.type === 'group') {
    // 分组节点：切换展开
    stat.open = !stat.open
    emit('toggle', stat.data.id)
  }
}
</script>

<style scoped>
.group-tree {
  font-size: 11px;
}

.group-node {
  margin-bottom: 4px;
}

/* he-tree 样式覆盖 */
.he-tree {
  font-size: inherit;
}

.he-tree .tree-node {
  padding: 0 !important;
}
</style>
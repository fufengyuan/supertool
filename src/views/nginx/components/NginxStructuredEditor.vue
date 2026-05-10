<template>
  <div class="flex flex-col gap-3 p-4">
    <!-- Toolbar -->
    <div class="flex flex-col gap-2">
      <!-- Row 1: action buttons -->
      <div class="flex items-center justify-between">
        <div class="flex gap-2 flex-wrap">
          <!-- Add block buttons -->
          <div class="dropdown dropdown-bottom">
            <label tabindex="0" class="btn btn-primary btn-xs gap-1">
              <SvgIcon name="plus" size="12" /> 添加区块
            </label>
            <ul tabindex="0" class="dropdown-content z-10 menu p-2 shadow bg-base-100 rounded-box w-40 border border-base-content/10">
              <li @click="addBlock('server')"><a>Server</a></li>
              <li @click="addBlock('upstream')"><a>Upstream</a></li>
              <li @click="addBlock('location')"><a>Location</a></li>
              <li @click="addBlock('map')"><a>Map</a></li>
              <li @click="addBlock('geo')"><a>Geo</a></li>
            </ul>
          </div>
          <div class="flex gap-1">
            <button @click="addDirectiveToFirstServer" class="btn btn-ghost btn-xs gap-1">
              <SvgIcon name="plus" size="10" /> 添加指令
            </button>
          </div>
        </div>
        <div class="flex items-center gap-2">
          <span class="text-xs text-base-content/50">{{ summaryText }}</span>
          <button v-if="configText" @click="showRaw = !showRaw" class="btn btn-ghost btn-xs">
            {{ showRaw ? '隐藏源码' : '查看源码' }}
          </button>
        </div>
      </div>
      <!-- Row 2: search -->
      <div class="flex items-center gap-2">
        <div class="relative flex-1 max-w-md">
          <SvgIcon name="search" size="12" class="absolute left-2.5 top-1/2 -translate-y-1/2 text-base-content/30 pointer-events-none" />
          <input
            v-model="searchQuery"
            type="text"
            placeholder="按域名搜索 server_name、路径、upstream..."
            class="input input-bordered input-xs w-full pl-7 pr-7 font-mono"
          />
          <button
            v-if="searchQuery"
            @click="searchQuery = ''"
            class="absolute right-1 top-1/2 -translate-y-1/2 btn btn-ghost btn-xs px-1 text-base-content/30 hover:text-base-content"
          >
            <SvgIcon name="x" size="12" />
          </button>
        </div>
        <span v-if="searchQuery && !parseError" class="text-xs text-base-content/50 shrink-0">
          匹配 {{ matchCount }} / {{ totalDisplayBlocks }} 区块
        </span>
        <span v-if="searchQuery && matchCount === 0 && blocks.length > 0" class="text-xs text-warning shrink-0">
          无匹配结果
        </span>
      </div>
    </div>

    <!-- Error state -->
    <div v-if="parseError" class="alert alert-warning text-sm py-2">
      <SvgIcon name="alert" size="14" />
      <span>配置解析部分失败，将在源码模式下显示完整内容</span>
    </div>

    <!-- No blocks -->
    <div v-if="blocks.length === 0 && !parseError" class="text-center text-base-content/50 py-10 text-sm">
      <SvgIcon name="file" size="20" class="mb-3 text-base-content/30" />
      <p class="font-medium">暂无配置内容</p>
      <p class="text-xs mt-1">点击「获取配置」从服务器拉取，或使用「添加区块」新建配置</p>
      <div class="flex justify-center gap-2 mt-3">
        <button @click="addBlock('server')" class="btn btn-outline btn-xs gap-1">
          <SvgIcon name="plus" size="10" /> 添加 Server
        </button>
        <button @click="addBlock('upstream')" class="btn btn-outline btn-xs gap-1">
          <SvgIcon name="plus" size="10" /> 添加 Upstream
        </button>
      </div>
    </div>

    <!-- Blocks list -->
    <div class="flex flex-col gap-3">
      <!-- ===== Http block ===== -->
      <template v-for="(httpBlock, hi) in httpBlocks" :key="'http-' + hi">
        <div class="collapse collapse-arrow border border-primary/20 rounded-xl bg-primary/5">
          <input type="checkbox" :checked="expandedHttp.has(hi)" @change="toggleHttp(hi)" class="peer" />
          <div class="collapse-title text-sm font-medium flex items-center gap-2 min-h-0 py-3 px-4 peer-checked:bg-primary/10">
            <SvgIcon name="layers" size="14" class="text-primary" />
            <span>http</span>
            <span class="badge badge-sm badge-ghost">{{ httpBlock.blocks.length }} server</span>
            <span class="badge badge-sm badge-ghost">{{ httpBlock.directives.length }} 指令</span>
          </div>
          <div class="collapse-content px-4 pb-4">
            <!-- Http-level directives -->
            <div v-if="httpBlock.directives.length > 0" class="mb-3">
              <div class="flex items-center gap-2 mb-1">
                <span class="text-xs font-medium text-base-content/50">http 级指令</span>
                <button @click="addDirectiveToHttp(hi)" class="btn btn-ghost btn-xs gap-1">
                  <SvgIcon name="plus" size="10" /> 添加
                </button>
              </div>
              <div class="grid grid-cols-3 gap-2">
                <div v-for="(d, di) in httpBlock.directives" :key="'hd-' + di" class="flex gap-1 items-center">
                  <span class="text-xs font-mono text-base-content/60 min-w-[60px]">{{ d.name }}</span>
                  <input :value="d.params.join(' ')" @input="updateOtherDirective(httpBlock, d, $event)" class="input input-bordered input-xs flex-1 font-mono text-xs" />
                  <button @click="removeDirectiveFromBlock(httpBlock, d)" class="btn btn-ghost btn-xs text-error px-1">
                    <SvgIcon name="x" size="10" />
                  </button>
                </div>
              </div>
            </div>
            <!-- Servers inside http -->
            <template v-for="(server, si) in getServersFromHttp(hi)" :key="'srv-http-' + si">
              <div :class="searchDimClass(serverMatchesSearch(server, searchQuery.toLowerCase().trim()))">
                <ServerBlockCard
                  :server="server"
                  :index="si"
                  :parent-type="'http-' + hi"
                  @update="emitUpdate"
                  @remove="removeServerFromHttp(hi, si)"
                  @add-location="addLocationToServerFromHttp(hi, si)"
                  @add-other-directive="addOtherDirectiveToServerFromHttp(hi, si)"
                  @remove-location="(li: number) => removeLocationFromServerBlock(server.block, li)"
                />
              </div>
            </template>
            <!-- Other blocks inside http -->
            <div v-for="(otherBlock, oi) in getOtherBlocksFromHttp(hi)" :key="'hob-' + oi"
              :class="searchDimClass(blockMatchesSearch(otherBlock, searchQuery.toLowerCase().trim()))"
            >
              <GenericBlockCard :block="otherBlock" @update="emitUpdate" />
            </div>
          </div>
        </div>
      </template>

      <!-- ===== Top-level Server blocks ===== -->
      <template v-for="(server, si) in topServerBlocks" :key="'srv-' + si">
        <div :class="searchDimClass(serverMatchesSearch(server, searchQuery.toLowerCase().trim()))">
          <ServerBlockCard
            :server="server"
            :index="si"
            parent-type="top"
            @update="emitUpdate"
            @remove="removeTopServer(si)"
            @add-location="addLocationToTopServer(si)"
            @add-other-directive="addOtherDirectiveToTopServer(si)"
            @remove-location="(li: number) => removeLocationFromServerBlock(server.block, li)"
          />
        </div>
      </template>

      <!-- ===== Upstream blocks ===== -->
      <div v-for="(up, ui) in upstreamBlocks" :key="'up-' + ui"
        :class="searchDimClass(upstreamMatchesSearch(up, searchQuery.toLowerCase().trim()))"
      >
        <div class="collapse collapse-arrow border border-accent/20 rounded-xl bg-accent/5">
        <input type="checkbox" :checked="expandedUpstream.has(ui)" @change="toggleUpstream(ui)" class="peer" />
        <div class="collapse-title text-sm font-medium flex items-center gap-2 min-h-0 py-3 px-4 peer-checked:bg-accent/10">
          <SvgIcon name="gitMerge" size="14" class="text-accent" />
          <span>{{ up.block.name }}</span>
          <span class="badge badge-sm badge-ghost">{{ up.servers.length }} 后端</span>
          <button @click.stop="removeUpstream(ui)" class="ml-auto btn btn-ghost btn-xs text-error">
            <SvgIcon name="x" size="12" />
          </button>
        </div>
        <div class="collapse-content px-4 pb-4">
          <!-- Upstream server entries -->
          <div v-for="(srv, si) in up.servers" :key="'us-' + si" class="flex items-center gap-2 mb-1.5">
            <input :value="srv.address" @input="updateUpstreamServer(ui, si, 'address', $event)" placeholder="127.0.0.1:8080" class="input input-bordered input-xs flex-1 font-mono" />
            <div class="flex items-center gap-1 text-xs text-base-content/50 shrink-0">
              <span>w</span>
              <input :value="srv.weight ?? ''" @input="updateUpstreamServer(ui, si, 'weight', $event)" type="number" min="1" max="100" class="input input-bordered input-xs w-14" />
            </div>
            <label class="flex items-center gap-1 text-xs shrink-0">
              <input type="checkbox" :checked="srv.backup ?? false" @change="updateUpstreamServerBackup(ui, si, $event)" class="checkbox checkbox-xs" />
              backup
            </label>
            <label class="flex items-center gap-1 text-xs shrink-0">
              <input type="checkbox" :checked="srv.down ?? false" @change="updateUpstreamServerDown(ui, si, $event)" class="checkbox checkbox-xs" />
              down
            </label>
            <button @click="removeUpstreamServer(ui, si)" class="btn btn-ghost btn-xs text-error px-1">
              <SvgIcon name="x" size="10" />
            </button>
          </div>
          <div class="flex gap-2 mt-2">
            <button @click="addUpstreamServer(ui)" class="btn btn-ghost btn-xs gap-1">
              <SvgIcon name="plus" size="10" /> 添加后端
            </button>
            <button @click="addOtherUpstreamDirective(ui)" class="btn btn-ghost btn-xs gap-1">
              <SvgIcon name="plus" size="10" /> 添加指令
            </button>
          </div>
          <!-- Other upstream directives -->
          <div v-for="(d, di) in up.other" :key="'upd-' + di" class="flex gap-1 mt-1.5">
            <input :value="d.name" @input="updateOtherDirectiveName(up.block, d, $event)" class="input input-bordered input-xs w-[100px] font-mono text-xs" />
            <input :value="d.params.join(' ')" @input="updateOtherDirectiveParams(up.block, d, $event)" class="input input-bordered input-xs flex-1 font-mono text-xs" />
            <button @click="removeDirectiveFromBlock(up.block, d)" class="btn btn-ghost btn-xs text-error px-1">
              <SvgIcon name="x" size="10" />
            </button>
          </div>
        </div>
      </div>
      </div>

      <!-- ===== Other blocks (events, map, geo, types, stream, etc.) ===== -->
      <div v-for="(block, bi) in otherDisplayBlocks" :key="'blk-' + bi"
        :class="searchDimClass(blockMatchesSearch(block, searchQuery.toLowerCase().trim()))"
      >
        <div class="border border-base-content/10 rounded-xl bg-base-200/30">
        <div class="flex items-center gap-2 p-3 cursor-pointer" @click="toggleOtherBlock(bi)">
          <SvgIcon :name="getBlockIcon(block.type)" size="14" :class="getBlockColor(block.type)" />
          <span class="text-sm font-mono font-medium">{{ block.name }}</span>
          <span v-if="block.directives.length > 0" class="badge badge-sm badge-ghost">{{ block.directives.length }} 指令</span>
          <span v-if="block.blocks.length > 0" class="badge badge-sm badge-ghost">{{ block.blocks.length }} 子块</span>
          <span class="ml-auto text-xs text-base-content/30">{{ collapsedOther.has(bi) ? '展开' : '收起' }}</span>
          <button @click.stop="removeOtherBlock(bi)" class="btn btn-ghost btn-xs text-error">
            <SvgIcon name="x" size="12" />
          </button>
        </div>
        <div v-show="!collapsedOther.has(bi)" class="px-3 pb-3 border-t border-base-content/10 pt-2">
          <!-- Nested sub-blocks -->
          <div v-for="(sub, si) in block.blocks" :key="'sub-' + si" class="border border-base-content/10 rounded-lg p-2 mb-2 bg-base-100">
            <div class="text-xs font-mono font-medium text-base-content/70 mb-1">{{ sub.name }}</div>
            <div v-for="(d, di) in sub.directives" :key="'sd-' + di" class="flex gap-1 mb-1">
              <input :value="d.name" @input="updateOtherDirectiveName(sub, d, $event)" class="input input-bordered input-xs w-[90px] font-mono text-xs" />
              <input :value="d.params.join(' ')" @input="updateOtherDirectiveParams(sub, d, $event)" class="input input-bordered input-xs flex-1 font-mono text-xs" />
              <button @click="removeDirectiveFromBlock(sub, d)" class="btn btn-ghost btn-xs text-error px-1">
                <SvgIcon name="x" size="10" />
              </button>
            </div>
            <button @click="addOtherDirective(sub)" class="btn btn-ghost btn-xs gap-1 mt-1">
              <SvgIcon name="plus" size="10" /> 添加指令
            </button>
          </div>
          <!-- Directives in this block -->
          <div v-for="(d, di) in block.directives" :key="'d-' + di" class="flex gap-1 mb-1">
            <input :value="d.name" @input="updateOtherDirectiveName(block, d, $event)" class="input input-bordered input-xs w-[100px] font-mono text-xs" />
            <input :value="d.params.join(' ')" @input="updateOtherDirectiveParams(block, d, $event)" class="input input-bordered input-xs flex-1 font-mono text-xs" />
            <button @click="removeDirectiveFromBlock(block, d)" class="btn btn-ghost btn-xs text-error px-1">
              <SvgIcon name="x" size="10" />
            </button>
          </div>
          <button @click="addOtherDirective(block)" class="btn btn-ghost btn-xs gap-1 mt-1">
            <SvgIcon name="plus" size="10" /> 添加指令
          </button>
        </div>
      </div>
    </div>

    <!-- ===== Unparsed blocks (parse warnings) ===== -->
    <div v-if="unparsedBlocks.length > 0" class="border border-warning/20 rounded-xl bg-warning/5 p-3">
      <div class="flex items-center gap-2 mb-2">
        <SvgIcon name="alert" size="14" class="text-warning" />
        <span class="text-sm font-medium text-warning">以下 {{ unparsedBlocks.length }} 个区块解析失败</span>
        <span class="text-xs text-base-content/50">请在「查看原生配置」中手动编辑</span>
      </div>
      <div v-for="(block, bi) in unparsedBlocks" :key="'un-' + bi" class="flex items-center gap-2 mb-1">
        <span class="font-mono text-xs text-base-content/70">{{ block.name }}</span>
        <button @click="removeBlockByRef(block)" class="btn btn-ghost btn-xs text-error ml-auto">
          <SvgIcon name="x" size="10" /> 删除
        </button>
      </div>
    </div>

    <!-- Raw config preview -->
    <div v-if="showRaw && configText" class="mt-2">
      <div class="text-xs font-medium text-base-content/50 mb-1">生成的配置</div>
      <pre class="bg-base-300 rounded-lg p-3 overflow-x-auto text-xs font-mono leading-relaxed max-h-64 overflow-y-auto whitespace-pre-wrap">{{ configText }}</pre>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import SvgIcon from '@/components/ui/SvgIcon.vue'
import ServerBlockCard from './ServerBlockCard.vue'
import {
  parseNginxConfig,
  serializeNginxConfig,
  summarizeServerBlock,
  summarizeUpstream,
  createDirective,
  type NginxBlock,
  type NginxDirective,
  type ParsedNginxConfig,
  type ServerBlockSummary,
  type UpstreamSummary,
} from '../utils/nginxParser'

const props = defineProps<{
  modelValue: string
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

const showRaw = ref(false)
const expandedHttp = ref(new Set<number>())
const expandedUpstream = ref(new Set<number>())
const collapsedOther = ref(new Set<number>())
const searchQuery = ref('')

// Auto-expand matching blocks when searching
watch(searchQuery, (q) => {
  if (!q) return
  const query = q.toLowerCase().trim()
  if (!query) return

  // Expand http blocks with matching servers
  const newExpandedHttp = new Set(expandedHttp.value)
  for (let hi = 0; hi < httpBlocks.value.length; hi++) {
    const servers = getServersFromHttp(hi)
    const hasMatch = servers.some(s => serverMatchesSearch(s, query))
    if (hasMatch) newExpandedHttp.add(hi)
  }
  expandedHttp.value = newExpandedHttp

  // Expand matching upstreams
  const newExpandedUp = new Set(expandedUpstream.value)
  for (let ui = 0; ui < upstreamBlocks.value.length; ui++) {
    if (upstreamMatchesSearch(upstreamBlocks.value[ui], query)) {
      newExpandedUp.add(ui)
    }
  }
  expandedUpstream.value = newExpandedUp
})

// ===== Search Matching =====

function serverMatchesSearch(server: ServerBlockSummary, q: string): boolean {
  if (!q) return true
  if (server.serverName.some(n => n.toLowerCase().includes(q))) return true
  if (server.listen.some(l => l.toLowerCase().includes(q))) return true
  if (server.root.toLowerCase().includes(q)) return true
  if (server.locations.some(loc => loc.path.toLowerCase().includes(q))) return true
  return false
}

function serverListMatchesSearch(servers: ServerBlockSummary[], q: string): boolean[] {
  return servers.map(s => serverMatchesSearch(s, q))
}

function upstreamMatchesSearch(up: UpstreamSummary, q: string): boolean {
  if (!q) return true
  if (up.name.toLowerCase().includes(q)) return true
  if (up.servers.some(s => s.address.toLowerCase().includes(q))) return true
  return false
}

function blockMatchesSearch(block: NginxBlock, q: string): boolean {
  if (!q) return true
  if (block.name.toLowerCase().includes(q)) return true
  if (block.directives.some(d => d.name.toLowerCase().includes(q) || d.params.some(p => p.toLowerCase().includes(q)))) return true
  return false
}

const totalDisplayBlocks = computed(() => {
  return topServerBlocks.value.length + upstreamBlocks.value.length + otherDisplayBlocks.value.length
})

const matchCount = computed(() => {
  const q = searchQuery.value.toLowerCase().trim()
  if (!q) return totalDisplayBlocks.value
  let count = 0
  count += topServerBlocks.value.filter(s => serverMatchesSearch(s, q)).length
  count += upstreamBlocks.value.filter(u => upstreamMatchesSearch(u, q)).length
  count += otherDisplayBlocks.value.filter(b => blockMatchesSearch(b, q)).length
  // Also count servers inside http blocks
  for (const http of httpBlocks.value) {
    const servers = http.blocks.filter(b => b.type === 'server').map(b => summarizeServerBlock(b))
    count += servers.filter(s => serverMatchesSearch(s, q)).length
  }
  return count
})

let parsedConfig: ParsedNginxConfig = { blocks: [], errors: [] }
const blocks = ref<NginxBlock[]>([])
const parseError = ref(false)

// Update blocks when modelValue changes
watch(() => props.modelValue, (val) => {
  if (val) {
    try {
      parsedConfig = parseNginxConfig(val)
      blocks.value = parsedConfig.blocks
      parseError.value = false
    } catch {
      parseError.value = true
      blocks.value = []
    }
  } else {
    parsedConfig = { blocks: [], errors: [] }
    blocks.value = []
    parseError.value = false
  }
}, { immediate: true })

// ===== Computed =====

const httpBlocks = computed<NginxBlock[]>(() => {
  return blocks.value.filter(b => b.type === 'http' && b.isParsed)
})

const topServerBlocks = computed<ServerBlockSummary[]>(() => {
  return blocks.value
    .filter(b => b.isParsed && b.type === 'server')
    .map(b => summarizeServerBlock(b))
})

const upstreamBlocks = computed<UpstreamSummary[]>(() => {
  return blocks.value
    .filter(b => b.isParsed && b.type === 'upstream')
    .map(b => summarizeUpstream(b))
})

const otherDisplayBlocks = computed<NginxBlock[]>(() => {
  return blocks.value
    .filter(b => b.isParsed && b.type !== 'server' && b.type !== 'upstream' && b.type !== 'http')
})

const unparsedBlocks = computed<NginxBlock[]>(() => {
  return blocks.value.filter(b => !b.isParsed)
})

const totalServerCount = computed(() => {
  let count = topServerBlocks.value.length
  for (const http of httpBlocks.value) {
    count += http.blocks.filter(b => b.type === 'server').length
  }
  return count
})

const totalUpstreamCount = computed(() => upstreamBlocks.value.length)

const summaryText = computed(() => {
  const parts: string[] = []
  if (totalServerCount.value > 0) parts.push(`${totalServerCount.value} server`)
  if (totalUpstreamCount.value > 0) parts.push(`${totalUpstreamCount.value} upstream`)
  if (blocks.value.length > 0) parts.push(`${blocks.value.length} 区块`)
  return parts.join(' · ') || '无配置'
})

// ===== Helpers =====

function getServersFromHttp(httpIdx: number): ServerBlockSummary[] {
  const httpBlock = httpBlocks.value[httpIdx]
  if (!httpBlock) return []
  return httpBlock.blocks
    .filter(b => b.type === 'server' && b.isParsed)
    .map(b => summarizeServerBlock(b))
}

function getOtherBlocksFromHttp(httpIdx: number): NginxBlock[] {
  const httpBlock = httpBlocks.value[httpIdx]
  if (!httpBlock) return []
  return httpBlock.blocks.filter(b => b.type !== 'server')
}

function getBlockIcon(type: string): string {
  const icons: Record<string, string> = {
    events: 'zap', map: 'gitMerge', geo: 'globe',
    types: 'file', stream: 'layers', mail: 'mail',
    if: 'gitBranch', limit_except: 'shield',
  }
  return icons[type] || 'file'
}

function getBlockColor(type: string): string {
  const colors: Record<string, string> = {
    events: 'text-warning', map: 'text-accent', geo: 'text-secondary',
    stream: 'text-primary', mail: 'text-info',
  }
  return colors[type] || 'text-base-content/50'
}

function emitUpdate() {
  const text = serializeNginxConfig({ blocks: blocks.value })
  emit('update:modelValue', text)
}

/** Returns CSS classes: dim block when search is active and block doesn't match */
function searchDimClass(matches: boolean): Record<string, boolean> {
  if (!searchQuery.value) return {}
  return {
    'opacity-30 pointer-events-none': !matches,
    'transition-opacity duration-200': true,
  }
}

// ===== Toggle =====

function toggleHttp(idx: number) {
  const s = new Set(expandedHttp.value)
  if (s.has(idx)) s.delete(idx); else s.add(idx)
  expandedHttp.value = s
}

function toggleUpstream(idx: number) {
  const s = new Set(expandedUpstream.value)
  if (s.has(idx)) s.delete(idx); else s.add(idx)
  expandedUpstream.value = s
}

function toggleOtherBlock(idx: number) {
  const s = new Set(collapsedOther.value)
  if (s.has(idx)) s.delete(idx); else s.add(idx)
  collapsedOther.value = s
}

// ===== Add new blocks =====

function addBlock(type: string) {
  if (type === 'server') {
    blocks.value.push(parseNginxConfig(`server {\n    listen 80;\n    server_name _;\n    root /var/www/html;\n    index index.html;\n\n    location / {\n        try_files $uri $uri/ =404;\n    }\n}\n`).blocks[0])
  } else if (type === 'upstream') {
    blocks.value.push(parseNginxConfig(`upstream backend {\n    server 127.0.0.1:8080 weight=5;\n    server 127.0.0.1:8081 weight=3;\n}\n`).blocks[0])
  } else if (type === 'location') {
    blocks.value.push(parseNginxConfig(`location / {\n    try_files $uri $uri/ =404;\n}\n`).blocks[0])
  } else if (type === 'map') {
    blocks.value.push(parseNginxConfig(`map $http_upgrade $connection_upgrade {\n    default upgrade;\n    '' close;\n}\n`).blocks[0])
  } else if (type === 'geo') {
    blocks.value.push(parseNginxConfig(`geo $country {\n    default ZZ;\n    127.0.0.1 US;\n}\n`).blocks[0])
  }
  emitUpdate()
}

function addDirectiveToFirstServer() {
  const firstServer = blocks.value.find(b => b.type === 'server')
  if (firstServer) {
    firstServer.directives.push(createDirective('new_directive', ['value']))
    emitUpdate()
  }
}

// ===== Http block mutations =====

function addDirectiveToHttp(hi: number) {
  const block = httpBlocks.value[hi]
  if (block) {
    block.directives.push(createDirective('new_directive', ['value']))
    emitUpdate()
  }
}

function removeServerFromHttp(hi: number, si: number) {
  const httpBlock = httpBlocks.value[hi]
  if (!httpBlock) return
  const serverBlocks = httpBlock.blocks.filter(b => b.type === 'server')
  if (si >= 0 && si < serverBlocks.length) {
    const idx = httpBlock.blocks.indexOf(serverBlocks[si])
    if (idx >= 0) httpBlock.blocks.splice(idx, 1)
    emitUpdate()
  }
}

function addLocationToServerFromHttp(hi: number, si: number) {
  const block = getServersFromHttp(hi)[si]?.block
  if (!block) return
  const locBlock = parseNginxConfig(`location / {\n    try_files $uri $uri/ =404;\n}\n`).blocks[0]
  if (locBlock) block.blocks.push(locBlock)
  emitUpdate()
}

function addOtherDirectiveToServerFromHttp(hi: number, si: number) {
  const block = getServersFromHttp(hi)[si]?.block
  if (!block) return
  block.directives.push(createDirective('new_directive', ['value']))
  emitUpdate()
}

// ===== Top-level server mutations =====

function removeTopServer(si: number) {
  const serverBlocksArr = blocks.value.filter(b => b.type === 'server')
  if (si >= 0 && si < serverBlocksArr.length) {
    const idx = blocks.value.indexOf(serverBlocksArr[si])
    if (idx >= 0) blocks.value.splice(idx, 1)
    emitUpdate()
  }
}

function addLocationToTopServer(si: number) {
  const block = topServerBlocks.value[si]?.block
  if (!block) return
  const locBlock = parseNginxConfig(`location / {\n    try_files $uri $uri/ =404;\n}\n`).blocks[0]
  if (locBlock) block.blocks.push(locBlock)
  emitUpdate()
}

function addOtherDirectiveToTopServer(si: number) {
  const block = topServerBlocks.value[si]?.block
  if (!block) return
  block.directives.push(createDirective('new_directive', ['value']))
  emitUpdate()
}

function removeLocationFromServerBlock(block: NginxBlock, li: number) {
  if (!block) return
  const locBlocks = block.blocks.filter(b => b.type === 'location')
  if (li >= 0 && li < locBlocks.length) {
    const idx = block.blocks.indexOf(locBlocks[li])
    if (idx >= 0) block.blocks.splice(idx, 1)
    emitUpdate()
  }
}

// ===== Upstream mutations =====

function addUpstreamServer(ui: number) {
  const block = upstreamBlocks.value[ui]?.block
  if (!block) return
  block.directives.push(createDirective('server', ['127.0.0.1:8080']))
  emitUpdate()
}

function removeUpstreamServer(ui: number, si: number) {
  const block = upstreamBlocks.value[ui]?.block
  if (!block) return
  const serverDirs = block.directives.filter(d => d.name === 'server')
  if (si >= 0 && si < serverDirs.length) {
    const idx = block.directives.indexOf(serverDirs[si])
    if (idx >= 0) block.directives.splice(idx, 1)
    emitUpdate()
  }
}

function updateUpstreamServer(ui: number, si: number, field: string, e: Event) {
  const val = (e.target as HTMLInputElement).value
  const block = upstreamBlocks.value[ui]?.block
  if (!block) return
  const serverDirs = block.directives.filter(d => d.name === 'server')
  if (si >= 0 && si < serverDirs.length) {
    const dir = serverDirs[si]
    if (field === 'address') {
      dir.params[0] = val || '127.0.0.1:8080'
    } else if (field === 'weight') {
      const weightIdx = dir.params.findIndex(p => p.startsWith('weight='))
      if (val) {
        const weightVal = `weight=${val}`
        if (weightIdx >= 0) dir.params[weightIdx] = weightVal
        else dir.params.push(weightVal)
      } else {
        if (weightIdx >= 0) dir.params.splice(weightIdx, 1)
      }
    }
    dir.raw = 'server ' + dir.params.join(' ') + ';'
    emitUpdate()
  }
}

function updateUpstreamServerBackup(ui: number, si: number, e: Event) {
  const checked = (e.target as HTMLInputElement).checked
  const block = upstreamBlocks.value[ui]?.block
  if (!block) return
  const serverDirs = block.directives.filter(d => d.name === 'server')
  if (si >= 0 && si < serverDirs.length) {
    const dir = serverDirs[si]
    const backupIdx = dir.params.indexOf('backup')
    const downIdx = dir.params.indexOf('down')
    if (checked && backupIdx < 0) {
      dir.params.splice(Math.max(downIdx, 0), 0, 'backup')
    } else if (!checked && backupIdx >= 0) {
      dir.params.splice(backupIdx, 1)
    }
    dir.raw = 'server ' + dir.params.join(' ') + ';'
    emitUpdate()
  }
}

function updateUpstreamServerDown(ui: number, si: number, e: Event) {
  const checked = (e.target as HTMLInputElement).checked
  const block = upstreamBlocks.value[ui]?.block
  if (!block) return
  const serverDirs = block.directives.filter(d => d.name === 'server')
  if (si >= 0 && si < serverDirs.length) {
    const dir = serverDirs[si]
    const downIdx = dir.params.indexOf('down')
    if (checked && downIdx < 0) {
      dir.params.push('down')
    } else if (!checked && downIdx >= 0) {
      dir.params.splice(downIdx, 1)
    }
    dir.raw = 'server ' + dir.params.join(' ') + ';'
    emitUpdate()
  }
}

function addOtherUpstreamDirective(ui: number) {
  const block = upstreamBlocks.value[ui]?.block
  if (!block) return
  block.directives.push(createDirective('new_directive', ['value']))
  emitUpdate()
}

function removeUpstream(ui: number) {
  const upBlocksArr = blocks.value.filter(b => b.type === 'upstream')
  if (ui >= 0 && ui < upBlocksArr.length) {
    const idx = blocks.value.indexOf(upBlocksArr[ui])
    if (idx >= 0) blocks.value.splice(idx, 1)
    emitUpdate()
  }
}

function removeOtherBlock(bi: number) {
  const otherBlocks = blocks.value.filter(b => b.type !== 'server' && b.type !== 'upstream' && b.type !== 'http')
  if (bi >= 0 && bi < otherBlocks.length) {
    const idx = blocks.value.indexOf(otherBlocks[bi])
    if (idx >= 0) blocks.value.splice(idx, 1)
    emitUpdate()
  }
}

function removeBlockByRef(block: NginxBlock) {
  const idx = blocks.value.indexOf(block)
  if (idx >= 0) blocks.value.splice(idx, 1)
  emitUpdate()
}

// ===== Generic directive mutations =====

function addOtherDirective(block: NginxBlock) {
  block.directives.push(createDirective('new_directive', ['value']))
  emitUpdate()
}

function removeDirectiveFromBlock(block: NginxBlock, dir: NginxDirective) {
  const idx = block.directives.indexOf(dir)
  if (idx >= 0) block.directives.splice(idx, 1)
  emitUpdate()
}

function updateOtherDirective(block: NginxBlock, dir: NginxDirective, e: Event) {
  const val = (e.target as HTMLInputElement).value
  dir.params = val ? val.split(/\s+/) : []
  dir.raw = dir.name + (dir.params.length > 0 ? ' ' + dir.params.join(' ') : '') + ';'
  emitUpdate()
}

function updateOtherDirectiveName(block: NginxBlock, dir: NginxDirective, e: Event) {
  dir.name = (e.target as HTMLInputElement).value
  dir.raw = dir.name + (dir.params.length > 0 ? ' ' + dir.params.join(' ') : '') + ';'
  emitUpdate()
}

function updateOtherDirectiveParams(block: NginxBlock, dir: NginxDirective, e: Event) {
  const val = (e.target as HTMLInputElement).value
  dir.params = val ? val.split(/\s+/) : []
  dir.raw = dir.name + (dir.params.length > 0 ? ' ' + dir.params.join(' ') : '') + ';'
  emitUpdate()
}

const configText = computed(() => {
  return serializeNginxConfig({ blocks: blocks.value })
})
</script>

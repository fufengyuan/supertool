<template>
  <div class="collapse collapse-arrow border border-primary/10 rounded-xl bg-base-200/40">
    <input type="checkbox" v-model="expanded" class="peer" />
    <div class="collapse-title text-sm font-medium flex items-center gap-2 min-h-0 py-3 px-4 peer-checked:bg-base-200/60">
      <SvgIcon name="serverRack" size="14" class="text-primary" />
      <span class="truncate">{{ displayName }}</span>
      <span v-if="server.locations.length > 0" class="badge badge-sm badge-ghost shrink-0">{{ server.locations.length }} location</span>
      <span v-if="server.sslEnabled" class="badge badge-sm badge-success shrink-0">SSL</span>
      <div class="ml-auto flex gap-1 shrink-0" @click.stop>
        <button @click="$emit('addLocation')" class="btn btn-ghost btn-xs" title="添加 location">
          <SvgIcon name="plus" size="10" /> Location
        </button>
        <button @click="confirmRemoveServer" class="btn btn-ghost btn-xs text-error" title="删除此 server 块">
          <SvgIcon name="x" size="12" />
        </button>
      </div>
    </div>
    <div class="collapse-content px-4 pb-4">
      <!-- Main server fields -->
      <div class="space-y-2 mb-3">
        <!-- Listen (multi-value) -->
        <div class="flex items-start gap-2">
          <label class="text-xs font-medium text-base-content/60 w-16 pt-1.5 shrink-0">Listen</label>
          <div class="flex-1 flex flex-col gap-1">
            <div v-for="(val, li) in server.listen" :key="'l-' + li" class="flex gap-1">
              <input :value="val" @input="updateListenValue(li, $event)" placeholder="80" class="input input-bordered input-xs flex-1 font-mono" />
              <button v-if="server.listen.length > 1" @click="removeListenValue(li)" class="btn btn-ghost btn-xs text-error px-1">
                <SvgIcon name="x" size="10" />
              </button>
            </div>
            <button @click="addListenValue" class="btn btn-ghost btn-xs justify-start text-base-content/40 gap-1 w-fit">
              <SvgIcon name="plus" size="10" /> 添加 listen
            </button>
          </div>
        </div>

        <!-- Server Name (multi-value) -->
        <div class="flex items-start gap-2">
          <label class="text-xs font-medium text-base-content/60 w-16 pt-1.5 shrink-0">Server Name</label>
          <div class="flex-1">
            <input :value="server.serverName.join(' ')" @input="updateServerNameAll($event)" placeholder="example.com *.example.com" class="input input-bordered input-xs w-full font-mono" />
          </div>
        </div>

        <div class="grid grid-cols-2 gap-2">
          <div class="flex flex-col gap-1">
            <label class="text-xs font-medium text-base-content/60">Root</label>
            <input :value="server.root" @input="updateServerRoot($event)" placeholder="/var/www/html" class="input input-bordered input-xs w-full font-mono" />
          </div>
          <div class="flex flex-col gap-1">
            <label class="text-xs font-medium text-base-content/60">Index</label>
            <input :value="server.index.join(' ')" @input="updateServerIndex($event)" placeholder="index.html index.htm" class="input input-bordered input-xs w-full font-mono" />
          </div>
        </div>
      </div>

      <!-- SSL section -->
      <div v-if="server.sslEnabled" class="mb-3 p-3 bg-base-200 rounded-lg border border-success/20">
        <div class="text-xs font-medium text-success mb-2 flex items-center gap-1">
          <SvgIcon name="shield" size="12" /> SSL 配置
        </div>
        <div class="grid grid-cols-2 gap-2">
          <div v-for="(d, di) in server.ssl" :key="'ssl-' + di" class="flex flex-col gap-0.5">
            <label class="text-[10px] text-base-content/40">{{ d.name }}</label>
            <input :value="d.params.join(' ')" @input="updateOtherDirective(server.block, d, $event)" class="input input-bordered input-xs w-full font-mono text-[11px]" />
          </div>
        </div>
      </div>

      <!-- Location blocks -->
      <div v-if="server.locations.length > 0" class="mb-2">
        <div class="text-xs font-medium text-base-content/60 mb-2 flex items-center gap-2">
          <SvgIcon name="grid" size="12" /> Location 块
          <span class="badge badge-sm badge-ghost">{{ server.locations.length }}</span>
        </div>
        <div class="flex flex-col gap-2">
          <div v-for="(loc, li) in server.locations" :key="'loc-' + li"
            class="border border-base-content/10 rounded-lg bg-base-100 overflow-hidden"
          >
            <!-- Location header -->
            <div class="flex items-center justify-between px-3 py-2 bg-base-200/50 border-b border-base-content/10">
              <div class="flex items-center gap-2">
                <SvgIcon name="folder" size="12" class="text-base-content/40" />
                <span class="text-xs font-mono font-medium">
                  <span v-if="loc.modifier" class="text-warning">{{ loc.modifier }} </span>
                  <span>{{ loc.path }}</span>
                </span>
              </div>
              <button @click="$emit('removeLocation', li)" class="btn btn-ghost btn-xs text-error" title="删除 location">
                <SvgIcon name="x" size="10" />
              </button>
            </div>

            <!-- Location body -->
            <div class="p-3 space-y-2">
              <!-- Modifier selector -->
              <div class="flex items-center gap-2">
                <label class="text-xs text-base-content/50 w-20 shrink-0">匹配模式</label>
                <select :value="loc.modifier" @change="updateLocationModifier(li, $event)" class="select select-bordered select-xs font-mono w-20">
                  <option value="">精确</option>
                  <option value="=">= 精确匹配</option>
                  <option value="~">~ 正则(区分)</option>
                  <option value="~*">~* 正则(忽略)</option>
                  <option value="^~">^~ 前缀优先</option>
                </select>
                <input :value="loc.path" @input="updateLocationPath(li, $event)" placeholder="/api" class="input input-bordered input-xs flex-1 font-mono" />
              </div>

              <!-- Proxy / Root / Try_files -->
              <div class="grid grid-cols-2 gap-2">
                <div class="flex flex-col gap-0.5">
                  <label class="text-xs text-base-content/50">Proxy Pass</label>
                  <input :value="loc.proxyPass" @input="updateLocationProxyPass(li, $event)" placeholder="http://backend" class="input input-bordered input-xs w-full font-mono text-[11px]" />
                </div>
                <div class="flex flex-col gap-0.5">
                  <label class="text-xs text-base-content/50">Root</label>
                  <input :value="loc.root" @input="updateLocationRoot(li, $event)" placeholder="/var/www/html" class="input input-bordered input-xs w-full font-mono text-[11px]" />
                </div>
              </div>
              <div class="flex flex-col gap-0.5">
                <label class="text-xs text-base-content/50">Try Files</label>
                <input :value="loc.tryFiles" @input="updateLocationTryFiles(li, $event)" placeholder="$uri $uri/ =404" class="input input-bordered input-xs w-full font-mono text-[11px]" />
              </div>

              <!-- Headers -->
              <div v-if="loc.headers.length > 0">
                <div class="text-xs text-base-content/50 mb-1">请求/响应头</div>
                <div v-for="(h, hi) in loc.headers" :key="'h-' + hi" class="flex gap-1 mb-1">
                  <span class="text-[11px] font-mono text-base-content/60 min-w-[60px]">{{ h.name }}</span>
                  <input :value="h.params.join(' ')" @input="updateOtherDirective(loc.block, h, $event)" class="input input-bordered input-xs flex-1 font-mono text-[11px]" />
                  <button @click="removeDirectiveFromBlock(loc.block, h)" class="btn btn-ghost btn-xs text-error px-1">
                    <SvgIcon name="x" size="10" />
                  </button>
                </div>
              </div>

              <!-- Sub-blocks inside location (if, limit_except) -->
              <div v-if="loc.subBlocks.length > 0">
                <div class="text-xs text-base-content/50 mb-1">子块</div>
                <div v-for="(sub, si) in loc.subBlocks" :key="'sub-' + si" class="border border-base-content/10 rounded p-2 mb-1 bg-base-200">
                  <div class="text-[11px] font-mono font-medium text-base-content/70 mb-1">{{ sub.name }}</div>
                  <div v-for="(d, di) in sub.directives" :key="'sd-' + di" class="flex gap-1 mb-0.5">
                    <input :value="d.name" @input="updateOtherDirectiveName(sub, d, $event)" class="input input-bordered input-xs w-[80px] font-mono text-[11px]" />
                    <input :value="d.params.join(' ')" @input="updateOtherDirectiveParams(sub, d, $event)" class="input input-bordered input-xs flex-1 font-mono text-[11px]" />
                    <button @click="removeDirectiveFromBlock(sub, d)" class="btn btn-ghost btn-xs text-error px-0.5">
                      <SvgIcon name="x" size="8" />
                    </button>
                  </div>
                  <button @click="addOtherDirective(sub)" class="btn btn-ghost btn-xs gap-0.5 mt-0.5">
                    <SvgIcon name="plus" size="8" /> 添加指令
                  </button>
                </div>
              </div>

              <!-- Other directives in location -->
              <div v-if="loc.other.length > 0" class="border-t border-base-content/10 pt-2">
                <div class="text-xs text-base-content/50 mb-1">其他指令</div>
                <div v-for="(d, di) in loc.other" :key="'od-' + di" class="flex gap-1 mb-1">
                  <input :value="d.name" @input="updateOtherDirectiveName(loc.block, d, $event)" class="input input-bordered input-xs w-[80px] font-mono text-[11px]" />
                  <input :value="d.params.join(' ')" @input="updateOtherDirectiveParams(loc.block, d, $event)" class="input input-bordered input-xs flex-1 font-mono text-[11px]" />
                  <button @click="removeDirectiveFromBlock(loc.block, d)" class="btn btn-ghost btn-xs text-error px-1">
                    <SvgIcon name="x" size="10" />
                  </button>
                </div>
              </div>
            </div>

            <!-- Add directive to location -->
            <div class="px-3 pb-2">
              <button @click="addDirectiveToLocation(li)" class="btn btn-ghost btn-xs gap-1">
                <SvgIcon name="plus" size="10" /> 添加指令
              </button>
            </div>
          </div>
        </div>
      </div>

      <!-- No locations yet -->
      <div v-if="server.locations.length === 0" class="mb-2">
        <div class="text-xs text-base-content/40 italic">暂无 location 块</div>
      </div>

      <!-- Sub-blocks (if, limit_except, etc.) -->
      <div v-if="server.subBlocks.length > 0" class="mb-2">
        <div class="text-xs font-medium text-base-content/60 mb-2 flex items-center gap-2">
          <SvgIcon name="gitBranch" size="12" /> 子块
          <span class="badge badge-sm badge-ghost">{{ server.subBlocks.length }}</span>
        </div>
        <div v-for="(sub, si) in server.subBlocks" :key="'sub-' + si" class="border border-base-content/10 rounded-lg p-2 mb-2 bg-base-100">
          <div class="text-[11px] font-mono font-medium text-base-content/70 mb-1.5">{{ sub.name }}</div>
          <div v-for="(d, di) in sub.directives" :key="'sd-' + di" class="flex gap-1 mb-1">
            <input :value="d.name" @input="updateOtherDirectiveName(sub, d, $event)" class="input input-bordered input-xs w-[80px] font-mono text-[11px]" />
            <input :value="d.params.join(' ')" @input="updateOtherDirectiveParams(sub, d, $event)" class="input input-bordered input-xs flex-1 font-mono text-[11px]" />
            <button @click="removeDirectiveFromBlock(sub, d)" class="btn btn-ghost btn-xs text-error px-0.5">
              <SvgIcon name="x" size="8" />
            </button>
          </div>
          <button @click="addOtherDirective(sub)" class="btn btn-ghost btn-xs gap-0.5 mt-0.5">
            <SvgIcon name="plus" size="8" /> 添加指令
          </button>
        </div>
      </div>

      <!-- Other directives in server -->
      <div class="border-t border-base-content/10 pt-2">
        <div class="flex items-center gap-2 mb-1">
          <span class="text-xs font-medium text-base-content/50">其他指令</span>
          <button @click="$emit('addOtherDirective')" class="btn btn-ghost btn-xs gap-1">
            <SvgIcon name="plus" size="10" /> 添加
          </button>
        </div>
        <div v-for="(d, di) in server.other" :key="'od-' + di" class="flex gap-1 mb-1">
          <input :value="d.name" @input="updateOtherDirectiveName(server.block, d, $event)" placeholder="指令名" class="input input-bordered input-xs w-[100px] font-mono text-xs" />
          <input :value="d.params.join(' ')" @input="updateOtherDirectiveParams(server.block, d, $event)" placeholder="参数值 …" class="input input-bordered input-xs flex-1 font-mono text-xs" />
          <button @click="removeDirectiveFromBlock(server.block, d)" class="btn btn-ghost btn-xs text-error px-1">
            <SvgIcon name="x" size="10" />
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import SvgIcon from '@/components/ui/SvgIcon.vue'
import {
  summarizeLocationBlock,
  createDirective,
  splitParamsSmart,
  joinParamsDisplay,
  type ServerBlockSummary,
  type NginxDirective,
} from '../utils/nginxParser'

const props = defineProps<{
  server: ServerBlockSummary
  index: number
  parentType: string
}>()

const emit = defineEmits<{
  update: []
  remove: []
  addLocation: []
  addOtherDirective: []
  removeLocation: [index: number]
}>()

const expanded = ref(true)

const displayName = computed(() => {
  const names = props.server.serverName
  const listen = props.server.listen
  if (names.length > 0) {
    const shown = names.slice(0, 2).join(' ')
    return names.length > 2 ? `server { ${shown} ... }` : `server { ${shown} }`
  }
  if (listen.length > 0) {
    return `server { listen ${listen[0]} }`
  }
  return 'server { ... }'
})

// ===== Listen mutations =====
function addListenValue() {
  const block = props.server.block
  block.directives.push(createDirective('listen', ['80']))
  emitUpdate()
}
function removeListenValue(idx: number) {
  const block = props.server.block
  const listenDirs = block.directives.filter(d => d.name === 'listen')
  if (idx >= 0 && idx < listenDirs.length) {
    const found = block.directives.indexOf(listenDirs[idx])
    if (found >= 0) block.directives.splice(found, 1)
    emitUpdate()
  }
}
function updateListenValue(idx: number, e: Event) {
  const val = (e.target as HTMLInputElement).value
  const block = props.server.block
  const listenDirs = block.directives.filter(d => d.name === 'listen')
  if (idx >= 0 && idx < listenDirs.length) {
    listenDirs[idx].params = val ? [val] : []
    listenDirs[idx].raw = val ? `listen ${val};` : ''
    emitUpdate()
  }
}

// ===== Server Name =====
function updateServerNameAll(e: Event) {
  const val = (e.target as HTMLInputElement).value
  const parts = val.split(/\s+/).filter(Boolean)
  const block = props.server.block
  const existing = block.directives.find(d => d.name === 'server_name')
  if (existing) {
    existing.params = parts
    existing.raw = parts.length > 0 ? `server_name ${parts.join(' ')};` : ''
  } else if (parts.length > 0) {
    block.directives.push(createDirective('server_name', parts))
  }
  emitUpdate()
}

// ===== Root =====
function updateServerRoot(e: Event) {
  const val = (e.target as HTMLInputElement).value
  const block = props.server.block
  const existing = block.directives.find(d => d.name === 'root')
  if (existing) {
    existing.params = val ? [val] : []
    existing.raw = val ? `root ${val};` : ''
  } else if (val) {
    block.directives.push(createDirective('root', [val]))
  }
  emitUpdate()
}

// ===== Index =====
function updateServerIndex(e: Event) {
  const val = (e.target as HTMLInputElement).value
  const parts = val.split(/\s+/).filter(Boolean)
  const block = props.server.block
  const existing = block.directives.find(d => d.name === 'index')
  if (existing) {
    existing.params = parts
    existing.raw = parts.length > 0 ? `index ${parts.join(' ')};` : ''
  } else if (parts.length > 0) {
    block.directives.push(createDirective('index', parts))
  }
  emitUpdate()
}

// ===== Location mutations =====
function updateLocationModifier(li: number, e: Event) {
  const val = (e.target as HTMLSelectElement).value
  const loc = props.server.locations[li]
  if (!loc) return
  const block = loc.block
  const oldModifier = loc.modifier
  const path = loc.path

  if (val !== oldModifier) {
    if (val) {
      block.params = [val, ...path.split(/\s+/)].filter(Boolean)
    } else {
      block.params = path.split(/\s+/)
    }
    block.name = 'location ' + block.params.join(' ')
    emitUpdate()
  }
}

function updateLocationPath(li: number, e: Event) {
  const val = (e.target as HTMLInputElement).value
  const loc = props.server.locations[li]
  if (!loc) return
  const block = loc.block
  if (loc.modifier) {
    block.params = [loc.modifier, ...val.split(/\s+/)].filter(Boolean)
  } else {
    block.params = val.split(/\s+/)
  }
  block.name = 'location ' + block.params.join(' ')
  emitUpdate()
}

function updateLocationProxyPass(li: number, e: Event) {
  const val = (e.target as HTMLInputElement).value
  const loc = props.server.locations[li]
  if (!loc) return
  const block = loc.block
  const existing = block.directives.find(d => d.name === 'proxy_pass')
  if (existing) {
    existing.params = val ? [val] : []
    existing.raw = val ? `proxy_pass ${val};` : ''
  } else if (val) {
    block.directives.push(createDirective('proxy_pass', [val]))
  }
  emitUpdate()
}

function updateLocationRoot(li: number, e: Event) {
  const val = (e.target as HTMLInputElement).value
  const loc = props.server.locations[li]
  if (!loc) return
  const block = loc.block
  const existing = block.directives.find(d => d.name === 'root')
  if (existing) {
    existing.params = val ? [val] : []
    existing.raw = val ? `root ${val};` : ''
  } else if (val) {
    block.directives.push(createDirective('root', [val]))
  }
  emitUpdate()
}

function updateLocationTryFiles(li: number, e: Event) {
  const val = (e.target as HTMLInputElement).value
  const loc = props.server.locations[li]
  if (!loc) return
  const block = loc.block
  const parts = val.split(/\s+/).filter(Boolean)
  const existing = block.directives.find(d => d.name === 'try_files')
  if (existing) {
    existing.params = parts
    existing.raw = parts.length > 0 ? `try_files ${parts.join(' ')};` : ''
  } else if (parts.length > 0) {
    block.directives.push(createDirective('try_files', parts))
  }
  emitUpdate()
}

function addDirectiveToLocation(li: number) {
  const loc = props.server.locations[li]
  if (!loc) return
  loc.block.directives.push(createDirective('new_directive', ['value']))
  emitUpdate()
}

// ===== Generic =====
function emitUpdate() {
  emit('update')
}

function removeDirectiveFromBlock(block: any, dir: NginxDirective) {
  const idx = block.directives.indexOf(dir)
  if (idx >= 0) block.directives.splice(idx, 1)
  emitUpdate()
}

function updateOtherDirective(block: any, dir: NginxDirective, e: Event) {
  const val = (e.target as HTMLInputElement).value
  dir.params = val ? splitParamsSmart(val) : []
  dir.raw = dir.name + (dir.params.length > 0 ? ' ' + joinParamsDisplay(dir.params) : '') + ';'
  emitUpdate()
}

function updateOtherDirectiveName(block: any, dir: NginxDirective, e: Event) {
  dir.name = (e.target as HTMLInputElement).value
  dir.raw = dir.name + (dir.params.length > 0 ? ' ' + joinParamsDisplay(dir.params) : '') + ';'
  emitUpdate()
}

function updateOtherDirectiveParams(block: any, dir: NginxDirective, e: Event) {
  const val = (e.target as HTMLInputElement).value
  dir.params = val ? splitParamsSmart(val) : []
  dir.raw = dir.name + (dir.params.length > 0 ? ' ' + joinParamsDisplay(dir.params) : '') + ';'
  emitUpdate()
}

function addOtherDirective(block: any) {
  block.directives.push(createDirective('new_directive', ['value']))
  emitUpdate()
}

function confirmRemoveServer() {
  if (confirm(`确定删除 server 块「${displayName.value}」？`)) {
    emit('remove')
  }
}
</script>

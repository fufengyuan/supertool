<template>
  <div>
    <!-- 工具栏 -->
    <div class="flex items-center justify-between mb-3">
      <h3 class="text-base font-semibold m-0">Server 配置</h3>
      <div class="flex items-center gap-2">
        <input
          v-model="searchText"
          placeholder="搜索 serverName..."
          class="input input-bordered input-xs w-40"
        />
        <button @click="openAddDialog" class="btn btn-primary btn-sm">
          <SvgIcon name="plus" size="14" /> 新增 Server
        </button>
      </div>
    </div>

    <!-- 加载中 -->
    <div v-if="loading" class="flex items-center justify-center py-8 text-base-content/50">
      <SvgIcon name="clock" size="14" /> 加载中...
    </div>

    <!-- 空状态 -->
    <div v-else-if="filteredServers.length === 0" class="flex flex-col items-center justify-center py-8 text-base-content/50">
      <SvgIcon name="server" size="24" class="mb-2 opacity-50" />
      <p class="text-sm">{{ searchText ? '未匹配到结果' : '暂无 Server 配置' }}</p>
    </div>

    <!-- 表格 -->
    <div v-else class="overflow-x-auto">
      <table class="table table-zebra table-xs">
        <thead>
          <tr>
            <th class="w-8 text-center">类型</th>
            <th>监听</th>
            <th>域名</th>
            <th class="text-center">SSL</th>
            <th class="w-14 text-center">启用</th>
            <th class="w-36 text-center">操作</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="(svr, index) in filteredServers" :key="svr.id">
            <td class="text-center">
              <span
                class="tooltip inline-flex"
                :data-tip="svr.proxyType === 1 ? 'TCP' : svr.proxyType === 2 ? 'UDP' : 'HTTP'"
              >
                <SvgIcon
                  :name="svr.proxyType === 1 ? 'activity' : svr.proxyType === 2 ? 'radio' : 'globe'"
                  size="14"
                  :class="svr.proxyType === 1 ? 'text-info' : svr.proxyType === 2 ? 'text-warning' : 'text-primary'"
                />
              </span>
            </td>
            <td class="font-mono text-xs">{{ formatListen(svr) }}</td>
            <td class="text-xs">{{ svr.serverName || '-' }}</td>
            <td class="text-center">
              <span v-if="svr.ssl == 1" class="badge badge-xs badge-success">SSL</span>
              <span v-else class="badge badge-xs badge-ghost">否</span>
            </td>
            <td class="text-center">
              <input
                type="checkbox"
                :checked="svr.enabled !== false"
                @change="toggleEnabled(svr)"
                class="checkbox checkbox-xs"
              />
            </td>
            <td class="text-center">
              <div class="flex items-center justify-center gap-0.5">
                <div class="flex flex-col gap-0">
                  <button
                    @click="moveUp(index)"
                    :disabled="index === 0"
                    class="btn btn-ghost btn-xs btn-square"
                    title="上移"
                  >
                    <SvgIcon name="chevronUp" size="8" />
                  </button>
                  <button
                    @click="moveDown(index)"
                    :disabled="index === filteredServers.length - 1"
                    class="btn btn-ghost btn-xs btn-square"
                    title="下移"
                  >
                    <SvgIcon name="chevronDown" size="8" />
                  </button>
                </div>
                <button @click="openEditDialog(svr)" class="btn btn-ghost btn-xs btn-square" title="编辑">
                  <SvgIcon name="pencil" size="12" />
                </button>
                <button @click="onCloneServer(svr)" class="btn btn-ghost btn-xs btn-square" title="克隆">
                  <SvgIcon name="clipboard" size="12" />
                </button>
                <button @click="onDeleteServer(svr.id)" class="btn btn-ghost btn-xs btn-square text-error" title="删除">
                  <SvgIcon name="trash" size="12" />
                </button>
              </div>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- 新增/编辑弹窗 - 抽屉式 -->
    <div v-if="showDialog" class="fixed inset-0 z-50">
      <!-- 遮罩层 -->
      <div class="fixed inset-0 bg-black/50" @click="closeDialog"></div>
      <!-- 抽屉面板 -->
      <div class="fixed inset-y-0 right-0 w-[88%] bg-base-100 shadow-2xl flex flex-col">
        <!-- 标题栏 -->
        <div class="flex items-center justify-between px-6 py-4 border-b border-base-content/10 shrink-0">
          <h3 class="font-bold text-lg">{{ editingServer ? '编辑 Server' : '新增 Server' }}</h3>
          <button @click="closeDialog" class="btn btn-ghost btn-sm btn-square">
            <SvgIcon name="x" size="18" />
          </button>
        </div>
        <!-- 内容区 -->
        <div class="flex-1 overflow-y-auto px-6 py-5">
          <!-- 基础字段：3 列布局 -->
          <div class="grid grid-cols-3 gap-x-8 gap-y-4">
            <div class="flex flex-col gap-1.5">
              <label class="text-sm font-medium">代理类型</label>
              <select v-model="form.proxyType" class="select select-bordered w-full">
                <option :value="0">HTTP</option>
                <option :value="1">TCP</option>
                <option :value="2">UDP</option>
              </select>
            </div>
            <div class="flex flex-col gap-1.5">
              <label class="text-sm font-medium">监听 IP</label>
              <input v-model="form.ip" placeholder="0.0.0.0" class="input input-bordered w-full" />
            </div>
            <div class="flex flex-col gap-1.5">
              <label class="text-sm font-medium">端口 <span class="text-error">*</span></label>
              <input v-model="form.listen" type="number" placeholder="80" class="input input-bordered w-full" />
            </div>
            <div class="flex flex-col gap-1.5">
              <label class="text-sm font-medium">serverName</label>
              <input v-model="form.serverName" placeholder="example.com" class="input input-bordered w-full" />
            </div>
            <div class="flex flex-col gap-1.5">
              <label class="text-sm font-medium">密码验证</label>
              <select v-model="form.passwordId" class="select select-bordered w-full">
                <option value="">无</option>
                <option v-for="pw in passwords" :key="pw.id" :value="pw.id">{{ pw.name || pw.path }}</option>
              </select>
            </div>
            <div class="flex flex-col gap-1.5">
              <label class="text-sm font-medium">代理 Upstream</label>
              <select v-model="form.proxyUpstreamId" class="select select-bordered w-full">
                <option value="">无</option>
                <option v-for="up in upstreams" :key="up.id" :value="up.id">{{ up.name }}</option>
              </select>
            </div>
            <div class="flex flex-col gap-1.5">
              <label class="text-sm font-medium">描述</label>
              <input v-model="form.descr" placeholder="可选描述" class="input input-bordered w-full" />
            </div>
            <div class="flex flex-col gap-1.5">
              <label class="text-sm font-medium">HTTP→HTTPS 重写</label>
              <select v-model.number="form.rewrite" class="select select-bordered w-full">
                <option :value="0">关闭</option>
                <option :value="1">开启</option>
              </select>
            </div>
            <div class="flex items-end gap-4 pb-2">
              <label class="flex items-center gap-2 text-sm cursor-pointer">
                <input type="checkbox" v-model="form.def" class="checkbox checkbox-sm" /> default_server
              </label>
              <label class="flex items-center gap-2 text-sm cursor-pointer">
                <input type="checkbox" v-model="form.proxyProtocol" class="checkbox checkbox-sm" /> proxy protocol
              </label>
              <label class="flex items-center gap-2 text-sm cursor-pointer">
                <input type="checkbox" v-model="form.ipv6" class="checkbox checkbox-sm" /> IPv6
              </label>
            </div>
          </div>

          <!-- 中间区域：SSL + 黑白名单 横排双卡片 -->
          <div class="grid grid-cols-2 gap-6 mt-6">
            <!-- SSL 卡片 -->
            <div class="border border-base-content/10 rounded-xl p-5 bg-base-200/30">
              <div class="flex items-center justify-between mb-4">
                <h4 class="text-sm font-semibold">SSL 配置</h4>
                <select v-model.number="form.ssl" class="select select-sm select-bordered w-20">
                  <option :value="0">关闭</option>
                  <option :value="1">开启</option>
                </select>
              </div>
              <template v-if="form.ssl == 1">
                <div class="grid grid-cols-2 gap-x-4 gap-y-3">
                  <div class="flex flex-col gap-1 col-span-2">
                    <label class="text-sm text-base-content/70">证书</label>
                    <select v-model="form.certId" class="select select-bordered w-full">
                      <option value="">无</option>
                      <option v-for="c in certs" :key="c.id" :value="c.id">{{ c.name || c.domain || c.pem }}</option>
                    </select>
                  </div>
                  <div class="flex flex-col gap-1">
                    <label class="text-sm text-base-content/70">PEM 路径</label>
                    <input v-model="form.pem" placeholder="/etc/nginx/ssl/cert.pem" class="input input-bordered w-full" />
                  </div>
                  <div class="flex flex-col gap-1">
                    <label class="text-sm text-base-content/70">Key 路径</label>
                    <input v-model="form.key" placeholder="/etc/nginx/ssl/cert.key" class="input input-bordered w-full" />
                  </div>
                  <div class="flex flex-col gap-1">
                    <label class="text-sm text-base-content/70">HTTP/2</label>
                    <select v-model.number="form.http2" class="select select-bordered w-full">
                      <option :value="0">禁用</option>
                      <option :value="1">旧版 (h2)</option>
                      <option :value="2">新版 (h2c)</option>
                    </select>
                  </div>
                  <div class="flex flex-col gap-1">
                    <label class="text-sm text-base-content/70">重定向端口</label>
                    <input v-model="form.rewriteListen" type="number" placeholder="80" class="input input-bordered w-full" />
                  </div>
                  <div class="col-span-2 flex items-center gap-2">
                    <input type="checkbox" v-model="form.rewrite" class="checkbox checkbox-sm" id="ssl-rewrite" />
                    <label for="ssl-rewrite" class="text-sm cursor-pointer">HTTP → HTTPS 重定向</label>
                  </div>
                  <div class="col-span-2">
                    <label class="text-sm text-base-content/70 block mb-1.5">TLS 协议</label>
                    <div class="flex flex-wrap gap-3">
                      <label v-for="proto in tlsOptions" :key="proto.value" class="flex items-center gap-1.5 text-sm cursor-pointer">
                        <input type="checkbox" :checked="selectedProtocols.includes(proto.value)" @change="toggleProtocol(proto.value)" class="checkbox checkbox-sm" />
                        {{ proto.label }}
                      </label>
                    </div>
                  </div>
                </div>
              </template>
              <template v-else>
                <p class="text-sm text-base-content/40 text-center py-4">SSL 已关闭</p>
              </template>
            </div>

            <!-- IP 黑白名单 + HTTP 重写 卡片 -->
            <div class="border border-base-content/10 rounded-xl p-5 bg-base-200/30">
              <h4 class="text-sm font-semibold mb-4">IP 黑白名单</h4>
              <div class="flex items-center gap-3 mb-4">
                <span class="text-sm text-base-content/70">策略</span>
                <select v-model.number="form.denyAllow" class="select select-sm select-bordered w-28">
                  <option :value="0">无</option>
                  <option :value="1">仅拒绝</option>
                  <option :value="2">仅允许</option>
                  <option :value="3">同时</option>
                </select>
              </div>
              <div v-if="form.denyAllow > 0" class="grid grid-cols-2 gap-x-4 gap-y-3">
                <div class="flex flex-col gap-1">
                  <label class="text-sm text-base-content/70">拒绝规则</label>
                  <select v-model="form.denyId" class="select select-bordered w-full">
                    <option value="">无</option>
                    <option v-for="da in denyAllows" :key="da.id" :value="da.id">{{ da.name || da.ip }}</option>
                  </select>
                </div>
                <div class="flex flex-col gap-1">
                  <label class="text-sm text-base-content/70">允许规则</label>
                  <select v-model="form.allowId" class="select select-bordered w-full">
                    <option value="">无</option>
                    <option v-for="da in denyAllows" :key="da.id" :value="da.id">{{ da.name || da.ip }}</option>
                  </select>
                </div>
              </div>
              <div v-if="form.denyAllow === 0" class="text-sm text-base-content/40 text-center py-3">未启用黑白名单</div>
            </div>
          </div>

          <!-- Locations 子表 -->
          <div class="mt-6 border-t border-base-content/10 pt-5">
            <div class="flex items-center justify-between mb-3">
              <span class="text-sm font-semibold">Location 规则</span>
              <button @click="onAddLocation" class="btn btn-primary btn-sm">
                <SvgIcon name="plus" size="14" /> 新增 Location
              </button>
            </div>
            <div v-if="locations.length === 0" class="text-center py-6 text-base-content/50">暂无 Location 规则</div>
            <div v-else class="overflow-x-auto">
              <table class="table table-zebra table-sm">
                <thead>
                  <tr>
                    <th class="w-8 text-center"><input type="checkbox" @change="toggleAllLocations($event)" class="checkbox checkbox-xs" /></th>
                    <th class="w-28">路径</th>
                    <th class="w-28">类型</th>
                    <th class="w-44">目标</th>
                    <th class="w-48">Upstream</th>
                    <th class="w-20 text-center">功能</th>
                    <th class="w-16 text-center">排序</th>
                    <th class="w-20 text-center">操作</th>
                  </tr>
                </thead>
                <tbody>
                  <tr v-for="(loc, idx) in locations" :key="loc._key || idx">
                    <td class="text-center"><input type="checkbox" v-model="loc.enabled" class="checkbox checkbox-xs" /></td>
                    <td><input v-model="loc.path" placeholder="/api" class="input input-bordered input-xs w-full" /></td>
                    <td>
                      <select v-model="loc.locType" class="select select-xs select-bordered w-full">
                        <option :value="0">反向代理 (proxy_pass)</option>
                        <option :value="1">静态文件 (root)</option>
                        <option :value="2">上游代理 (upstream)</option>
                        <option :value="3">空白 (blank)</option>
                        <option :value="4">重定向 (return)</option>
                      </select>
                    </td>
                    <td>
                      <input v-if="loc.locType === 0" v-model="loc.value" placeholder="http://..." class="input input-bordered input-xs w-full" />
                      <input v-else-if="loc.locType === 1" v-model="loc.rootPath" placeholder="root 路径" class="input input-bordered input-xs w-full" />
                      <input v-else-if="loc.locType === 4" v-model="loc.returnUrl" placeholder="URL" class="input input-bordered input-xs w-full" />
                      <span v-else class="text-xs text-base-content/40">自动</span>
                    </td>
                    <td>
                      <div class="flex items-center gap-1">
                        <select v-model="loc.upstreamId" class="select select-xs select-bordered flex-1 min-w-0">
                          <option value="">无</option>
                          <option v-for="up in upstreams" :key="up.id" :value="up.id">{{ up.name }}</option>
                        </select>
                        <input v-if="loc.upstreamId" v-model="loc.upstreamPath" placeholder="/" class="input input-bordered input-xs w-16" />
                      </div>
                    </td>
                    <td class="text-center">
                      <div class="flex items-center justify-center gap-1">
                        <span v-if="loc.header" class="tooltip" data-tip="Header"><span class="badge badge-sm badge-info">H</span></span>
                        <span v-if="loc.websocket" class="tooltip" data-tip="WebSocket"><span class="badge badge-sm badge-success">WS</span></span>
                        <span v-if="loc.cros" class="tooltip" data-tip="CROS"><span class="badge badge-sm badge-warning">C</span></span>
                      </div>
                    </td>
                    <td class="text-center">
                      <div class="flex items-center gap-0.5 justify-center">
                        <button @click="moveLocationUp(idx)" :disabled="idx === 0" class="btn btn-ghost btn-xs btn-square"><SvgIcon name="chevronUp" size="12" /></button>
                        <span class="text-xs text-base-content/50 w-4">{{ loc.sort ?? idx + 1 }}</span>
                        <button @click="moveLocationDown(idx)" :disabled="idx === locations.length - 1" class="btn btn-ghost btn-xs btn-square"><SvgIcon name="chevronDown" size="12" /></button>
                      </div>
                    </td>
                    <td class="text-center">
                      <div class="flex items-center justify-center gap-0.5">
                        <button @click="openLocationParams(idx)" class="btn btn-ghost btn-xs btn-square" title="额外参数"><SvgIcon name="menu" size="12" /></button>
                        <button @click="onDeleteLocation(idx)" class="btn btn-ghost btn-xs btn-square text-error" title="删除"><SvgIcon name="x" size="14" /></button>
                      </div>
                    </td>
                  </tr>
                </tbody>
              </table>
            </div>
          </div>

          <!-- Location 额外参数弹窗 -->
          <div v-if="showLocParamDialog" class="fixed inset-0 z-[60]">
            <div class="fixed inset-0 bg-black/40" @click="showLocParamDialog = false"></div>
            <div class="fixed inset-y-0 right-0 w-[40%] min-w-[400px] bg-base-100 shadow-2xl flex flex-col">
              <div class="flex items-center justify-between px-5 py-3.5 border-b border-base-content/10 shrink-0">
                <h4 class="font-semibold text-sm">Location 额外参数</h4>
                <button @click="showLocParamDialog = false" class="btn btn-ghost btn-xs btn-square"><SvgIcon name="x" size="16" /></button>
              </div>
              <div class="flex-1 overflow-y-auto px-5 py-4">
                <div v-if="locParamEntries.length === 0" class="text-sm text-base-content/50 text-center py-6">暂无额外参数</div>
                <div v-for="(entry, ei) in locParamEntries" :key="ei" class="flex items-center gap-2 mb-2">
                  <input v-model="entry.name" placeholder="指令名" class="input input-bordered input-sm w-36 font-mono text-xs" />
                  <input v-model="entry.value" placeholder="值" class="input input-bordered input-sm flex-1 font-mono text-xs" />
                  <button @click="locParamEntries.splice(ei, 1)" class="btn btn-ghost btn-xs btn-square text-error"><SvgIcon name="x" size="12" /></button>
                </div>
              </div>
              <div class="flex items-center justify-between px-5 py-3 border-t border-base-content/10 shrink-0">
                <button @click="locParamEntries.push({ name: '', value: '' })" class="btn btn-ghost btn-xs"><SvgIcon name="plus" size="12" /> 添加参数</button>
                <div class="flex gap-2">
                  <button @click="showLocParamDialog = false" class="btn btn-ghost btn-sm">关闭</button>
                  <button @click="saveLocationParams" class="btn btn-primary btn-sm">保存</button>
                </div>
              </div>
            </div>
          </div>

          <!-- 隐藏 paramJson -->
          <textarea v-model="form.paramJson" class="hidden"></textarea>
        </div>
        <!-- 底部操作栏 -->
        <div class="flex items-center justify-end gap-2 px-6 py-4 border-t border-base-content/10 shrink-0">
          <button @click="closeDialog" class="btn btn-ghost btn-sm">取消</button>
          <button @click="onSave" class="btn btn-primary btn-sm" :disabled="!form.serverName && !form.listen && !form.ip">保存</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { getTauriAPI } from '../../utils/tauri-api'
import { useToast } from '../../composables/useToast'
import SvgIcon from '@/components/ui/SvgIcon.vue'

const props = defineProps<{ presetId: string }>()

const toast = useToast()
const loading = ref(false)
const showDialog = ref(false)
const editingServer = ref<any>(null)
const searchText = ref('')
const api = getTauriAPI()

// 主数据
const servers = ref<any[]>([])
const upstreams = ref<any[]>([])
const passwords = ref<any[]>([])
const denyAllows = ref<any[]>([])
const certs = ref<any[]>([])

// 表单
const form = ref({
  id: '',
  presetId: '',
  proxyType: 0,
  listen: '',
  ip: '',
  def: false,
  ipv6: false,
  proxyProtocol: false,
  serverName: '',
  ssl: 0,
  certId: '',
  pem: '',
  key: '',
  rewrite: false,
  rewriteListen: '',
  http2: 0,
  protocols: '',
  passwordId: '',
  denyAllow: 0,
  denyId: '',
  allowId: '',
  proxyUpstreamId: '',
  descr: '',
  enabled: true,
  sort: 0,
  paramJson: '',
  createdAt: '',
  updatedAt: '',
})

// TLS 选项
const tlsOptions = [
  { value: 'TLSv1', label: 'TLSv1' },
  { value: 'TLSv1.1', label: 'TLSv1.1' },
  { value: 'TLSv1.2', label: 'TLSv1.2' },
  { value: 'TLSv1.3', label: 'TLSv1.3' },
]

const selectedProtocols = ref<string[]>([])

function toggleProtocol(val: string) {
  const idx = selectedProtocols.value.indexOf(val)
  if (idx >= 0) {
    selectedProtocols.value.splice(idx, 1)
  } else {
    selectedProtocols.value.push(val)
  }
  form.value.protocols = selectedProtocols.value.join(' ')
}

// Locations 子表
const locations = ref<any[]>([])
const showLocParamDialog = ref(false)
const locParamEntries = ref<Array<{name: string, value: string}>>([])
let editingLocIndex = -1

// 搜索过滤
const filteredServers = computed(() => {
  if (!searchText.value) return servers.value
  const q = searchText.value.toLowerCase()
  return servers.value.filter(
    (s) =>
      (s.serverName && s.serverName.toLowerCase().includes(q)) ||
      (s.listen && s.listen.toLowerCase().includes(q)) ||
      (s.ip && String(s.ip).includes(q))
  )
})

// 工具
function formatListen(svr: any) {
  let result = svr.ip || ''
  if (svr.listen) result += (result ? ':' : '') + (typeof svr.listen === 'number' ? svr.listen : svr.listen)
  if (!result) result = svr.listen || svr.ip || '-'
  return result
}

// 加载数据
async function loadData() {
  if (!props.presetId) return
  loading.value = true
  try {
    const [svrResult, upResult, pwResult, daResult, certResult] = await Promise.all([
      api.getServersByPreset(props.presetId),
      api.getUpstreamsByPreset(props.presetId),
      api.getPasswordsByPreset(props.presetId),
      api.getDenyAllowsByPreset(props.presetId),
      api.getCertsByPreset(props.presetId),
    ])
    servers.value = (svrResult?.data ?? svrResult ?? []).sort(
      (a: any, b: any) => (a.sort ?? 0) - (b.sort ?? 0)
    )
    upstreams.value = upResult?.data ?? upResult ?? []
    passwords.value = pwResult?.data ?? pwResult ?? []
    denyAllows.value = daResult?.data ?? daResult ?? []
    certs.value = certResult?.data ?? certResult ?? []
  } catch (err: any) {
    toast.error('加载数据失败: ' + (err?.message || err))
  } finally {
    loading.value = false
  }
}

watch(() => props.presetId, () => { loadData() }, { immediate: true })

function resetForm() {
  form.value = {
    id: crypto.randomUUID(),
    presetId: props.presetId,
    proxyType: 0,
    listen: '',
    ip: '',
    def: false,
    ipv6: false,
    proxyProtocol: false,
    serverName: '',
    ssl: 0,
    certId: '',
    pem: '',
    key: '',
    rewrite: false,
    rewriteListen: '',
    http2: 0,
    protocols: '',
    passwordId: '',
    denyAllow: 0,
    denyId: '',
    allowId: '',
    proxyUpstreamId: '',
    descr: '',
    enabled: true,
    sort: 0,
    paramJson: '',
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
  }
  selectedProtocols.value = []
  locations.value = []
}

function openAddDialog() {
  editingServer.value = null
  resetForm()
  showDialog.value = true
}

async function openEditDialog(svr: any) {
  editingServer.value = svr
  form.value = { ...svr }
  // 解析 protocols
  if (svr.protocols) {
    selectedProtocols.value = svr.protocols.split(/[, ]+/).filter(Boolean)
  } else {
    selectedProtocols.value = []
  }
  showDialog.value = true
  // 加载 locations
  try {
    const result = await api.getLocationsByServer(svr.id)
    locations.value = (result?.data ?? result ?? []).map((l: any) => ({
      ...l,
      _key: crypto.randomUUID(),
    }))
  } catch (err: any) {
    toast.error('加载 Location 失败: ' + (err?.message || err))
    locations.value = []
  }
}

function closeDialog() {
  showDialog.value = false
  editingServer.value = null
}

// ---- CRUD ----

async function onSave() {
  try {
    // 组装完整的 server 对象
    const serverData = {
      id: form.value.id,
      presetId: form.value.presetId,
      proxyType: form.value.proxyType,
      listen: form.value.listen,
      ip: form.value.ip,
      def: form.value.def,
      ipv6: form.value.ipv6,
      proxyProtocol: form.value.proxyProtocol,
      serverName: form.value.serverName,
      ssl: form.value.ssl,
      certId: form.value.certId,
      pem: form.value.pem,
      key: form.value.key,
      rewrite: form.value.rewrite,
      rewriteListen: form.value.rewriteListen,
      http2: form.value.http2,
      protocols: form.value.protocols,
      passwordId: form.value.passwordId,
      denyAllow: form.value.denyAllow,
      denyId: form.value.denyId,
      allowId: form.value.allowId,
      proxyUpstreamId: form.value.proxyUpstreamId,
      descr: form.value.descr,
      enabled: form.value.enabled,
      sort: form.value.sort,
      paramJson: form.value.paramJson || '',
      createdAt: form.value.createdAt,
      updatedAt: new Date().toISOString(),
    }

    if (editingServer.value) {
      await api.updateNginxServer(serverData)
      const idx = servers.value.findIndex((s) => s.id === serverData.id)
      if (idx !== -1) servers.value[idx] = serverData
      toast.success('Server 已更新')
    } else {
      const result = await api.addNginxServer(serverData)
      const saved = result?.data ?? result
      servers.value.push({ ...serverData, ...saved })
      toast.success('Server 已添加')
    }

    // 保存 locations - 新增
    const LOC_TYPE_MAP: Record<string, number> = {
      proxy_pass: 0,
      root: 1,
      upstream: 2,
      blank: 3,
      return: 4,
    }
    for (const loc of locations.value) {
      loc.serverId = form.value.id
      loc.locType = typeof loc.type === 'string' ? (LOC_TYPE_MAP[loc.type] ?? 0) : (loc.locType ?? loc.type ?? 0)
      delete loc.type
      if (loc._key && !loc.id) {
        // 新增
        const newLoc = {
          id: crypto.randomUUID(),
          serverId: form.value.id,
          enabled: loc.enabled !== false,
          path: loc.path || '',
          type: loc.type || 'proxy_pass',
          value: loc.value || '',
          rootPath: loc.rootPath || '',
          upstreamId: loc.upstreamId || '',
          upstreamPath: loc.upstreamPath || '',
          header: loc.header || false,
          websocket: loc.websocket || false,
          cros: loc.cros || false,
          returnUrl: loc.returnUrl || '',
          sort: loc.sort ?? 0,
          createdAt: new Date().toISOString(),
          updatedAt: new Date().toISOString(),
        }
        delete loc._key
        try {
          await api.addNginxLocation(newLoc)
        } catch (e: any) {
          // location may already exist
        }
      } else if (loc.id && !loc._deleted) {
        // 更新
        loc.updatedAt = new Date().toISOString()
        try {
          await api.updateNginxLocation(loc)
        } catch (e: any) {
          // ignore
        }
      }
    }

    closeDialog()
    await loadData()
  } catch (err: any) {
    toast.error('保存失败: ' + (err?.message || err))
  }
}

async function onCloneServer(svr: any) {
  const clone = {
    ...svr,
    id: crypto.randomUUID(),
    serverName: svr.serverName ? svr.serverName + ' (副本)' : '',
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
  }
  try {
    await api.addNginxServer(clone)
    toast.success('Server 已克隆')
    await loadData()
  } catch (err: any) {
    toast.error('克隆失败: ' + (err?.message || err))
  }
}

async function onDeleteServer(id: string) {
  try {
    await api.deleteNginxServer(id)
    servers.value = servers.value.filter((s) => s.id !== id)
    toast.success('Server 已删除')
  } catch (err: any) {
    toast.error('删除失败: ' + (err?.message || err))
  }
}

async function toggleEnabled(svr: any) {
  svr.enabled = !svr.enabled
  svr.updatedAt = new Date().toISOString()
  try {
    await api.updateNginxServer(svr)
  } catch (err: any) {
    toast.error('更新失败: ' + (err?.message || err))
    svr.enabled = !svr.enabled
  }
}

// 排序
async function moveUp(index: number) {
  if (index <= 0) return
  swapServers(index, index - 1)
}

async function moveDown(index: number) {
  if (index >= filteredServers.value.length - 1) return
  swapServers(index, index + 1)
}

async function swapServers(i: number, j: number) {
  const arr = servers.value
  const temp = arr[i].sort
  arr[i].sort = arr[j].sort
  arr[j].sort = temp
  ;[arr[i], arr[j]] = [arr[j], arr[i]]
  servers.value = [...arr]
  try {
    await Promise.all([
      api.updateNginxServer(arr[i]),
      api.updateNginxServer(arr[j]),
    ])
  } catch (err: any) {
    toast.error('排序更新失败')
    await loadData()
  }
}

// Location 操作
function onAddLocation() {
  locations.value.push({
    serverId: form.value.id,
    enabled: true,
    path: '',
    locType: 0,
    value: '',
    rootPath: '',
    upstreamId: '',
    upstreamPath: '',
    header: false,
    websocket: false,
    cros: false,
    returnUrl: '',
    sort: locations.value.length + 1,
    paramJson: '',
    _key: crypto.randomUUID(),
  })
}

function openLocationParams(idx: number) {
  editingLocIndex = idx
  const loc = locations.value[idx]
  try {
    const parsed = loc.paramJson ? JSON.parse(loc.paramJson) : []
    locParamEntries.value = Array.isArray(parsed) ? parsed.map((p: any) => ({ name: p.name || '', value: p.value || '' })) : []
  } catch {
    locParamEntries.value = []
  }
  showLocParamDialog.value = true
}

function saveLocationParams() {
  if (editingLocIndex >= 0 && editingLocIndex < locations.value.length) {
    locations.value[editingLocIndex].paramJson = JSON.stringify(locParamEntries.value.filter(e => e.name.trim()))
  }
  showLocParamDialog.value = false
}

function onDeleteLocation(idx: number) {
  const loc = locations.value[idx]
  if (loc.id) {
    // 已存在的记录标记删除
    loc._deleted = true
    // 尝试从后端删除
    api.deleteNginxLocation(loc.id).catch(() => {})
  }
  locations.value.splice(idx, 1)
}

function toggleAllLocations(event: Event) {
  const checked = (event.target as HTMLInputElement).checked
  locations.value.forEach(loc => { loc.enabled = checked })
}

function moveLocationUp(idx: number) {
  const arr = locations.value
  const temp = arr[idx].sort
  arr[idx].sort = arr[idx - 1].sort
  arr[idx - 1].sort = temp
  ;[arr[idx], arr[idx - 1]] = [arr[idx - 1], arr[idx]]
  locations.value = [...arr]
}

function moveLocationDown(idx: number) {
  if (idx >= locations.value.length - 1) return
  const arr = locations.value
  const temp = arr[idx].sort
  arr[idx].sort = arr[idx + 1].sort
  arr[idx + 1].sort = temp
  ;[arr[idx], arr[idx + 1]] = [arr[idx + 1], arr[idx]]
  locations.value = [...arr]
}
</script>

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
                <button @click="onRowPreview(svr)" class="btn btn-ghost btn-xs btn-square" title="配置预览">
                  <SvgIcon name="eye" size="12" />
                </button>
              </div>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- 列表页配置预览弹窗 -->
    <div v-if="showListPreview" class="fixed inset-0 z-[70]">
      <div class="fixed inset-0 bg-black/50" @click="showListPreview = false"></div>
      <div class="fixed inset-y-0 right-0 w-[75%] min-w-[700px] max-w-[1100px] bg-base-100 shadow-2xl flex flex-col">
        <div class="flex items-center justify-between px-6 py-4 border-b border-base-content/10 shrink-0">
          <h3 class="font-bold text-sm">Server 配置预览</h3>
          <button @click="showListPreview = false" class="btn btn-ghost btn-sm btn-square">
            <SvgIcon name="x" size="18" />
          </button>
        </div>
        <div class="flex-1 overflow-auto p-5">
          <pre v-if="listPreviewContent" class="nginx-highlight text-xs font-mono leading-relaxed whitespace-pre-wrap bg-base-200 rounded-lg p-4 overflow-x-auto"><code v-html="listHighlightedPreview"></code></pre>
          <div v-else-if="listPreviewLoading" class="flex items-center justify-center h-full text-base-content/50 text-sm">
            <SvgIcon name="clock" size="14" class="mr-2" /> 生成中...
          </div>
          <div v-else class="flex items-center justify-center h-full text-base-content/50 text-sm">点击「配置预览」按钮生成配置</div>
        </div>
        <div class="flex items-center justify-end gap-2 px-6 py-4 border-t border-base-content/10 shrink-0">
          <button @click="copyListPreview" v-if="listPreviewContent" class="btn btn-ghost btn-sm gap-1">
            <SvgIcon name="clipboard" size="14" /> 复制
          </button>
          <button @click="showListPreview = false" class="btn btn-primary btn-sm">关闭</button>
        </div>
      </div>
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
        <div class="flex-1 overflow-y-auto px-6 py-5 space-y-5">
          <!-- 基本配置 -->
          <div class="border border-base-content/10 rounded-xl bg-base-200/20">
            <div class="px-5 py-2.5 border-b border-base-content/10">
              <h4 class="text-sm font-semibold">基本配置</h4>
            </div>
            <div class="p-5">
              <div class="grid grid-cols-4 gap-x-5 gap-y-3">
                <div class="flex flex-col gap-1">
                  <label class="text-xs text-base-content/70">代理类型</label>
                  <select v-model="form.proxyType" class="select select-sm select-bordered">
                    <option :value="0">HTTP</option>
                    <option :value="1">TCP</option>
                    <option :value="2">UDP</option>
                  </select>
                </div>
                <div class="flex flex-col gap-1">
                  <label class="text-xs text-base-content/70">监听端口 <span class="text-error">*</span></label>
                  <input v-model="form.listen" type="number" placeholder="80" class="input input-sm input-bordered w-28" />
                </div>
                <div class="flex flex-col gap-1">
                  <label class="text-xs text-base-content/70">serverName</label>
                  <input v-model="form.serverName" placeholder="example.com" class="input input-sm input-bordered w-full" />
                </div>
                <div class="flex items-end gap-3 pb-1">
                  <label class="flex items-center gap-1.5 text-xs cursor-pointer">
                    <input type="checkbox" v-model="form.def" class="checkbox checkbox-xs" /> default
                  </label>
                  <label class="flex items-center gap-1.5 text-xs cursor-pointer">
                    <input type="checkbox" v-model="form.ipv6" class="checkbox checkbox-xs" /> IPv6
                  </label>
                </div>
                <div class="flex flex-col gap-1 col-span-4">
                  <label class="text-xs text-base-content/70">描述</label>
                  <input v-model="form.descr" placeholder="可选描述" class="input input-sm input-bordered w-full" />
                </div>
                <div class="flex flex-col gap-1">
                  <label class="text-xs text-base-content/70">密码验证</label>
                  <select v-model="form.passwordId" class="select select-sm select-bordered w-full">
                    <option value="">无</option>
                    <option v-for="pw in passwords" :key="pw.id" :value="pw.id">{{ pw.name || pw.path }}</option>
                  </select>
                </div>
                <div class="flex items-center gap-4 pt-1">
                  <label class="flex items-center gap-1.5 text-xs cursor-pointer">
                    <input type="checkbox" v-model="form.rewrite" class="checkbox checkbox-xs" /> HTTP→HTTPS
                  </label>
                  <label class="flex items-center gap-1.5 text-xs cursor-pointer">
                    <input type="checkbox" v-model="form.proxyProtocol" class="checkbox checkbox-xs" /> proxy protocol
                  </label>
                </div>
              </div>
              <!-- TCP/UDP proxy: show upstream/proxy_pass -->
              <div v-if="form.proxyType !== 0" class="mt-3 pt-3 border-t border-base-content/10 grid grid-cols-2 gap-x-5 gap-y-3">
                <div class="flex flex-col gap-1">
                  <label class="text-xs text-base-content/70">代理 Upstream</label>
                  <select v-model="form.proxyUpstreamId" class="select select-sm select-bordered w-full">
                    <option value="">无</option>
                    <option v-for="up in upstreams" :key="up.id" :value="up.id">{{ up.name }}</option>
                  </select>
                </div>
                <div class="flex flex-col gap-1">
                  <label class="text-xs text-base-content/70">监听 IP</label>
                  <input v-model="form.ip" placeholder="0.0.0.0" class="input input-sm input-bordered" />
                </div>
              </div>
            </div>
          </div>

          <!-- SSL 配置 + IP 黑白名单 横排双卡片 -->
          <div class="grid grid-cols-2 gap-5">
            <div class="border border-base-content/10 rounded-xl bg-base-200/20">
              <div class="flex items-center justify-between px-5 py-2.5 border-b border-base-content/10">
                <h4 class="text-sm font-semibold">SSL 配置</h4>
                <select v-model.number="form.ssl" class="select select-xs select-bordered w-16">
                  <option :value="0">关闭</option>
                  <option :value="1">开启</option>
                </select>
              </div>
              <div class="p-4">
                <template v-if="form.ssl == 1">
                  <div class="flex flex-col gap-3">
                    <div class="flex flex-col gap-1">
                      <label class="text-xs text-base-content/70">证书</label>
                      <select v-model="form.certId" class="select select-sm select-bordered w-full">
                        <option value="">无</option>
                        <option v-for="c in certs" :key="c.id" :value="c.id">{{ c.name || c.domain || c.pem }}</option>
                      </select>
                    </div>
                    <div class="grid grid-cols-2 gap-3">
                      <div class="flex flex-col gap-1">
                        <label class="text-xs text-base-content/70">PEM 路径</label>
                        <input v-model="form.pem" placeholder="/etc/nginx/ssl/cert.pem" class="input input-sm input-bordered w-full" />
                      </div>
                      <div class="flex flex-col gap-1">
                        <label class="text-xs text-base-content/70">Key 路径</label>
                        <input v-model="form.key" placeholder="/etc/nginx/ssl/cert.key" class="input input-sm input-bordered w-full" />
                      </div>
                    </div>
                    <div class="grid grid-cols-2 gap-3">
                      <div class="flex flex-col gap-1">
                        <label class="text-xs text-base-content/70">HTTP/2</label>
                        <select v-model.number="form.http2" class="select select-sm select-bordered w-28">
                          <option :value="0">禁用</option>
                          <option :value="1">旧版</option>
                          <option :value="2">新版</option>
                        </select>
                      </div>
                      <div class="flex flex-col gap-1">
                        <label class="text-xs text-base-content/70">重定向端口</label>
                        <input v-model="form.rewriteListen" type="number" placeholder="80" class="input input-sm input-bordered w-28" />
                      </div>
                    </div>
                    <div class="flex flex-col gap-1">
                      <label class="text-xs text-base-content/70">TLS 协议</label>
                      <div class="flex flex-wrap gap-3">
                        <label v-for="proto in tlsOptions" :key="proto.value" class="flex items-center gap-1 text-xs cursor-pointer">
                          <input type="checkbox" :checked="selectedProtocols.includes(proto.value)" @change="toggleProtocol(proto.value)" class="checkbox checkbox-xs" />
                          {{ proto.label }}
                        </label>
                      </div>
                    </div>
                  </div>
                </template>
                <p v-else class="text-xs text-base-content/40 text-center py-3">SSL 已关闭</p>
              </div>
            </div>

            <div class="border border-base-content/10 rounded-xl bg-base-200/20">
              <div class="px-5 py-2.5 border-b border-base-content/10">
                <h4 class="text-sm font-semibold">IP 黑白名单</h4>
              </div>
              <div class="p-4">
                <div class="flex items-center gap-3 mb-3">
                  <span class="text-xs text-base-content/70">策略</span>
                  <select v-model.number="form.denyAllow" class="select select-xs select-bordered w-28">
                    <option :value="0">无</option>
                    <option :value="1">仅拒绝</option>
                    <option :value="2">仅允许</option>
                    <option :value="3">同时</option>
                  </select>
                </div>
                <div v-if="form.denyAllow > 0" class="grid grid-cols-2 gap-3">
                  <div class="flex flex-col gap-1">
                    <label class="text-xs text-base-content/70">拒绝规则</label>
                    <select v-model="form.denyId" class="select select-sm select-bordered w-full">
                      <option value="">无</option>
                      <option v-for="da in denyAllows" :key="da.id" :value="da.id">{{ da.name || da.ip }}</option>
                    </select>
                  </div>
                  <div class="flex flex-col gap-1">
                    <label class="text-xs text-base-content/70">允许规则</label>
                    <select v-model="form.allowId" class="select select-sm select-bordered w-full">
                      <option value="">无</option>
                      <option v-for="da in denyAllows" :key="da.id" :value="da.id">{{ da.name || da.ip }}</option>
                    </select>
                  </div>
                </div>
                <p v-if="form.denyAllow === 0" class="text-xs text-base-content/40 text-center py-2">未启用黑白名单</p>
              </div>
            </div>
          </div>

          <!-- Server 额外参数 -->
          <div class="flex justify-end">
            <button @click="openServerParams" class="btn btn-ghost btn-xs gap-1">
              <SvgIcon name="menu" size="12" /> 额外参数
            </button>
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
                    <th class="w-36">路径</th>
                    <th class="w-24">类型</th>
                    <th class="w-1/2">目标/配置</th>
                    <th class="w-14 text-center">排序</th>
                    <th class="w-16 text-center">操作</th>
                  </tr>
                </thead>
                <tbody>
                  <tr v-for="(loc, idx) in locations" :key="loc._key || idx">
                    <td class="text-center"><input type="checkbox" v-model="loc.enabled" class="checkbox checkbox-xs" /></td>
                    <td><input v-model="loc.path" placeholder="/api" class="input input-bordered input-xs w-full" /></td>
                    <td>
                      <select v-model="loc.locType" class="select select-xs select-bordered w-full">
                        <option :value="0">反向代理</option>
                        <option :value="1">静态目录</option>
                        <option :value="2">上游集群</option>
                        <option :value="3">空白</option>
                        <option :value="4">重定向</option>
                      </select>
                    </td>
                    <td>
                      <!-- proxy_pass type: all inline -->
                      <template v-if="loc.locType === 0">
                        <div class="flex items-center gap-1">
                          <input v-model="loc.value" placeholder="http://127.0.0.1:8080" class="input input-bordered input-xs w-52 font-mono" />
                          <div class="flex items-center gap-2 ml-1">
                            <label class="flex items-center gap-0.5 text-[11px] cursor-pointer" title="WebSocket 支持">
                              <input type="checkbox" v-model="loc.websocket" class="checkbox checkbox-xs" />WebSocket
                            </label>
                            <label class="flex items-center gap-0.5 text-[11px] cursor-pointer" title="跨域 CORS">
                              <input type="checkbox" v-model="loc.cros" class="checkbox checkbox-xs" />跨域
                            </label>
                            <label class="flex items-center gap-0.5 text-[11px] cursor-pointer" title="Host 请求头转发">
                              <input type="checkbox" v-model="loc.header" class="checkbox checkbox-xs" />Host转发
                            </label>
                            <template v-if="loc.header">
                              <select :value="getHeaderHostSelectValue(loc)" @change="onHeaderHostSelect(loc, ($event.target as HTMLSelectElement).value)" class="select select-xs select-bordered w-22 font-mono">
                                <option value="$host">$host</option>
                                <option value="$http_host">$http_host</option>
                                <option value="$proxy_host">$proxy_host</option>
                                <option value="$server_name">$server_name</option>
                                <option value="__custom__">其他...</option>
                              </select>
                              <input v-if="isCustomHeaderHost(loc.headerHost)" v-model="loc.headerHost" placeholder="自定义值" class="input input-bordered input-xs w-20 font-mono" />
                            </template>
                          </div>
                          <button @click="openLocationParams(idx)" class="btn btn-ghost btn-xs btn-square" title="额外参数"><SvgIcon name="menu" size="12" /></button>
                        </div>
                      </template>
                      <!-- upstream type -->
                      <template v-else-if="loc.locType === 2">
                        <div class="flex items-center gap-1">
                          <select v-model="loc.upstreamId" class="select select-xs select-bordered w-36">
                            <option value="">选择上游集群</option>
                            <option v-for="up in upstreams" :key="up.id" :value="up.id">{{ up.name }}</option>
                          </select>
                          <input v-if="loc.upstreamId" v-model="loc.upstreamPath" placeholder="/path" class="input input-bordered input-xs w-16 font-mono" />
                          <div class="flex items-center gap-2 ml-1">
                            <label class="flex items-center gap-0.5 text-[11px] cursor-pointer" title="WebSocket 支持">
                              <input type="checkbox" v-model="loc.websocket" class="checkbox checkbox-xs" />WebSocket
                            </label>
                            <label class="flex items-center gap-0.5 text-[11px] cursor-pointer" title="跨域 CORS">
                              <input type="checkbox" v-model="loc.cros" class="checkbox checkbox-xs" />跨域
                            </label>
                            <label class="flex items-center gap-0.5 text-[11px] cursor-pointer" title="Host 请求头转发">
                              <input type="checkbox" v-model="loc.header" class="checkbox checkbox-xs" />Host转发
                            </label>
                            <template v-if="loc.header">
                              <select :value="getHeaderHostSelectValue(loc)" @change="onHeaderHostSelect(loc, ($event.target as HTMLSelectElement).value)" class="select select-xs select-bordered w-22 font-mono">
                                <option value="$host">$host</option>
                                <option value="$http_host">$http_host</option>
                                <option value="$proxy_host">$proxy_host</option>
                                <option value="$server_name">$server_name</option>
                                <option value="__custom__">其他...</option>
                              </select>
                              <input v-if="isCustomHeaderHost(loc.headerHost)" v-model="loc.headerHost" placeholder="自定义值" class="input input-bordered input-xs w-20 font-mono" />
                            </template>
                          </div>
                          <button @click="openLocationParams(idx)" class="btn btn-ghost btn-xs btn-square" title="额外参数"><SvgIcon name="menu" size="12" /></button>
                        </div>
                      </template>
                      <!-- root type -->
                      <template v-else-if="loc.locType === 1">
                        <div class="flex items-center gap-1">
                          <select v-model="loc.rootType" class="select select-xs select-bordered w-16">
                            <option value="root">root目录</option>
                            <option value="alias">alias别名</option>
                          </select>
                          <input v-model="loc.rootPath" placeholder="/var/www/html" class="input input-bordered input-xs w-48 font-mono" />
                          <input v-model="loc.rootPage" placeholder="index.html" class="input input-bordered input-xs w-36 font-mono" />
                          <button @click="openLocationParams(idx)" class="btn btn-ghost btn-xs btn-square" title="额外参数"><SvgIcon name="menu" size="12" /></button>
                        </div>
                      </template>
                      <!-- return type -->
                      <template v-else-if="loc.locType === 4">
                        <div class="flex items-center gap-1">
                          <select v-model="loc.value" class="select select-xs select-bordered w-14">
                            <option value="301">301</option>
                            <option value="302">302</option>
                            <option value="307">307</option>
                            <option value="308">308</option>
                          </select>
                          <input v-model="loc.returnUrl" placeholder="https://example.com$request_uri" class="input input-bordered input-xs w-80 font-mono" />
                          <label class="flex items-center gap-0.5 text-[11px] cursor-pointer shrink-0">
                            <input type="checkbox" v-model="loc.returnPath" class="checkbox checkbox-xs" />
                            追加路径
                          </label>
                          <button @click="openLocationParams(idx)" class="btn btn-ghost btn-xs btn-square" title="额外参数"><SvgIcon name="menu" size="12" /></button>
                        </div>
                      </template>
                      <!-- blank -->
                      <span v-else class="text-xs text-base-content/40">—</span>
                    </td>
                    <td>
                      <div class="flex items-center gap-0.5 justify-center">
                        <button @click="moveLocationUp(idx)" :disabled="idx === 0" class="btn btn-ghost btn-xs btn-square"><SvgIcon name="chevronUp" size="12" /></button>
                        <span class="text-xs text-base-content/50 w-4">{{ loc.sort ?? idx + 1 }}</span>
                        <button @click="moveLocationDown(idx)" :disabled="idx === locations.length - 1" class="btn btn-ghost btn-xs btn-square"><SvgIcon name="chevronDown" size="12" /></button>
                      </div>
                    </td>
                    <td class="text-center">
                      <button @click="onDeleteLocation(idx)" class="btn btn-ghost btn-xs btn-square text-error" title="删除"><SvgIcon name="x" size="14" /></button>
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
                <div v-for="(entry, ei) in locParamEntries" :key="ei" class="flex flex-col gap-1 mb-3 p-2 rounded-lg border border-base-content/10">
                  <div class="flex items-center gap-2">
                    <select v-model="entry.templateId" class="select select-bordered select-xs flex-1 font-mono text-xs">
                      <option value="">— 自定义参数 —</option>
                      <option v-for="tpl in locParamTemplates" :key="tpl.id" :value="tpl.id">{{ tpl.name }}</option>
                    </select>
                    <button @click="locParamEntries.splice(ei, 1)" class="btn btn-ghost btn-xs btn-square text-error shrink-0"><SvgIcon name="x" size="12" /></button>
                  </div>
                  <div v-if="!entry.templateId" class="flex items-center gap-2">
                    <input v-model="entry.name" placeholder="指令名" class="input input-bordered input-sm w-36 font-mono text-xs" />
                    <input v-model="entry.value" placeholder="值" class="input input-bordered input-sm flex-1 font-mono text-xs" />
                  </div>
                  <div v-else class="text-xs text-base-content/50 pl-1">
                    使用模板: <span class="text-primary font-medium">{{ getTemplateName(entry.templateId) }}</span>
                  </div>
                </div>
              </div>
              <div class="flex items-center justify-between px-5 py-3 border-t border-base-content/10 shrink-0">
                <button @click="locParamEntries.push({ name: '', value: '', templateId: '' })" class="btn btn-ghost btn-xs"><SvgIcon name="plus" size="12" /> 添加参数</button>
                <div class="flex gap-2">
                  <button @click="showLocParamDialog = false" class="btn btn-ghost btn-sm">关闭</button>
                  <button @click="saveLocationParams" class="btn btn-primary btn-sm">保存</button>
                </div>
              </div>
            </div>
          </div>

          <!-- Server 额外参数弹窗 -->
          <div v-if="showServerParamDialog" class="fixed inset-0 z-[60]">
            <div class="fixed inset-0 bg-black/40" @click="showServerParamDialog = false"></div>
            <div class="fixed inset-y-0 right-0 w-[40%] min-w-[400px] bg-base-100 shadow-2xl flex flex-col">
              <div class="flex items-center justify-between px-5 py-3.5 border-b border-base-content/10 shrink-0">
                <h4 class="font-semibold text-sm">Server 额外参数</h4>
                <button @click="showServerParamDialog = false" class="btn btn-ghost btn-xs btn-square"><SvgIcon name="x" size="16" /></button>
              </div>
              <div class="flex-1 overflow-y-auto px-5 py-4">
                <div class="text-xs text-base-content/50 mb-3">自定义参数会注入到 server {} 块中，position=1 前置（在 listen 之后），=0 后置（在 Location 之后）</div>
                <div v-if="serverParamEntries.length === 0" class="text-sm text-base-content/50 text-center py-6">暂无额外参数</div>
                <div v-for="(entry, ei) in serverParamEntries" :key="ei" class="flex flex-col gap-1 mb-3 p-2 rounded-lg border border-base-content/10">
                  <div class="flex items-center gap-2">
                    <select v-model="entry.templateId" class="select select-bordered select-xs flex-1 font-mono text-xs">
                      <option value="">— 自定义参数 —</option>
                      <option v-for="tpl in serverParamTemplates" :key="tpl.id" :value="tpl.id">{{ tpl.name }}</option>
                    </select>
                    <label class="text-xs text-base-content/50 shrink-0">位置</label>
                    <select v-model.number="entry.position" class="select select-bordered select-xs w-16">
                      <option :value="0">后置</option>
                      <option :value="1">前置</option>
                    </select>
                    <button @click="serverParamEntries.splice(ei, 1)" class="btn btn-ghost btn-xs btn-square text-error shrink-0"><SvgIcon name="x" size="12" /></button>
                  </div>
                  <div v-if="!entry.templateId" class="flex items-center gap-2">
                    <input v-model="entry.name" placeholder="指令名" class="input input-bordered input-sm w-36 font-mono text-xs" />
                    <input v-model="entry.value" placeholder="值" class="input input-bordered input-sm flex-1 font-mono text-xs" />
                  </div>
                  <div v-else class="text-xs text-base-content/50 pl-1">
                    使用模板: <span class="text-primary font-medium">{{ getServerTemplateName(entry.templateId) }}</span>
                  </div>
                </div>
              </div>
              <div class="flex items-center justify-between px-5 py-3 border-t border-base-content/10 shrink-0">
                <button @click="serverParamEntries.push({ name: '', value: '', position: 0, templateId: '' })" class="btn btn-ghost btn-xs"><SvgIcon name="plus" size="12" /> 添加参数</button>
                <div class="flex gap-2">
                  <button @click="showServerParamDialog = false" class="btn btn-ghost btn-sm">关闭</button>
                  <button @click="saveServerParams" class="btn btn-primary btn-sm">保存</button>
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
          <button @click="onPreview" class="btn btn-ghost btn-sm gap-1">
            <SvgIcon name="eye" size="14" /> 预览
          </button>
          <button @click="onSave" class="btn btn-primary btn-sm" :disabled="!form.serverName && !form.listen && !form.ip">保存</button>
        </div>
      </div>
    </div>

    <!-- 预览弹窗 -->
    <div v-if="showPreview" class="fixed inset-0 z-[70]">
      <div class="fixed inset-0 bg-black/50" @click="showPreview = false"></div>
      <div class="fixed inset-y-0 right-0 w-[75%] min-w-[700px] max-w-[1100px] bg-base-100 shadow-2xl flex flex-col">
        <div class="flex items-center justify-between px-6 py-4 border-b border-base-content/10 shrink-0">
          <h3 class="font-bold text-sm">配置预览</h3>
          <button @click="showPreview = false" class="btn btn-ghost btn-sm btn-square">
            <SvgIcon name="x" size="18" />
          </button>
        </div>
        <div class="flex-1 overflow-auto p-5">
          <pre v-if="previewContent" class="nginx-highlight text-xs font-mono leading-relaxed whitespace-pre-wrap bg-base-200 rounded-lg p-4 overflow-x-auto"><code v-html="highlightedPreview"></code></pre>
          <div v-else-if="previewLoading" class="flex items-center justify-center h-full text-base-content/50 text-sm">
            <SvgIcon name="clock" size="14" class="mr-2" /> 生成中...
          </div>
          <div v-else class="flex items-center justify-center h-full text-base-content/50 text-sm">点击「预览」生成配置</div>
        </div>
        <div class="flex items-center justify-end gap-2 px-6 py-4 border-t border-base-content/10 shrink-0">
          <button @click="copyPreview" v-if="previewContent" class="btn btn-ghost btn-sm gap-1">
            <SvgIcon name="clipboard" size="14" /> 复制
          </button>
          <button @click="showPreview = false" class="btn btn-primary btn-sm">关闭</button>
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
import hljs from 'highlight.js/lib/core'
import nginxLang from 'highlight.js/lib/languages/nginx'
hljs.registerLanguage('nginx', nginxLang)

const props = defineProps<{ presetId: string }>()

const toast = useToast()
const loading = ref(false)
const showDialog = ref(false)
const editingServer = ref<any>(null)
const searchText = ref('')
const api = getTauriAPI()

// Host 转发下拉选项
const HEADER_HOST_OPTIONS = ['$host', '$http_host', '$proxy_host', '$server_name']

function getHeaderHostSelectValue(loc: any): string {
  if (!loc.headerHost || HEADER_HOST_OPTIONS.includes(loc.headerHost)) return loc.headerHost || '$host'
  return '__custom__'
}
function isCustomHeaderHost(val: string): boolean {
  return !!val && !HEADER_HOST_OPTIONS.includes(val)
}
function onHeaderHostSelect(loc: any, val: string) {
  if (val !== '__custom__') loc.headerHost = val
}

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
const locParamEntries = ref<Array<{name: string, value: string, templateId: string}>>([])
const locParamTemplates = ref<any[]>([])
let editingLocIndex = -1

// Server 额外参数
const showServerParamDialog = ref(false)
const serverParamEntries = ref<Array<{name: string, value: string, position: number, templateId: string}>>([])
const serverParamTemplates = ref<any[]>([])

// 编辑弹窗预览
const showPreview = ref(false)
const previewContent = ref('')
const previewLoading = ref(false)

// 列表页配置预览
const showListPreview = ref(false)
const listPreviewContent = ref('')
const listPreviewLoading = ref(false)

const highlightedPreview = computed(() => {
  if (!previewContent.value) return ''
  try {
    return hljs.highlight(previewContent.value, { language: 'nginx' }).value
  } catch {
    return escapeHtml(previewContent.value)
  }
})

function escapeHtml(text: string) {
  return text.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
}

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
      rootType: l.rootType || 'root',
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

    // 保存 locations
    for (const loc of locations.value) {
      loc.serverId = form.value.id
      if (loc._key && !loc.id) {
        // 新增
        const newLoc = {
          id: crypto.randomUUID(),
          serverId: form.value.id,
          enabled: loc.enabled !== false,
          path: loc.path || '',
          value: loc.value || '',
          rootPath: loc.rootPath || '',
          rootPage: loc.rootPage || '',
          rootType: loc.rootType || 'root',
          upstreamId: loc.upstreamId || '',
          upstreamPath: loc.upstreamPath || '',
          header: loc.header || false,
          headerHost: loc.headerHost || '',
          websocket: loc.websocket || false,
          cros: loc.cros || false,
          returnUrl: loc.returnUrl || '',
          returnPath: loc.returnPath || false,
          descr: loc.descr || '',
          sort: loc.sort ?? 0,
          locType: loc.locType ?? 0,
          paramJson: loc.paramJson || '',
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
    rootPage: '',
    rootType: 'root',
    upstreamId: '',
    upstreamPath: '',
    header: false,
    headerHost: '',
    websocket: false,
    cros: false,
    returnUrl: '',
    returnPath: false,
    sort: locations.value.length + 1,
    paramJson: '',
    _key: crypto.randomUUID(),
  })
}

async function openLocationParams(idx: number) {
  editingLocIndex = idx
  const loc = locations.value[idx]
  // Load templates for template selector
  try {
    const result = await api.getTemplatesByPreset(props.presetId)
    locParamTemplates.value = result?.data ?? result ?? []
  } catch {
    locParamTemplates.value = []
  }
  try {
    const parsed = loc.paramJson ? JSON.parse(loc.paramJson) : []
    locParamEntries.value = Array.isArray(parsed) ? parsed.map((p: any) => ({ name: p.name || '', value: p.value || '', templateId: p.templateId || '' })) : []
  } catch {
    locParamEntries.value = []
  }
  showLocParamDialog.value = true
}

function getTemplateName(templateId: string): string {
  const tpl = locParamTemplates.value.find(t => t.id === templateId)
  return tpl?.name || templateId
}

function getServerTemplateName(templateId: string): string {
  const tpl = serverParamTemplates.value.find(t => t.id === templateId)
  return tpl?.name || templateId
}

async function openServerParams() {
  // Load templates
  try {
    const result = await api.getTemplatesByPreset(props.presetId)
    serverParamTemplates.value = result?.data ?? result ?? []
  } catch {
    serverParamTemplates.value = []
  }
  try {
    const parsed = form.value.paramJson ? JSON.parse(form.value.paramJson) : []
    serverParamEntries.value = Array.isArray(parsed) ? parsed.map((p: any) => ({
      name: p.name || '',
      value: p.value || '',
      position: p.position ?? 0,
      templateId: p.templateId || '',
    })) : []
  } catch {
    serverParamEntries.value = []
  }
  showServerParamDialog.value = true
}

function saveServerParams() {
  form.value.paramJson = JSON.stringify(serverParamEntries.value.filter(e => e.name.trim() || e.templateId))
  showServerParamDialog.value = false
}

function saveLocationParams() {
  if (editingLocIndex >= 0 && editingLocIndex < locations.value.length) {
    locations.value[editingLocIndex].paramJson = JSON.stringify(locParamEntries.value.filter(e => e.name.trim() || e.templateId))
  }
  showLocParamDialog.value = false
}

async function onPreview() {
  // 构建完整的 server 对象（同 onSave 的逻辑）
  const serverData = {
    id: form.value.id || crypto.randomUUID(),
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
    enabled: true,
    sort: form.value.sort ?? 0,
    paramJson: form.value.paramJson || '',
    createdAt: form.value.createdAt || new Date().toISOString(),
    updatedAt: new Date().toISOString(),
  }

  // 构建 locations（新增标记用 _key）
  const locData = locations.value.map((loc: any) => ({
    id: loc.id || crypto.randomUUID(),
    serverId: form.value.id || '',
    enabled: loc.enabled !== false,
    path: loc.path || '',
    value: loc.value || '',
    rootPath: loc.rootPath || '',
    rootPage: loc.rootPage || '',
    rootType: loc.rootType || 'root',
    upstreamId: loc.upstreamId || '',
    upstreamPath: loc.upstreamPath || '',
    header: loc.header || false,
    headerHost: loc.headerHost || '',
    websocket: loc.websocket || false,
    cros: loc.cros || false,
    returnUrl: loc.returnUrl || '',
    returnPath: loc.returnPath || false,
    descr: loc.descr || '',
    sort: loc.sort ?? 0,
    locType: loc.locType ?? 0,
    paramJson: loc.paramJson || '',
    createdAt: loc.createdAt || new Date().toISOString(),
  }))

  showPreview.value = true
  previewContent.value = ''
  previewLoading.value = true
  try {
    const result = await api.previewNginxServer(props.presetId, serverData, locData)
    const content = result?.data ?? result
    previewContent.value = typeof content === 'string' ? content : JSON.stringify(content, null, 2)
  } catch (err: any) {
    previewContent.value = '生成预览失败: ' + (err?.message || err)
  } finally {
    previewLoading.value = false
  }
}

function copyPreview() {
  if (!previewContent.value) return
  navigator.clipboard.writeText(previewContent.value)
    .then(() => toast.success('已复制到剪贴板'))
    .catch(() => toast.error('复制失败'))
}

const listHighlightedPreview = computed(() => {
  if (!listPreviewContent.value) return ''
  try {
    return hljs.highlight(listPreviewContent.value, { language: 'nginx' }).value
  } catch {
    return listPreviewContent.value.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
  }
})

async function onRowPreview(svr: any) {
  showListPreview.value = true
  listPreviewContent.value = ''
  listPreviewLoading.value = true
  try {
    // Load locations for this server
    const locResult = await api.getLocationsByServer(svr.id)
    const locationsData = locResult?.data ?? locResult ?? []

    const result = await api.previewNginxServer(props.presetId, svr, locationsData)
    const content = result?.data ?? result
    listPreviewContent.value = typeof content === 'string' ? content : JSON.stringify(content, null, 2)
  } catch (err: any) {
    listPreviewContent.value = '生成预览失败: ' + (err?.message || err)
  } finally {
    listPreviewLoading.value = false
  }
}

function copyListPreview() {
  if (!listPreviewContent.value) return
  navigator.clipboard.writeText(listPreviewContent.value)
    .then(() => toast.success('已复制到剪贴板'))
    .catch(() => toast.error('复制失败'))
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

<style scoped>
/* highlight.js token colors — Catppuccin Mocha */
.nginx-highlight :deep(.hljs-keyword) { color: #cba6f7; font-weight: 500; }
.nginx-highlight :deep(.hljs-attr) { color: #89b4fa; }
.nginx-highlight :deep(.hljs-string) { color: #a6e3a1; }
.nginx-highlight :deep(.hljs-number) { color: #fab387; }
.nginx-highlight :deep(.hljs-comment) { color: #6c7086; font-style: italic; }
.nginx-highlight :deep(.hljs-variable) { color: #f38ba8; }
.nginx-highlight :deep(.hljs-title) { color: #f9e2af; }
.nginx-highlight :deep(.hljs-literal) { color: #fab387; }
.nginx-highlight :deep(.hljs-built_in) { color: #a6e3a1; }
.nginx-highlight :deep(.hljs-section) { color: #89b4fa; }
.nginx-highlight code { font-family: inherit; background: transparent; padding: 0; }
</style>

<template>
  <div class="flex-1 overflow-y-auto bg-base-200">
    <div class="max-w-[1200px] mx-auto px-8 py-8">
      <!-- Header -->
      <div class="flex items-center justify-between mb-6">
        <div>
          <h3 class="m-0 text-xl font-bold text-base-content flex items-center gap-2">
            <SvgIcon name="rocket" :size="20" /> {{ editing ? '编辑部署配置' : '新建部署配置' }}
          </h3>
          <p class="m-0 mt-1 text-sm text-base-content/60">{{ editing ? '按步骤修改；多环境、健康检查等高级能力在底部「高级设置」中可继续配置' : '按步骤快速创建；多环境、健康检查等高级能力可在创建后继续配置' }}</p>
        </div>
        <div class="flex items-center gap-2">
          <button class="btn btn-ghost btn-sm" @click="emit('cancel')">取消</button>
        </div>
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
          <div class="grid grid-cols-3 gap-4">
            <div class="col-span-2">
              <label class="block mb-1 text-xs font-medium text-base-content/60 uppercase tracking-wider">Git 仓库 <span class="text-error normal-case tracking-normal">*</span></label>
              <select v-model="draft.gitRepoId" class="select select-bordered w-full bg-base-200 text-sm" @change="onRepoChange">
                <option value="">选择 Git 仓库...</option>
                <option v-for="repo in gitRepos" :key="repo.id" :value="repo.id">{{ repo.name }} — {{ repo.path }}</option>
              </select>
            </div>
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
          <template v-if="draft.buildTool === 'maven'">
            <div class="grid grid-cols-2 gap-4">
              <div>
                <label class="block mb-1 text-xs font-medium text-base-content/60 uppercase tracking-wider">Maven 路径</label>
                <input v-model="draft.mavenHome" class="input input-bordered w-full bg-base-200 text-sm" placeholder="自动检测 / 如 /opt/homebrew/opt/maven" />
              </div>
              <div>
                <label class="block mb-1 text-xs font-medium text-base-content/60 uppercase tracking-wider">JDK 路径</label>
                <input v-model="draft.javaHome" class="input input-bordered w-full bg-base-200 text-sm" placeholder="自动检测 / 如 /opt/homebrew/opt/openjdk" />
              </div>
              <div>
                <label class="block mb-1 text-xs font-medium text-base-content/60 uppercase tracking-wider">settings.xml</label>
                <input v-model="draft.mavenSettings" class="input input-bordered w-full bg-base-200 text-sm" placeholder="~/.m2/settings.xml（可留空）" />
              </div>
              <div>
                <label class="block mb-1 text-xs font-medium text-base-content/60 uppercase tracking-wider">重启脚本</label>
                <input v-model="draft.restartScript" class="input input-bordered w-full bg-base-200 text-sm" placeholder="./restart.sh" />
              </div>
              <div>
                <label class="block mb-1 text-xs font-medium text-base-content/60 uppercase tracking-wider">Maven Profile</label>
                <input v-model="draft.mavenProfile" class="input input-bordered w-full bg-base-200 text-sm" placeholder="prod" />
              </div>
            </div>
          </template>
          <div v-else-if="draft.buildTool === 'cargo'" class="grid grid-cols-2 gap-4">
            <div>
              <label class="block mb-1 text-xs font-medium text-base-content/60 uppercase tracking-wider">构建命令</label>
              <input v-model="draft.buildCommand" class="input input-bordered w-full bg-base-200 text-sm font-mono" placeholder="cargo build --release --features xxx" />
            </div>
          </div>
          <div v-else-if="['npm', 'pnpm', 'yarn'].includes(draft.buildTool)">
            <div class="grid grid-cols-2 gap-4">
              <div>
                <label class="block mb-1 text-xs font-medium text-base-content/60 uppercase tracking-wider">{{ draft.buildTool }} 路径</label>
                <input v-model="draft.nodeHome" class="input input-bordered w-full bg-base-200 text-sm" placeholder="自动检测 / 如 ~/.nvm/versions/node/v20.x" />
              </div>
              <div>
                <label class="block mb-1 text-xs font-medium text-base-content/60 uppercase tracking-wider">构建脚本</label>
                <select v-model="draft.npmScript" class="select select-bordered w-full bg-base-200 text-sm">
                  <!-- 当前值不在候选列表时兜底显示，避免 select 空白 -->
                  <option v-if="draft.npmScript && !npmScriptOptions.includes(draft.npmScript) && draft.npmScript !== 'custom'" :value="draft.npmScript">{{ draft.npmScript }}（当前配置）</option>
                  <option v-for="s in npmScriptOptions" :key="s" :value="s">{{ s }}</option>
                  <option value="custom">自定义...</option>
                </select>
              </div>
            </div>
            <input v-if="draft.npmScript === 'custom'" v-model="draft.npmCustomScript" class="input input-bordered w-full bg-base-200 text-sm mt-2" placeholder="脚本名称" />
          </div>

          <!-- 部署模式：共享组件（单体/多模块）+ Jar/Lib 分离开关；仅多模块项目显示 -->
          <div v-if="isMultiModule" class="flex flex-col gap-3">
            <DeployModeSelector
              v-model="monolithMode"
              v-model:libSeparate="libSeparate"
              :deploy-path="draft.deployPath"
            />

            <!-- 多模块部署：模块勾选区 -->
            <div v-if="!monolithMode" class="border border-primary/20 rounded-xl overflow-hidden">
              <div class="flex items-center gap-2 px-4 py-3 bg-primary/5 border-b border-primary/10">
                <SvgIcon name="layers" :size="15" class="text-primary flex-shrink-0" />
                <span class="text-sm font-semibold text-base-content">部署模块</span>
                <span v-if="scanningProj" class="ml-1 loading loading-spinner loading-xs text-primary" />
                <span v-else class="ml-1 text-xs text-base-content/60">识别到 {{ modules.length }} 个模块{{ selectedModules.length ? `，已勾选 ${selectedModules.length} 个` : '' }}</span>
                <span class="ml-auto text-[10px] text-base-content/50">每个模块独立构建并部署到独立远程目录</span>
              </div>
              <div class="p-2 max-h-48 overflow-y-auto flex flex-col">
                <template v-for="(m, idx) in modules" :key="m.moduleName">
                  <div class="flex flex-col px-3 py-2 rounded-lg hover:bg-base-200/60 transition-colors">
                    <div class="flex items-center gap-2.5">
                      <input v-model="m.checked" type="checkbox" class="checkbox checkbox-primary checkbox-sm" />
                      <span class="text-sm font-medium text-base-content">{{ m.moduleName }}</span>
                      <span class="ml-auto text-xs text-base-content/40 font-mono">{{ m.modulePath }}</span>
                      <button @click="toggleModuleExpand(idx)" class="btn btn-ghost btn-xs text-base-content/50" title="远程路径">
                        <SvgIcon name="settings" size="12" />
                      </button>
                    </div>
                    <div v-show="expandedModuleIdx === idx" class="mt-2 pl-8 pr-2 pb-1">
              <label class="block mb-1 text-[11px] font-medium text-base-content/50 uppercase tracking-wider">远程子目录（相对全局目录）</label>
                      <input v-model="m.deployPath" class="input input-bordered w-full bg-base-200 text-xs font-mono" :placeholder="`相对全局目录的子路径，如 pre-corp（默认沿用全局 ${draft.deployPath || '~/apphome'}）`" />
                    </div>
                  </div>
                </template>
                <div v-if="!selectedModules.length" class="px-3 py-2 text-xs text-amber-600">
                  未勾选任何模块，将不部署子模块
                </div>
              </div>
            </div>
            <!-- 单体部署：选择主模块（产物 jar 所在模块）+ 构建目录 -->
            <div v-else class="flex flex-col gap-3 rounded-xl border border-primary/20 overflow-hidden">
              <div class="px-3 py-2.5 bg-base-200/60 text-xs text-base-content/60">
                单体部署：整体构建产出单个 jar。主模块常在子目录（如预付卡 <code class="bg-base-100 px-1 rounded">SRC/b2b2c/seller-api</code>），选错目录将拿不到 jar 产物，请务必选择产物所在模块
              </div>
              <div class="px-3 pb-3 flex flex-col gap-2.5">
                <label class="block mb-0.5 text-xs font-medium text-base-content/60 uppercase tracking-wider">主模块 <span class="text-error normal-case tracking-normal">*</span></label>
                <select v-model="draft.parentBuildPath" class="select select-bordered w-full bg-base-200 text-sm">
                  <option value="">项目根目录（主模块在根目录时）</option>
                  <option v-for="m in modules" :key="m.modulePath" :value="m.modulePath">{{ m.moduleName }} — {{ m.modulePath }}</option>
                </select>
                <span class="text-xs text-base-content/40">或在下方手动填写主模块相对路径（收集其 target 下的 jar 产物）</span>
                <input v-model="draft.parentBuildPath" class="input input-bordered w-full bg-base-200 text-sm font-mono" placeholder="如 ./SRC/b2b2c/seller-api" />
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
          <div class="grid grid-cols-3 gap-4">
            <div class="col-span-2">
              <label class="block mb-1 text-xs font-medium text-base-content/60 uppercase tracking-wider">目标服务器 <span class="text-error normal-case tracking-normal">*</span></label>
              <GroupedServerSelector :servers="servers" :groups="serverGroups" v-model="selectedServerIds" mode="multi" />
              <span class="block text-xs text-base-content/60 mt-1.5">已选 {{ selectedServerIds.length }} 台，部署时按顺序逐台上传</span>
            </div>
            <div>
              <label class="block mb-1 text-xs font-medium text-base-content/60 uppercase tracking-wider">部署路径 <span class="text-error normal-case tracking-normal">*</span></label>
              <input v-model="draft.deployPath" class="input input-bordered w-full bg-base-200 text-sm font-mono" :placeholder="suggestedDeployPath" />
              <span class="block text-xs text-base-content/60 mt-1">{{ draft.buildTool === 'maven' ? '如 /opt/apphome' : '如 /home/nginxWebUI/ui' }}；多模块各子目录在其下增量拼接</span>
            </div>
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

          <!-- 高级设置：多环境 / 部署保障（涵盖旧分组表单字段） -->
          <div class="border border-base-content/10 rounded-xl overflow-hidden">
            <div class="flex items-center gap-2 px-4 py-3 cursor-pointer select-none hover:bg-base-200/50 transition-colors" @click="showAdvanced = !showAdvanced">
              <SvgIcon name="chevronDown" :size="15" class="transition-transform duration-200 text-base-content/60 flex-shrink-0" :class="{ '-rotate-90': !showAdvanced }" />
              <SvgIcon name="sliders" :size="15" class="text-primary flex-shrink-0" />
              <span class="text-sm font-semibold text-base-content">高级设置</span>
              <span class="ml-auto text-[10px] text-base-content/50" v-if="!showAdvanced">{{ draft.environments.length }} 个多环境{{ draft.healthCheckUrl ? ' · 健康检查' : '' }} {{ draft.incrementalUpload ? ' · 增量上传' : '' }}</span>
            </div>

            <div v-show="showAdvanced" class="border-t border-base-content/10 px-4 py-4 flex flex-col gap-5">
              <!-- 多环境部署 -->
              <div>
                <div class="flex items-center justify-between mb-2">
                  <span class="text-xs font-semibold text-base-content/70 uppercase tracking-wider">多环境部署</span>
                  <button class="btn btn-ghost btn-xs" @click="addEnv"><SvgIcon name="plus" :size="12" /> 添加环境</button>
                </div>
                <p class="m-0 mb-2 text-xs text-base-content/50">一套构建 + 多套部署目标（路径 / 环境变量 / 健康检查），不配置则仅使用全局部署路径。</p>
                <div v-for="(env, i) in draft.environments" :key="i" class="border border-base-content/10 rounded-xl overflow-hidden mb-2">
                  <div class="flex items-center gap-2 px-3 py-2.5 bg-base-200/50 border-b border-base-content/5">
                    <input v-model="env.name" class="input input-bordered bg-base-100 text-sm flex-1 min-w-0" placeholder="环境名，如 测试环境" />
                    <button @click="removeEnv(i)" class="btn btn-ghost btn-sm btn-square text-error hover:bg-error/10" title="删除环境"><SvgIcon name="x" :size="13" /></button>
                  </div>
                  <div class="grid grid-cols-2 gap-3 px-3 py-3">
                    <div>
                      <label class="block mb-1 text-xs font-medium text-base-content/60 uppercase tracking-wider">部署路径</label>
                      <input v-model="env.deployPath" class="input input-bordered w-full bg-base-200 text-sm font-mono" placeholder="/opt/app-test" />
                    </div>
                    <div>
                      <label class="block mb-1 text-xs font-medium text-base-content/60 uppercase tracking-wider">健康检查 URL</label>
                      <input v-model="env.healthCheckUrl" class="input input-bordered w-full bg-base-200 text-sm font-mono" placeholder="http://test.example.com/health（可留空）" />
                    </div>
                    <div>
                      <label class="block mb-1 text-xs font-medium text-base-content/60 uppercase tracking-wider">健康检查超时（秒）</label>
                      <input v-model.number="env.healthCheckTimeout" type="number" min="1" class="input input-bordered w-full bg-base-200 text-sm" />
                    </div>
                    <div>
                      <label class="block mb-1 text-xs font-medium text-base-content/60 uppercase tracking-wider">失败重试次数</label>
                      <input v-model.number="env.healthCheckRetries" type="number" min="1" class="input input-bordered w-full bg-base-200 text-sm" />
                    </div>
                  </div>
                  <div class="px-3 pb-3">
                    <label class="block mb-1 text-xs font-medium text-base-content/60 uppercase tracking-wider">环境变量（每行 KEY=VALUE）</label>
                    <textarea v-model="env.envVars" class="textarea textarea-bordered w-full bg-base-200 text-xs font-mono resize-y leading-relaxed" rows="2" placeholder="NODE_ENV=production&#10;VITE_API_BASE=https://api.example.com" />
                  </div>
                </div>
                <div v-if="draft.environments.length" class="px-3 py-2 rounded-lg bg-base-200/60 text-xs text-base-content/50">多环境会覆盖全局部署路径；服务器沿用上一步所选的目标服务器，可在创建后于「多环境部署」分组调整为每环境独立服务器。</div>
              </div>

              <!-- 部署保障 -->
              <div class="border-t border-base-content/10 pt-4">
                <div class="text-xs font-semibold text-base-content/70 uppercase tracking-wider mb-2">部署保障</div>
                <div class="flex flex-wrap gap-4">
                  <label class="flex items-center gap-2 select-none cursor-pointer text-sm">
                    <input v-model="draft.incrementalUpload" type="checkbox" class="toggle toggle-primary toggle-sm" />
                    <span>增量上传</span>
                  </label>
                  <label class="flex items-center gap-2 select-none cursor-pointer text-sm">
                    <input v-model="draft.requiresApproval" type="checkbox" class="toggle toggle-warning toggle-sm" />
                    <span>部署需审核</span>
                  </label>
                </div>
                <div class="grid grid-cols-3 gap-3 mt-3">
                  <div class="col-span-1">
                    <label class="block mb-1 text-xs font-medium text-base-content/60 uppercase tracking-wider">健康检查 URL（全局）</label>
                    <input v-model="draft.healthCheckUrl" class="input input-bordered w-full bg-base-200 text-sm font-mono" placeholder="留空跳过健康检查" />
                  </div>
                  <div>
                    <label class="block mb-1 text-xs font-medium text-base-content/60 uppercase tracking-wider">超时（秒）</label>
                    <input v-model.number="draft.healthCheckTimeout" type="number" min="1" class="input input-bordered w-full bg-base-200 text-sm" title="单次探测超时（秒）" />
                  </div>
                  <div>
                    <label class="block mb-1 text-xs font-medium text-base-content/60 uppercase tracking-wider">重试次数</label>
                    <input v-model.number="draft.healthCheckRetries" type="number" min="1" class="input input-bordered w-full bg-base-200 text-sm" title="失败重试次数" />
                  </div>
                </div>
              </div>
            </div>
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
            {{ creating ? (editing ? '保存中...' : '创建中...') : (editing ? '保存修改' : '创建配置') }}
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
import DeployModeSelector from './DeployModeSelector.vue'
import { getTauriAPI } from '../../utils/tauri-api'
import type { Server } from '../../types'

interface GitRepoEntry { id: string; name: string; path?: string; branch?: string }
interface ServerGroupEntry { id: string; name: string; color: string; parentId: string | null }
interface BuildToolOption { key: string; name: string; icon: string; version?: string; available: boolean }
interface ModuleItem { moduleName: string; modulePath: string; checked: boolean; deployPath?: string; src?: Record<string, unknown> }

const props = defineProps<{
  gitRepos: GitRepoEntry[]
  groups: string[]
  servers: Server[]
  serverGroups: ServerGroupEntry[]
  buildTools: BuildToolOption[]
  /** 编辑模式：传入已存在配置预填，complete 时 payload 携带 id 走更新逻辑 */
  initial?: Record<string, unknown> | null
}>()

const emit = defineEmits<{
  complete: [payload: Record<string, unknown>]
  cancel: []
  /** 编辑模式下跳转原「高级设置」分组表单（改高级字段） */
  openAdvanced: []
}>()

// 是否编辑模式（存在 id 即编辑）
const editing = computed(() => !!props.initial?.id)

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
  // ── 高级配置（涵盖旧分组表单全部字段）──
  repoUrl: '',
  mavenHome: '',
  javaHome: '',
  nodeHome: '',
  mavenSettings: '',
  parentBuildPath: '',
  buildCommand: '',
  incrementalUpload: true,
  requiresApproval: false,
  healthCheckUrl: '',
  healthCheckTimeout: 30,
  healthCheckRetries: 2,
  // 多环境部署：一套构建 + 多套部署目标
  environments: [] as {
    name: string; deployPath: string; servers: { serverId: string; label?: string; deployDir: string }[];
    envVars: string; healthCheckUrl: string; healthCheckTimeout: number; healthCheckRetries: number;
  }[],
})
// 高级设置折叠面板是否展开
const showAdvanced = ref(false)

// 多环境：添加 / 删除一个环境（服务器沿用全局勾选，回落为空以继承部署目标）
function addEnv() {
  draft.environments.push({
    name: `环境 ${draft.environments.length + 1}`,
    deployPath: '',
    servers: [],
    envVars: '',
    healthCheckUrl: '',
    healthCheckTimeout: 30,
    healthCheckRetries: 2,
  })
}
function removeEnv(i: number) { draft.environments.splice(i, 1); }

// 部署模式（共享组件 v-model）：单体部署（parentBuildMode=true，打包单jar）/ 多模块部署（false，逐模块独立构建）
const monolithMode = ref(true)
// 是否启用 jar 与 lib 分离（两种部署模式均可选，是模式下的能力项不是独立模式）
const libSeparate = ref(true)

// npm 构建脚本下拉选项：项目 package.json scripts 中以 build 开头的脚本（build/build:h5/build:prod...）
const npmScriptOptions = computed<string[]>(() => {
  const names = (scanned.value.npmScripts as string[] | undefined) || []
  const builds = names.filter(n => n === 'build' || n.startsWith('build'))
  return builds.length ? builds : ['build']
})

// 多模块：扫描识别出 moduleNames 后生成的勾选列表；无多模块则为空数组
const modules = ref<ModuleItem[]>([])

// 模块远程路径展开控制
const expandedModuleIdx = ref<number | null>(null)
function toggleModuleExpand(idx: number) {
  expandedModuleIdx.value = expandedModuleIdx.value === idx ? null : idx
}

// ── 编辑模式：从已有配置预填向导 ──
function prefillFromInitial() {
  const c = props.initial
  if (!c) {return;}
  draft.name = (c.name as string) || ''
  draft.gitRepoId = (c.gitRepoId as string) || ''
  draft.groupName = (c.groupName as string) || '未分组'
  draft.deployBranch = (c.deployBranch as string) || 'main'
  draft.buildTool = (c.buildTool as string) || ''
  draft.mavenProfile = (c.mavenProfile as string) || 'prod'
  draft.npmScript = (c.npmScript as string) || 'build'
  draft.npmCustomScript = (c.npmCustomScript as string) || ''
  // 存量配置的构建脚本可能存在模块行 buildCommand（"npm run build:h5:staging"），
  // 配置级为默认值（npmScript='build' 且无自定义脚本）时回填真实脚本，避免编辑保存后命令退化
  if (draft.npmScript === 'build' && !draft.npmCustomScript) {
    const mods = Array.isArray(c.modules) ? (c.modules as { buildCommand?: string; enabled?: boolean }[]) : []
    const modCmd = mods.find(m => m.enabled !== false && m.buildCommand)?.buildCommand?.trim()
    if (modCmd) {
      // 剥包管理器前缀并截掉尾部参数（如 "pnpm build --mode test" → "build"）
      const script = modCmd
        .replace(/^(npm run|npx|pnpm run|pnpm|yarn|npm)\s+/, '')
        .split(/\s+/)[0]
        .trim()
      if (script && script !== 'build') { draft.npmScript = script }
    }
  }
  draft.restartScript = (c.restartScript as string) || './restart.sh'
  draft.deployPath = (c.deployPath as string) || ''
  draft.localPath = (c.localPath as string) || ''
  // ── 高级配置回填（涵盖旧分组表单全部字段）──
  draft.repoUrl = (c.repoUrl as string) || ''
  draft.mavenHome = (c.mavenHome as string) || ''
  draft.javaHome = (c.javaHome as string) || ''
  draft.nodeHome = (c.nodeHome as string) || ''
  draft.mavenSettings = (c.mavenSettings as string) || ''
  draft.parentBuildPath = (c.parentBuildPath as string) || ''
  draft.buildCommand = (c.buildCommand as string) || ''
  draft.incrementalUpload = (c.incrementalUpload as boolean) ?? true
  draft.requiresApproval = !!c.requiresApproval
  draft.healthCheckUrl = (c.healthCheckUrl as string) || ''
  draft.healthCheckTimeout = (c.healthCheckTimeout as number) ?? 30
  draft.healthCheckRetries = (c.healthCheckRetries as number) ?? 2
  // 多环境：config.environments 为 JSON 字符串或数组
  const envs = c.environments
  if (typeof envs === 'string' && envs) {
    try { draft.environments = JSON.parse(envs); } catch {/* 忽略 */}
  } else if (Array.isArray(envs)) {
    draft.environments = envs;
  }
  // 部署模式：parentBuildMode=true 单体；false 多模块
  monolithMode.value = (c.parentBuildMode as boolean) ?? true
  // Jar/Lib 分离：非多模块项目不可见，保留原值
  libSeparate.value = (c.libSeparate as boolean) ?? true
  // 服务器：servers 为 JSON 字符串
  if (typeof c.servers === 'string' && c.servers) {
    try {
      const parsed = JSON.parse(c.servers)
      if (Array.isArray(parsed)) { selectedServerIds.value = parsed.map((s: { serverId?: string }) => s.serverId).filter((v: string | undefined): v is string => !!v); }
    } catch {/* 忽略 */}
  } else if (Array.isArray(c.servers)) {
    selectedServerIds.value = (c.servers as { serverId?: string }[]).map(s => s.serverId).filter((v: string | undefined): v is string => !!v);
  }
  // 模块：已有模块作为勾选列表（默认按 enabled 勾选）
  const initialMods = Array.isArray(c.modules) ? (c.modules as { moduleName?: string; modulePath?: string; enabled?: boolean }[]) : []
  if (initialMods.length) {
    modules.value = initialMods.map(m => ({
      moduleName: m.moduleName || '',
      modulePath: m.modulePath || m.moduleName || '',
      checked: m.enabled !== false,
      src: { ...m },
    }))
  }
}
// 编辑模式进入时预填
watch(() => props.initial, (v) => { if (v?.id) { prefillFromInitial(); } }, { immediate: true })

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
    // 对话框直接定位到当前仓库目录，避免从头翻目录
    const repo = props.gitRepos.find(r => r.id === draft.gitRepoId)
    const result = await getTauriAPI().showOpenDialogForDirs({
      defaultPath: draft.localPath || repo?.path || '',
    })
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
          // 编辑模式且已回填已有模块时：保留已保存模块（含 src 元数据与勾选态），避免扫描覆盖
          if (editing.value && modules.value.length) {
            scanned.value.isMultiModule = true;
          } else {
            modules.value = names.map(n => ({ moduleName: n, modulePath: n, checked: true }));
            scanned.value.isMultiModule = true;
          }
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
    rows.push({ label: '部署模式', value: monolithMode.value ? '单体部署' : '多模块部署' });
    rows.push({ label: 'Jar/Lib 分离', value: libSeparate.value ? '是' : '否' });
    if (!monolithMode.value) {
      rows.push({ label: '部署模块', value: moduleSummary.value || '—' });
    }
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
    // 单体部署（monolithMode）：父模块统一构建，模块勾选列表不落库（产物为单个 jar）
    // 多模块部署：逐模块独立构建，各自部署独立目录
    // parentBuildPath 交由父组件决定（取 git 仓库根目录，指向父 POM）
    const monolith = monolithMode.value
    // 编辑模式：始终复用已有模块（含 src 原字段，仅调 enabled），避免单体模式下误删已有模块
    // 新建模式：单体部署不落模块列表；多模块仅落勾选出的模块
    const modPayload = editing
      ? modules.value.map(m => ({
          ...(m.src as Record<string, unknown>),
          moduleName: m.moduleName,
          modulePath: m.modulePath,
          enabled: m.checked,
          deployPath: m.deployPath || (m.src as Record<string, unknown> | undefined)?.deployPath || '',
        }))
      : (monolith ? [] : selectedModules.value.map(m => ({
          moduleName: m.moduleName,
          modulePath: m.modulePath,
          enabled: m.checked,
          deployPath: m.deployPath || '',
        })))
    emit('complete', {
      ...draft,
      id: (props.initial as Record<string, unknown> | null)?.id ?? null,
      servers: serverEntries,
      parentBuildMode: monolith,
      parentBuildPath: draft.parentBuildPath || '',
      // 非多模块项目不可见该开关，fallback 为 false（避免误开启分离上传）
      libSeparate: isMultiModule.value && libSeparate.value,
      modules: modPayload,
    })
  } finally {
    creating.value = false;
  }
}
</script>

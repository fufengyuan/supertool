<template>
  <div class="flex flex-col h-full overflow-hidden">
    <header class="shrink-0 flex items-center gap-3 px-5 py-3 border-b border-base-content/10 bg-base-100">
      <span class="shrink-0 w-9 h-9 rounded-lg bg-primary/15 text-primary flex items-center justify-center">
        <SvgIcon name="bot" size="18" />
      </span>
      <div class="min-w-0 flex-1">
        <h2 class="text-sm font-bold text-base-content leading-tight">AI 配置助手</h2>
        <p class="text-[11px] text-base-content/50 leading-tight truncate">
          读懂你已配的参数，告诉你某个字段该填什么、这次部署为什么失败，并把改动整理成待确认的提案
        </p>
      </div>
      <button class="btn btn-ghost btn-xs shrink-0 gap-1" @click="showRules = !showRules">
        <SvgIcon name="shield" size="12" /> 权限边界
      </button>
      <button class="btn btn-ghost btn-xs shrink-0 gap-1" title="在桌面角落挂一个悬浮助手" @click="openFloating">
        <SvgIcon name="externalLink" size="12" /> 悬浮唤起
      </button>
    </header>

    <div class="flex-1 min-h-0 flex">
      <main class="flex-1 min-w-0 flex flex-col min-h-0">
        <AssistantChat />
      </main>

      <aside
        v-if="showRules"
        class="w-[300px] shrink-0 border-l border-base-content/10 bg-base-100 overflow-y-auto p-4 flex flex-col gap-4"
      >
        <section>
          <h3 class="text-xs font-bold text-base-content mb-2 flex items-center gap-1.5">
            <SvgIcon name="check" size="13" class="text-success" /> 它会做什么
          </h3>
          <ul class="text-[11px] text-base-content/70 leading-relaxed m-0 pl-4 flex flex-col gap-1">
            <li>读取你已配置的服务器、数据库连接、部署配置与部署日志</li>
            <li>查找本机项目目录、核对某个构建/产物路径是否存在（只取路径与元信息）</li>
            <li>按项目踩过的坑做字段级体检（构建目录 / 产物目录 / lib 分离 / 健康检查…）</li>
            <li>解释每个字段的含义，给出下一步操作并带你到对应页面</li>
            <li>把要改的内容整理成「变更提案」，列出每个取值和理由</li>
          </ul>
        </section>

        <section>
          <h3 class="text-xs font-bold text-base-content mb-2 flex items-center gap-1.5">
            <SvgIcon name="lock" size="13" class="text-warning" /> 它不会做什么
          </h3>
          <ul class="text-[11px] text-base-content/70 leading-relaxed m-0 pl-4 flex flex-col gap-1">
            <li><b>看不到任何密码/密钥</b>：返回值里这类字段一律是 [已隐藏]</li>
            <li><b>不能直接改配置</b>：所有写入都必须你在提案卡片上点「确认应用」</li>
            <li><b>读不到任何文件内容</b>：路径工具只给存在性/类型/大小，磁盘写、命令执行、SQL 一律没有</li>
            <li><b>凭据目录不可访问</b>：<code>.ssh</code>、钥匙串、<code>.aws</code> 等既搜不到也不能核对</li>
            <li>需要凭据时（如新建服务器），由你在卡片上亲手填写，不经过对话</li>
          </ul>
        </section>

        <section>
          <h3 class="text-xs font-bold text-base-content mb-2">怎么用起来最省事</h3>
          <ol class="text-[11px] text-base-content/70 leading-relaxed m-0 pl-4 flex flex-col gap-1">
            <li>先在 <b>设置 → AI 模型</b> 配好提供商（协议 / 接口地址 / 模型 ID / 上下文窗口）</li>
            <li>直接说目标，例如「给商城后端加一台测试环境服务器」</li>
            <li>部署失败时把配置名告诉它，它会读日志并给出改哪个字段</li>
            <li>提案逐条确认；改坏了还能回到对应页面手工调整</li>
          </ol>
        </section>

        <section v-if="capabilities.length">
          <h3 class="text-xs font-bold text-base-content mb-2">已装载的工具（{{ capabilities.length }}）</h3>
          <div class="flex flex-col gap-1.5">
            <div v-for="c in capabilities" :key="c.name" class="text-[10px] leading-snug">
              <span class="font-mono text-primary/80">{{ c.name }}</span>
              <div class="text-base-content/55">{{ firstLine(c.description) }}</div>
            </div>
          </div>
        </section>
      </aside>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import SvgIcon from '../../components/ui/SvgIcon.vue'
import AssistantChat from './components/AssistantChat.vue'
import { useAssistantChat } from '../../composables/useAssistantChat'
import { getTauriAPI } from '../../utils/tauri-api'
import { useToast } from '../../composables/useToast'

const showRules = ref(true)
const toast = useToast()
// 页面自身只需要能力清单（侧栏展示），会话与事件订阅都在 AssistantChat 内部，
// 这里不再 start()，避免同一个窗口里挂两份 assistant-event 监听
const { capabilities, refreshState } = useAssistantChat()

async function openFloating() {
  try {
    await (getTauriAPI() as any).openFloatingAssistant?.()
  } catch (e) {
    toast.error(`唤起失败：${String((e as Error)?.message || e)}`)
  }
}

function firstLine(text: string) {
  return text.split(/[。\n]/)[0] || text
}

onMounted(() => {
  void refreshState()
})
</script>

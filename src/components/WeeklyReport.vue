<template>
  <div class="weekly-report-container">
    <div class="report-header">
      <h2>📊 {{ $t('report.title') }}</h2>
      <div class="report-controls">
        <ReportConfig
          ref="reportConfigRef"
          :initial-range="selectedRange"
          :initial-start-date="startDate"
          :initial-end-date="endDate"
          @range-change="onRangeChange"
          @date-change="onCustomDateChange"
        />
        <ReportActions
          :generating="generating"
          :report-data="reportData"
          @generate="generateReport"
          @export-markdown="exportMarkdown"
          @export-word="exportWord"
        />
      </div>
    </div>

    <div class="report-tabs">
      <button class="tab-btn" :class="{ active: activeTab === 'current' }" @click="activeTab = 'current'">当前周报</button>
      <button class="tab-btn" :class="{ active: activeTab === 'history' }" @click="activeTab = 'history'; loadHistory()">历史周报</button>
    </div>

    <template v-if="activeTab === 'current'">
      <ReportContent :report-data="reportData" :get-project-name="getProjectName" :format-date="formatDate" />
    </template>

    <template v-if="activeTab === 'history'">
      <div class="history-list">
        <div v-if="historyLoading" class="loading-state">加载中...</div>
        <div v-else-if="historyReports.length === 0" class="empty-state">暂无历史周报</div>
        <div v-else class="history-items">
          <div v-for="report in historyReports" :key="report.id" class="history-item" :class="{ selected: selectedHistoryId === report.id }" @click="loadHistoryReport(report.id)">
            <div class="history-item-info">
              <span class="history-date">{{ formatDate(report.startDate) }} ~ {{ formatDate(report.endDate) }}</span>
              <span class="history-created">创建于 {{ formatDateTime(report.createdAt) }}</span>
            </div>
            <button class="history-load-btn" @click.stop="loadHistoryReport(report.id)">查看</button>
          </div>
        </div>
      </div>
      <div v-if="historyReportData" class="history-report-view">
        <div class="history-report-header">
          <h3>周报详情: {{ formatDate(historyReportData.startDate) }} ~ {{ formatDate(historyReportData.endDate) }}</h3>
          <button class="btn btn-ghost" @click="restoreFromHistory">恢复到当前视图</button>
        </div>
        <ReportContent :report-data="historyReportData" :get-project-name="getProjectName" :format-date="formatDate" />
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
// @ts-nocheck
import { ref, onMounted, onBeforeUnmount } from 'vue'
import { useI18n } from 'vue-i18n'
import { useToast } from '../composables/useToast'
import { useErrorHandler } from '../composables/useErrorHandler'
import { getTauriAPI } from '../utils/tauri-api'
import ReportConfig from './weekly/ReportConfig.vue'
import ReportContent from './weekly/ReportContent.vue'
import ReportActions from './weekly/ReportActions.vue'
import type { Project } from '../types'

const { t } = useI18n()
const toast = useToast()
const { handleError } = useErrorHandler()

const tauri = getTauriAPI()

const reportConfigRef = ref(null)
const selectedRange = ref('thisWeek')
const startDate = ref('')
const endDate = ref('')
const generating = ref(false)
const reportData = ref(null)
const projects = ref<Project[]>([])

const activeTab = ref('current')
const historyReports = ref<any[]>([])
const historyLoading = ref(false)
const selectedHistoryId = ref(null)
const historyReportData = ref(null)

const calculateDateRange = (range: string) => {
  const today = new Date()
  switch (range) {
    case 'thisWeek': {
      const thisMonday = new Date(today)
      thisMonday.setDate(today.getDate() - today.getDay() + (today.getDay() === 0 ? -6 : 1))
      thisMonday.setHours(0, 0, 0, 0)
      const thisSunday = new Date(thisMonday)
      thisSunday.setDate(thisMonday.getDate() + 6)
      thisSunday.setHours(23, 59, 59, 999)
      return { start: thisMonday, end: thisSunday }
    }
    case 'lastWeek': {
      const lastMonday = new Date(today)
      lastMonday.setDate(today.getDate() - today.getDay() + (today.getDay() === 0 ? -13 : -6))
      lastMonday.setHours(0, 0, 0, 0)
      const lastSunday = new Date(lastMonday)
      lastSunday.setDate(lastMonday.getDate() + 6)
      lastSunday.setHours(23, 59, 59, 999)
      return { start: lastMonday, end: lastSunday }
    }
    default:
      return { start: new Date(startDate.value), end: new Date(endDate.value) }
  }
}

const formatDate = (date: any) => {
  if (!date) return ''
  const d = new Date(date)
  return d.toLocaleDateString('zh-CN')
}

const formatDateTime = (date: any) => {
  if (!date) return ''
  const d = new Date(date)
  return d.toLocaleString('zh-CN')
}

const getProjectName = (projectId: string) => {
  const project = projects.value.find((p) => p.id === projectId)
  return project ? project.name : t('report.unassignedTasks')
}

const generateReport = async () => {
  generating.value = true
  try {
    projects.value = (await tauri.getProjects(true)) || []
    const range = calculateDateRange(selectedRange.value)
    const allTodos = (await tauri.getTodos()) || []

    const completedTasks = allTodos.filter(
      (todo) => todo.completed && new Date(todo.completedAt) >= range.start && new Date(todo.completedAt) <= range.end
    )
    const activeTasks = allTodos.filter(
      (todo) => !todo.completed && new Date(todo.updatedAt) >= range.start && new Date(todo.updatedAt) <= range.end
    )
    const nextWeekPlan = allTodos.filter(
      (todo) =>
        !todo.completed && todo.dueDate &&
        new Date(todo.dueDate) > range.end &&
        new Date(todo.dueDate) <= new Date(range.end.getTime() + 7 * 24 * 60 * 60 * 1000)
    )

    const projectStatsMap = new Map()
    const weeklyWorkMap = new Map()
    const nextWeekPlanMap = new Map()

    for (const task of completedTasks) {
      const projectId = task.projectId || 'unassigned'
      if (!weeklyWorkMap.has(projectId)) weeklyWorkMap.set(projectId, [])
      weeklyWorkMap.get(projectId).push(task)
      if (!projectStatsMap.has(projectId)) {
        projectStatsMap.set(projectId, { projectId, daysActive: new Set().add(new Date(task.completedAt).toDateString()) })
      } else {
        projectStatsMap.get(projectId).daysActive.add(new Date(task.completedAt).toDateString())
      }
    }
    for (const task of activeTasks) {
      const projectId = task.projectId || 'unassigned'
      if (!weeklyWorkMap.has(projectId)) weeklyWorkMap.set(projectId, [])
      weeklyWorkMap.get(projectId).push(task)
      if (!projectStatsMap.has(projectId)) {
        projectStatsMap.set(projectId, { projectId, daysActive: new Set().add(new Date(task.updatedAt).toDateString()) })
      } else {
        projectStatsMap.get(projectId).daysActive.add(new Date(task.updatedAt).toDateString())
      }
    }
    for (const task of nextWeekPlan) {
      const projectId = task.projectId || 'unassigned'
      if (!nextWeekPlanMap.has(projectId)) nextWeekPlanMap.set(projectId, [])
      nextWeekPlanMap.get(projectId).push(task)
    }

    const projectStats = Array.from(projectStatsMap.values()).map((stat: any) => ({ ...stat, daysActive: stat.daysActive.size }))

    reportData.value = {
      startDate: range.start,
      endDate: range.end,
      completedTasks,
      projects: projects.value,
      projectStats,
      weeklyWork: Object.fromEntries(weeklyWorkMap),
      nextWeekPlan: Object.fromEntries(nextWeekPlanMap),
      gitCommits: [],
    }

    await saveReportToDatabase(reportData.value)
  } catch (error) {
    handleError(error, { context: '生成周报', showToast: true })
  } finally {
    generating.value = false
  }
}

const saveReportToDatabase = async (data: any) => {
  try {
    const content = JSON.stringify(data, (key, val) => {
      if (val instanceof Date) return val.toISOString()
      return val
    })
    await tauri.saveWeeklyReport({
      weekStart: data.startDate instanceof Date ? data.startDate.toISOString() : data.startDate,
      weekEnd: data.endDate instanceof Date ? data.endDate.toISOString() : data.endDate,
      content,
    })
    toast.success('周报已自动保存')
  } catch (error) {
    console.error('Failed to auto-save weekly report:', error)
  }
}

const loadHistory = async () => {
  historyLoading.value = true
  try {
    const reports = await tauri.getWeeklyReports()
    historyReports.value = Array.isArray(reports) ? reports : []
  } catch (error) {
    handleError(error, { context: '加载历史周报', showToast: true })
  } finally {
    historyLoading.value = false
  }
}

const loadHistoryReport = async (id: any) => {
  try {
    const report = await tauri.getWeeklyReport(String(id))
    if (report) {
      selectedHistoryId.value = id
      const parsedData = typeof report.content === 'string' ? JSON.parse(report.content) : (report as any).data || report.content
      if (parsedData.startDate) parsedData.startDate = new Date(parsedData.startDate)
      if (parsedData.endDate) parsedData.endDate = new Date(parsedData.endDate)
      historyReportData.value = parsedData
    }
  } catch (error) {
    handleError(error, { context: '加载周报详情', showToast: true })
  }
}

const restoreFromHistory = () => {
  if (historyReportData.value) {
    reportData.value = { ...historyReportData.value }
    activeTab.value = 'current'
    toast.success('已从历史周报恢复')
  }
}

const onRangeChange = (range: string) => {
  selectedRange.value = range
  if (range !== 'custom') generateReport()
}

const onCustomDateChange = ({ start, end }: { start: string; end: string }) => {
  startDate.value = start
  endDate.value = end
  if (selectedRange.value === 'custom' && start && end) generateReport()
}

const exportMarkdown = () => {
  if (!reportData.value) return
  let markdown = `# 周报\n\n`
  const data = reportData.value as any
  markdown += `**时间范围**: ${formatDate(data.startDate)} 至 ${formatDate(data.endDate)}\n\n`
  markdown += `## 项目统计表\n\n| 项目 | 耗时天数 |\n| --- | --- |\n`
  for (const stat of data.projectStats) markdown += `| ${getProjectName(stat.projectId)} | ${stat.daysActive} |\n`
  markdown += `\n## 本周工作总结\n\n`
  for (const [projectId, tasks] of Object.entries(data.weeklyWork)) {
    markdown += `### ${getProjectName(projectId)}\n\n`
    for (const task of tasks as any[]) markdown += `- ${task.text}\n`
    markdown += `\n`
  }
  markdown += `## 下周工作计划\n\n`
  for (const [projectId, tasks] of Object.entries(data.nextWeekPlan)) {
    markdown += `### ${getProjectName(projectId)}\n\n`
    for (const task of tasks as any[]) markdown += `- ${task.text}\n`
    markdown += `\n`
  }
  const blob = new Blob([markdown], { type: 'text/markdown' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `weekly_report_${formatDate(data.startDate)}_${formatDate(data.endDate)}.md`
  document.body.appendChild(a)
  a.click()
  document.body.removeChild(a)
  URL.revokeObjectURL(url)
}

const exportWord = async () => {
  if (!reportData.value) return
  try {
    const result = await tauri.exportWordReport(reportData.value)
    if (result) toast.success('Word文档导出成功！')
  } catch (error) {
    handleError(error, { context: '导出Word', showToast: true })
  }
}

let unlistenDataChanged: (() => void) | undefined

onMounted(async () => {
  const range = calculateDateRange('thisWeek')
  startDate.value = range.start.toISOString().split('T')[0]
  endDate.value = range.end.toISOString().split('T')[0]
  await generateReport()

  try {
    const cb = ({ type }: any) => {
      if (type === 'todos' || type === 'projects') generateReport()
    }
    unlistenDataChanged = await tauri.onDataChanged(cb)
  } catch {}
})

onBeforeUnmount(() => {
  unlistenDataChanged?.()
})
</script>

<style scoped>
.weekly-report-container { padding: 20px; width: 100%; max-width: 100%; }
.report-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 30px; flex-wrap: wrap; gap: 20px; }
.report-header h2 { margin: 0; color: oklch(var(--bc)); font-size: 24px; }
.report-controls { display: flex; gap: 16px; align-items: flex-end; flex-wrap: wrap; }
.report-tabs { display: flex; gap: 4px; margin-bottom: 20px; border-bottom: 2px solid oklch(var(--bc) / 0.1); }
.tab-btn { padding: 8px 20px; border: none; background: transparent; color: oklch(var(--bc) / 0.6); font-size: 14px; font-weight: 500; cursor: pointer; border-bottom: 2px solid transparent; margin-bottom: -2px; transition: all 0.2s ease; }
.tab-btn:hover { color: oklch(var(--bc)); }
.tab-btn.active { color: oklch(var(--p)); border-bottom-color: oklch(var(--p)); }
.history-list { margin-bottom: 20px; }
.loading-state, .empty-state { text-align: center; padding: 40px; color: oklch(var(--bc) / 0.6); }
.history-items { display: flex; flex-direction: column; gap: 8px; }
.history-item { display: flex; justify-content: space-between; align-items: center; padding: 12px 16px; background: oklch(var(--b1)); border: 1px solid oklch(var(--bc) / 0.1); border-radius: 8px; cursor: pointer; transition: all 0.2s ease; }
.history-item:hover { border-color: oklch(var(--p)); background: oklch(var(--p) / 0.1); }
.history-item.selected { border-color: oklch(var(--p)); background: oklch(var(--p) / 0.1); }
.history-item-info { display: flex; flex-direction: column; gap: 4px; }
.history-date { font-weight: 500; color: oklch(var(--bc)); }
.history-created { font-size: 12px; color: oklch(var(--bc) / 0.6); }
.history-load-btn { padding: 6px 16px; border: 1px solid oklch(var(--p)); background: transparent; color: oklch(var(--p)); border-radius: 6px; cursor: pointer; font-size: 13px; transition: all 0.2s ease; }
.history-load-btn:hover { background: oklch(var(--p)); color: white; }
.history-report-view { margin-top: 20px; }
.history-report-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; }
.history-report-header h3 { margin: 0; color: oklch(var(--bc)); }
.btn-ghost { padding: 6px 14px; border: 1px solid oklch(var(--bc) / 0.1); background: transparent; color: oklch(var(--bc) / 0.6); border-radius: 6px; cursor: pointer; font-size: 13px; }
.btn-ghost:hover { border-color: oklch(var(--p)); color: oklch(var(--p)); }
</style>

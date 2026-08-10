import { ref, computed, onMounted, onBeforeUnmount, watch, nextTick } from 'vue'
import { useToast } from './useToast'
import { getErrorMessage } from '../utils/helpers'
import { getTauriAPI } from '../utils/tauri-api'
import { convertFileSrc } from '@tauri-apps/api/core'

export function useAccountingBook() {
  const toast = useToast()



// Types
interface Attachment {
  path: string
  type: string
  name: string
  size: number
}

interface PendingAttachment {
  path?: string // 编辑时保留的既有附件路径（已上传过，无需再传）
  name: string
  type: 'image' | 'pdf'
  size: number
  data: string // base64（新选附件才有值）
  preview: string
}

interface AccountingRecord {
  id: string
  date: string
  type: string
  category: string
  amount: number
  description: string
  status: string
  entity: string
  project: string
  supplier: string
  invoice_number: string
  tax_amount: number
  payment_method: string
  approver: string
  voucher_number: string
  attachments_json: Attachment[]
  createdBy: string
  createdAt: string
  updatedAt: string
}

interface AccountingCategory {
  id: string
  name: string
  type: string
  icon: string
  sortOrder: number
  createdAt: string
  builtin?: boolean
}

// State
const records = ref<AccountingRecord[]>([])
const categories = ref<AccountingCategory[]>([])
const loading = ref(false)
const statsLoading = ref(false)

const stats = ref({
  totalIncome: 0,
  totalExpense: 0,
  balance: 0,
  pendingAmount: 0,
  reimbursedAmount: 0,
  dailyAvg: 0,
  incomeMom: 0,
  expenseMom: 0,
  byCategory: [] as Array<{ category: string; amount: number }>
})

const currentPage = ref(1)
const pageSize = 50
const totalRecords = ref(0)

// Filters
const periodFilter = ref('month')
const typeFilter = ref('all')
const categoryFilter = ref('all')
const statusFilter = ref('all')
const paymentFilter = ref('all')
const entityFilter = ref('')
const projectFilter = ref('')
const searchQuery = ref('')

const todayStr = () => new Date().toISOString().slice(0, 10)
const customStartDate = ref(new Date().toISOString().slice(0, 7) + '-01')
const customEndDate = ref(todayStr())

// Record form
const showRecordForm = ref(false)
const editingRecord = ref<AccountingRecord | null>(null)
const form = ref({
  id: '',
  date: todayStr(),
  type: 'expense' as 'income' | 'expense',
  category: '',
  amount: 0,
  description: '',
  status: 'pending' as string,
  entity: '',
  project: '',
  supplier: '',
  invoice_number: '',
  tax_amount: 0,
  payment_method: '',
  approver: '',
  voucher_number: '',
  attachments: [] as PendingAttachment[]
})

// File upload
const isDragOver = ref(false)
const fileInputRef = ref<HTMLInputElement | null>(null)

// Preview
const showPreview = ref(false)
const previewSrc = ref('')
const previewName = ref('')
const previewIsPdf = ref(false)
const previewLoading = ref(false)
const previewGallery = ref<Attachment[]>([])
const previewIndex = ref(0)

// Category manager
const showCategoryManager = ref(false)
const newCategory = ref({ type: 'expense' as 'income' | 'expense', name: '', icon: '📌' })

// Budget manager
const showBudgetManager = ref(false)
const budgets = ref<Array<{ id: string; category: string; amount: number; period: string; createdAt: string }>>([])
const newBudget = ref({ category: '', amount: 0 as number })
const budgetAlerts = ref<Array<{ category: string; budget: number; spent: number; percent: number; over: boolean }>>([])

// Templates
const showTemplates = ref(false)
const showTemplateEditor = ref(false)
const editingTemplate = ref<{ id: string } | null>(null)
const templates = ref<Array<{ id: string; name: string; type: string; category: string; amount: number; description: string; entity: string; project: string; supplier: string; payment_method: string; tax_rate: number; useCount: number; createdAt: string }>>([])
const templateForm = ref({ name: '', type: 'expense' as string, category: '', amount: 0, description: '', entity: '', project: '', supplier: '', payment_method: '', tax_rate: 0 })

// Confirm dialog
const confirmDialog = ref<HTMLDialogElement | null>(null)
const confirmMessage = ref('')
let confirmCallback: (() => void) | null = null

function showConfirm(message: string, callback: () => void) {
  confirmMessage.value = message
  confirmCallback = callback
  confirmDialog.value?.showModal()
}
function executeConfirm() {
  confirmCallback?.()
  confirmCallback = null
  confirmDialog.value?.close()
}
function cancelConfirm() {
  confirmCallback = null
  confirmDialog.value?.close()
}

// Trend chart
const trendData = ref<Array<{ month: string; income: number; expense: number; count: number }>>([])
const trendLoading = ref(false)
const trendChartRef = ref<HTMLDivElement | null>(null)
const trendChartWidth = ref(700)
let trendResizeObserver: ResizeObserver | null = null

async function loadTrend() {
  trendLoading.value = true
  try {
    const result = await getTauriAPI().getAccountingTrend(12)
    if (result && Array.isArray(result)) {
      // Backend returns rows as [{ month, type, total }] — need to merge by month
      const map = new Map<string, { income: number; expense: number; count: number }>()
      for (const row of result) {
        const month = row.month as string
        if (!map.has(month)) {map.set(month, { income: 0, expense: 0, count: 0 })}
        const entry = map.get(month)!
        if (row.type === 'income') {entry.income = row.total as number}
        else if (row.type === 'expense') {entry.expense = row.total as number}
        entry.count++
      }
      // Convert Map to sorted array (ascending by month)
      trendData.value = Array.from(map.entries())
        .map(([month, data]) => ({ month, ...data }))
        .sort((a, b) => a.month.localeCompare(b.month))
    } else {
      trendData.value = []
    }
  } catch (_e) { console.error('加载趋势失败:', _e) }
  finally { trendLoading.value = false }
}
function initTrendResizeObserver() {
  if (!trendChartRef.value) {return}
  trendResizeObserver = new ResizeObserver((entries) => {
    for (const entry of entries) {
      const w = entry.contentRect.width
      if (w > 0) {trendChartWidth.value = Math.max(w, 500)}
    }
  })
  trendResizeObserver.observe(trendChartRef.value)
}

// Computed
// 历史值下拉（从已有记录去重提取，输入即记忆、下次直接选）
function distinctValues(get: (r: any) => string): string[] {
  const seen = new Set<string>()
  const out: string[] = []
  for (const r of records.value) {
    const v = (get(r) || '').trim()
    if (v && !seen.has(v)) {seen.add(v); out.push(v)}
  }
  return out.slice(0, 50)
}
const supplierOptions = computed(() => distinctValues(r => r.supplier))
const entityOptions = computed(() => distinctValues(r => r.entity))
const projectOptions = computed(() => distinctValues(r => r.project))

const formCategories = computed(() => {
  if (categories.value.length === 0) {
    // Fallback enterprise categories
    return getEnterpriseCategories().filter(c => c.type === form.value.type)
  }
  return categories.value.filter(c => c.type === form.value.type)
})

const filteredCategories = computed(() => {
  if (typeFilter.value === 'all') {return categories.value.length ? categories.value : getEnterpriseCategories()}
  const all = categories.value.length ? categories.value : getEnterpriseCategories()
  return all.filter(c => c.type === typeFilter.value)
})

const incomeCategories = computed(() => {
  const all = categories.value.length ? categories.value : getEnterpriseCategories()
  return all.filter(c => c.type === 'income')
})

const expenseCategories = computed(() => {
  const all = categories.value.length ? categories.value : getEnterpriseCategories()
  return all.filter(c => c.type === 'expense')
})

const topCategories = computed(() => {
  if (stats.value.totalExpense === 0) {return []}
  return stats.value.byCategory
    .filter(c => c.amount > 0)
    .sort((a, b) => b.amount - a.amount)
    .slice(0, 5)
    .map(c => ({ ...c, percent: Math.round((c.amount / stats.value.totalExpense) * 100) }))
})

const totalPages = computed(() => Math.max(1, Math.ceil(totalRecords.value / pageSize)))
const formValid = computed(() => form.value.category && form.value.amount > 0 && form.value.date)

// Enterprise default categories
function getEnterpriseCategories(): AccountingCategory[] {
  return [
    // Expense categories
    { id: 'e1', name: '服务器费用', type: 'expense', icon: '🖥️', sortOrder: 1, createdAt: '', builtin: true },
    { id: 'e2', name: '办公采购', type: 'expense', icon: '📦', sortOrder: 2, createdAt: '', builtin: true },
    { id: 'e3', name: '差旅费', type: 'expense', icon: '✈️', sortOrder: 3, createdAt: '', builtin: true },
    { id: 'e4', name: '业务招待', type: 'expense', icon: '🍽️', sortOrder: 4, createdAt: '', builtin: true },
    { id: 'e5', name: '工资薪酬', type: 'expense', icon: '💰', sortOrder: 5, createdAt: '', builtin: true },
    { id: 'e6', name: '技术服务费', type: 'expense', icon: '🔧', sortOrder: 6, createdAt: '', builtin: true },
    { id: 'e7', name: '房租水电', type: 'expense', icon: '🏢', sortOrder: 7, createdAt: '', builtin: true },
    { id: 'e8', name: '营销推广', type: 'expense', icon: '📢', sortOrder: 8, createdAt: '', builtin: true },
    { id: 'e9', name: '税费', type: 'expense', icon: '🏛️', sortOrder: 9, createdAt: '', builtin: true },
    { id: 'e10', name: '其他', type: 'expense', icon: '📌', sortOrder: 10, createdAt: '', builtin: true },
    // Income categories
    { id: 'i1', name: '客户回款', type: 'income', icon: '💵', sortOrder: 1, createdAt: '', builtin: true },
    { id: 'i2', name: '项目收入', type: 'income', icon: '📊', sortOrder: 2, createdAt: '', builtin: true },
    { id: 'i3', name: '投资收益', type: 'income', icon: '📈', sortOrder: 3, createdAt: '', builtin: true },
    { id: 'i4', name: '其他收入', type: 'income', icon: '💎', sortOrder: 4, createdAt: '', builtin: true },
  ]
}

// Approval helpers
function canApprove(record: AccountingRecord): boolean {
  return record.status === 'pending'
}

function canReimburse(record: AccountingRecord): boolean {
  return record.status === 'approved'
}

function statusLabel(status: string): string {
  const map: Record<string, string> = {
    pending: '待审批',
    approved: '已审批',
    rejected: '已驳回',
    reimbursed: '已报销',
    confirmed: '已确认',
    void: '已作废'
  }
  return map[status] || status
}

// Date range calculation
function getDateRange(): { startDate: string; endDate: string } {
  const now = new Date()
  switch (periodFilter.value) {
    case 'month': {
      const start = new Date(now.getFullYear(), now.getMonth(), 1)
      const end = new Date(now.getFullYear(), now.getMonth() + 1, 0)
      return { startDate: start.toISOString().slice(0, 10), endDate: end.toISOString().slice(0, 10) }
    }
    case 'lastMonth': {
      const start = new Date(now.getFullYear(), now.getMonth() - 1, 1)
      const end = new Date(now.getFullYear(), now.getMonth(), 0)
      return { startDate: start.toISOString().slice(0, 10), endDate: end.toISOString().slice(0, 10) }
    }
    case 'quarter': {
      const quarter = Math.floor(now.getMonth() / 3)
      const start = new Date(now.getFullYear(), quarter * 3, 1)
      const end = new Date(now.getFullYear(), quarter * 3 + 3, 0)
      return { startDate: start.toISOString().slice(0, 10), endDate: end.toISOString().slice(0, 10) }
    }
    case 'year': {
      const start = new Date(now.getFullYear(), 0, 1)
      const end = new Date(now.getFullYear(), 11, 31)
      return { startDate: start.toISOString().slice(0, 10), endDate: end.toISOString().slice(0, 10) }
    }
    case 'custom':
      return { startDate: customStartDate.value, endDate: customEndDate.value }
    default:
      return { startDate: customStartDate.value, endDate: customEndDate.value }
  }
}

interface AccountingRecordsQuery {
  startDate: string
  endDate: string
  page: number
  pageSize: number
  type?: string
  category?: string
  status?: string
  payment_method?: string
  entity?: string
  project?: string
  search?: string
}

// Data loading
async function loadData() {
  loading.value = true
  try {
    const range = getDateRange()
    const params: AccountingRecordsQuery = {
      startDate: range.startDate,
      endDate: range.endDate,
      page: currentPage.value,
      pageSize
    }
    if (typeFilter.value !== 'all') {
      params.type = typeFilter.value
      const cat = categories.value.find(c => c.name === categoryFilter.value)
      if (cat && cat.type !== typeFilter.value) {categoryFilter.value = 'all'}
    }
    if (categoryFilter.value !== 'all') {params.category = categoryFilter.value}
    if (statusFilter.value !== 'all') {params.status = statusFilter.value}
    if (paymentFilter.value !== 'all') {params.payment_method = paymentFilter.value}
    if (entityFilter.value) {params.entity = entityFilter.value}
    if (projectFilter.value) {params.project = projectFilter.value}
    if (searchQuery.value) {params.search = searchQuery.value}

    const result = await getTauriAPI().getAccountingRecords(params)
    if (result) {
      records.value = (result.records || []) as unknown as AccountingRecord[]
      totalRecords.value = result.total || 0
    }
  } catch (e: unknown) {
    toast.error('加载记录失败: ' + getErrorMessage(e))
  } finally {
    loading.value = false
  }
}

async function loadStats() {
  statsLoading.value = true
  try {
    const range = getDateRange()
    const result = await getTauriAPI().getAccountingStats({ startDate: range.startDate, endDate: range.endDate })
    if (result) {
      stats.value.totalIncome = result.totalIncome
      stats.value.totalExpense = result.totalExpense
      stats.value.balance = result.balance
      stats.value.pendingAmount = result.pendingAmount || 0
      stats.value.reimbursedAmount = result.reimbursedAmount || 0
      stats.value.byCategory = result.byCategory || []

      // Daily average expense
      const start = new Date(range.startDate)
      const end = new Date(range.endDate)
      const days = Math.max(1, Math.ceil((end.getTime() - start.getTime()) / (1000 * 60 * 60 * 24)) + 1)
      stats.value.dailyAvg = stats.value.totalExpense / days
    }

    // MoM comparison (compare with previous period)
    const prevRange = getPreviousPeriodRange()
    if (prevRange) {
      const prevResult = await getTauriAPI().getAccountingStats({ startDate: prevRange.startDate, endDate: prevRange.endDate })
      if (prevResult) {
        if (prevResult.totalIncome === 0) {
          stats.value.incomeMom = stats.value.totalIncome > 0 ? 999 : 0
        } else {
          stats.value.incomeMom = Math.round(((stats.value.totalIncome - prevResult.totalIncome) / prevResult.totalIncome) * 100)
        }
        if (prevResult.totalExpense === 0) {
          stats.value.expenseMom = stats.value.totalExpense > 0 ? 999 : 0
        } else {
          stats.value.expenseMom = Math.round(((stats.value.totalExpense - prevResult.totalExpense) / prevResult.totalExpense) * 100)
        }
      }
    }
  } catch (e: unknown) {
    console.error('加载统计失败:', e)
  } finally {
    statsLoading.value = false
  }
}

function getPreviousPeriodRange(): { startDate: string; endDate: string } | null {
  const now = new Date()
  switch (periodFilter.value) {
    case 'month': {
      const start = new Date(now.getFullYear(), now.getMonth() - 1, 1)
      const end = new Date(now.getFullYear(), now.getMonth(), 0)
      return { startDate: start.toISOString().slice(0, 10), endDate: end.toISOString().slice(0, 10) }
    }
    case 'lastMonth': {
      const start = new Date(now.getFullYear(), now.getMonth() - 2, 1)
      const end = new Date(now.getFullYear(), now.getMonth() - 1, 0)
      return { startDate: start.toISOString().slice(0, 10), endDate: end.toISOString().slice(0, 10) }
    }
    case 'quarter': {
      const quarter = Math.floor(now.getMonth() / 3)
      const prevStart = new Date(now.getFullYear(), (quarter - 1) * 3, 1)
      const prevEnd = new Date(now.getFullYear(), quarter * 3, 0)
      return { startDate: prevStart.toISOString().slice(0, 10), endDate: prevEnd.toISOString().slice(0, 10) }
    }
    case 'year': {
      const start = new Date(now.getFullYear() - 1, 0, 1)
      const end = new Date(now.getFullYear() - 1, 11, 31)
      return { startDate: start.toISOString().slice(0, 10), endDate: end.toISOString().slice(0, 10) }
    }
  }
  return null
}

async function loadCategories() {
  try {
    const result = await getTauriAPI().getAccountingCategories()
    if (result) {categories.value = result}
  } catch (e: unknown) {
    console.error('加载分类失败:', e)
  }
}

function trendX(i: number): number {
  const chartW = trendChartWidth.value - 60
  return 50 + (chartW / Math.max(trendData.value.length - 1, 1)) * i
}

function trendMaxVal(): number {
  let max = 0
  for (const d of trendData.value) {
    max = Math.max(max, d.income, d.expense)
  }
  return max || 1
}

function trendDotY(val: number): number {
  const maxVal = trendMaxVal()
  return 170 - (val / maxVal) * 150
}

function trendBarY(val: number): number { return trendDotY(val) }
function trendBarH(val: number): number { return 170 - trendDotY(val) }

function trendLinePoints(field: 'income' | 'expense'): string {
  return trendData.value.map((d, i) => `${trendX(i)},${trendDotY(d[field])}`).join(' ')
}

const trendGridLines = computed(() => {
  const max = trendMaxVal()
  const steps = 4
  const lines = []
  for (let i = 0; i <= steps; i++) {
    const val = Math.round((max / steps) * i)
    const y = 170 - (val / max) * 150
    lines.push({ y, label: val >= 10000 ? `${(val / 10000).toFixed(1)}万` : val >= 1000 ? `${(val / 1000).toFixed(1)}k` : val.toString() })
  }
  return lines
})

// Budget
async function loadBudgets() {
  try {
    const result = await getTauriAPI().getBudgets()
    if (result) {budgets.value = result}
  } catch (_e) { console.error('加载预算失败:', _e) }
}

async function loadBudgetAlerts() {
  try {
    const result = await getTauriAPI().checkBudgetAlerts()
    if (result) {budgetAlerts.value = result}
  } catch (_e) { console.error('加载预算预警失败:', _e) }
}

async function addNewBudget() {
  if (!newBudget.value.category || newBudget.value.amount <= 0) {return}
  try {
    await getTauriAPI().addBudget({ category: newBudget.value.category, amount: newBudget.value.amount })
    toast.success('预算已添加')
    newBudget.value.category = ''
    newBudget.value.amount = 0
    await Promise.all([loadBudgets(), loadBudgetAlerts()])
  } catch (e: unknown) { toast.error('添加失败: ' + getErrorMessage(e)) }
}

async function deleteBudgetConfirm(b: { id: string; category: string }) {
  showConfirm(`确定删除 ${b.category} 的预算？`, async () => {
    try {
      await getTauriAPI().deleteBudget(b.id)
      toast.success('预算已删除')
      await Promise.all([loadBudgets(), loadBudgetAlerts()])
    } catch (e: unknown) { toast.error('删除失败: ' + getErrorMessage(e)) }
  })
}

// Templates
async function loadTemplates() {
  try {
    const result = await getTauriAPI().getTemplates()
    if (result) {templates.value = result}
  } catch (_e) { console.error('加载模板失败:', _e) }
}

async function useTemplate(tpl: { id: string; name: string; type: string; category: string; amount: number; description: string; entity: string; project: string; supplier: string; payment_method: string; tax_rate?: number }) {
  editingRecord.value = null
  form.value = {
    id: '', date: todayStr(), type: tpl.type as 'income' | 'expense', category: tpl.category,
    amount: tpl.amount, description: tpl.description || '', status: 'pending',
    entity: tpl.entity || '', project: tpl.project || '', supplier: tpl.supplier || '',
    invoice_number: '', tax_amount: 0, payment_method: tpl.payment_method || '',
    approver: '', voucher_number: '', attachments: []
  }
  // 应用模板税率 → 预填税额（tax_amount = amount * tax_rate）
  if (tpl.tax_rate && tpl.amount > 0) {
    form.value.tax_amount = Math.round(tpl.amount * tpl.tax_rate) / 100
  }
  showTemplates.value = false
  showRecordForm.value = true
  try {
    await getTauriAPI().useTemplate?.(tpl.id)
  } catch (_e) {
    toast.warning('模板使用计数更新失败')
  }
}

function editTemplate(tpl: { id: string; name: string; type: string; category: string; amount: number; description: string; entity: string; project: string; supplier: string; payment_method: string; tax_rate: number }) {
  editingTemplate.value = { id: tpl.id }
  templateForm.value = {
    name: tpl.name, type: tpl.type, category: tpl.category, amount: tpl.amount,
    description: tpl.description, entity: tpl.entity, project: tpl.project,
    supplier: tpl.supplier, payment_method: tpl.payment_method, tax_rate: tpl.tax_rate
  }
  showTemplateEditor.value = true
}

async function deleteTemplateConfirm(tpl: { id: string; name: string }) {
  showConfirm(`确定删除模板"${tpl.name}"？`, async () => {
    try {
      await getTauriAPI().deleteTemplate(tpl.id)
      toast.success('模板已删除')
      await loadTemplates()
    } catch (e: unknown) { toast.error('删除失败: ' + getErrorMessage(e)) }
  })
}

async function saveTemplate() {
  if (!templateForm.value.name || !templateForm.value.category || templateForm.value.amount <= 0) {
    toast.warning('请填写模板名称、分类和金额')
    return
  }
  try {
    if (editingTemplate.value) {
      await getTauriAPI().updateTemplate(editingTemplate.value.id, { ...templateForm.value })
      toast.success('模板已更新')
    } else {
      await getTauriAPI().addTemplate({ ...templateForm.value })
      toast.success('模板已保存')
    }
    showTemplateEditor.value = false
    await loadTemplates()
  } catch (e: unknown) { toast.error('保存失败: ' + getErrorMessage(e)) }
}

function saveAsTemplate() {
  if (!formValid.value) {return}
  editingTemplate.value = null
  // 税率从当前凭证推算（tax_amount / amount），不再写死 0
  const taxRate = form.value.tax_amount && form.value.amount > 0
    ? Math.round((Number(form.value.tax_amount) / Number(form.value.amount)) * 100)
    : 0
  templateForm.value = {
    name: `${form.value.category}`, type: form.value.type, category: form.value.category,
    amount: form.value.amount, description: form.value.description, entity: form.value.entity,
    project: form.value.project, supplier: form.value.supplier, payment_method: form.value.payment_method,
    tax_rate: taxRate
  }
  showTemplateEditor.value = true
}

// Handlers
function onPeriodChange() {
  currentPage.value = 1
  loadData()
  loadStats()
}

function goToPage(page: number) {
  currentPage.value = page
  loadData()
}

let searchTimer: ReturnType<typeof setTimeout>
function debounceSearch() {
  clearTimeout(searchTimer)
  searchTimer = setTimeout(() => {
    currentPage.value = 1
    loadData()
  }, 300)
}

// Record CRUD
function openAddRecord() {
  editingRecord.value = null
  form.value = {
    id: '',
    date: todayStr(),
    type: 'expense',
    category: '',
    amount: 0,
    description: '',
    status: 'pending',
    entity: '',
    project: '',
    supplier: '',
    invoice_number: '',
    tax_amount: 0,
    payment_method: '',
    approver: '',
    voucher_number: '',
    attachments: []
  }
  showRecordForm.value = true
}

function editRecord(record: AccountingRecord) {
  editingRecord.value = record
  form.value = {
    id: record.id,
    date: record.date,
    type: record.type as 'income' | 'expense',
    category: record.category,
    amount: record.amount,
    description: record.description || '',
    status: record.status,
    entity: record.entity || '',
    project: record.project || '',
    supplier: record.supplier || '',
    invoice_number: record.invoice_number || '',
    tax_amount: record.tax_amount || 0,
    payment_method: record.payment_method || '',
    approver: record.approver || '',
    voucher_number: record.voucher_number || '',
    attachments: Array.isArray(record.attachments_json) ? record.attachments_json.map((a: Attachment) => ({
      path: a.path, name: a.name, type: a.type as 'image' | 'pdf', size: a.size, data: '', preview: ''
    })) : []
  }
  showRecordForm.value = true
}

function closeRecordForm() {
  showRecordForm.value = false
  editingRecord.value = null
}

async function saveRecord() {
  if (!formValid.value) {return}
  if (form.value.amount <= 0 || isNaN(form.value.amount)) {
    toast.warning('金额必须大于 0')
    return
  }
  try {
    // Upload attachments first（编辑保留的既有附件有 path，跳过上传；仅上传新选的）
    const attachments: Attachment[] = []
    for (const att of form.value.attachments) {
      if (att.path !== undefined && att.path !== '') {
        attachments.push({ path: att.path, type: att.type, name: att.name, size: att.size })
      } else if (att.data && att.data.length > 0 && getTauriAPI().uploadAccountingReceipt) {
        const result = await getTauriAPI().uploadAccountingReceipt(att.name, att.data)
        if (result) {
          attachments.push(result)
        }
      }
    }

    const recordData = {
      date: form.value.date,
      type: form.value.type,
      category: form.value.category,
      amount: form.value.amount,
      description: form.value.description,
      status: form.value.status,
      entity: form.value.entity,
      project: form.value.project,
      supplier: form.value.supplier,
      invoice_number: form.value.invoice_number,
      tax_amount: form.value.tax_amount,
      payment_method: form.value.payment_method,
      approver: form.value.approver,
      attachments_json: JSON.stringify(attachments)
    }

    if (editingRecord.value) {
      await getTauriAPI().updateAccountingRecord(editingRecord.value.id, recordData)
      toast.success('凭证已更新')
    } else {
      await getTauriAPI().addAccountingRecord(recordData)
      toast.success('凭证已添加')
    }
    closeRecordForm()
    loadData()
    loadStats()
  } catch (e: unknown) {
    toast.error('保存失败: ' + getErrorMessage(e))
  }
}

async function deleteRecord(record: AccountingRecord) {
  showConfirm(`确定删除 ${record.date} 的 ${record.category} ¥${formatMoney(record.amount)} 记录？`, async () => {
    try {
      await getTauriAPI().deleteAccountingRecord(record.id)
      toast.success('记录已删除')
      loadData()
      loadStats()
    } catch (e: unknown) {
      toast.error('删除失败: ' + getErrorMessage(e))
    }
  })
}

async function approveRecord(record: AccountingRecord, newStatus: string) {
  try {
    await getTauriAPI().updateAccountingRecord(record.id, { status: newStatus })
    toast.success(`状态已更新为: ${statusLabel(newStatus)}`)
    loadData()
    loadStats()
  } catch (e: unknown) {
    toast.error('状态更新失败: ' + getErrorMessage(e))
  }
}

// File upload handling
function triggerFileInput() {
  fileInputRef.value?.click()
}

async function handleFileSelect(event: Event) {
  const input = event.target as HTMLInputElement
  if (!input.files) {return}
  await processFiles(Array.from(input.files))
  input.value = '' // Reset for re-selection
}

function handleDrop(event: DragEvent) {
  isDragOver.value = false
  const files = event.dataTransfer?.files
  if (!files) {return}
  processFiles(Array.from(files))
}

async function processFiles(files: File[]) {
  const allowedTypes = ['application/pdf', 'image/png', 'image/jpeg', 'image/gif', 'image/bmp', 'image/webp']
  for (const file of files) {
    if (!allowedTypes.includes(file.type)) {
      toast.warning(`不支持的文件格式: ${file.name}`)
      continue
    }
    try {
      const base64 = await fileToBase64(file)
      const isPdf = file.type === 'application/pdf'
      form.value.attachments.push({
        name: file.name,
        type: isPdf ? 'pdf' : 'image',
        size: file.size,
        data: base64,
        preview: isPdf ? '' : base64
      })
    } catch (_e) {
      toast.error(`读取文件失败: ${file.name}`)
    }
  }
}

function fileToBase64(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader()
    reader.onload = () => resolve(reader.result as string)
    reader.onerror = reject
    reader.readAsDataURL(file)
  })
}

function removeAttachment(idx: number) {
  form.value.attachments.splice(idx, 1)
}

function formatFileSize(bytes: number): string {
  if (bytes < 1024) {return bytes + ' B'}
  if (bytes < 1024 * 1024) {return (bytes / 1024).toFixed(1) + ' KB'}
  return (bytes / (1024 * 1024)).toFixed(1) + ' MB'
}

// Receipt preview
function isImage(name: string | undefined | null): boolean {
  if (!name) {return false}
  const ext = name.split('.').pop()?.toLowerCase()
  return ['png', 'jpg', 'jpeg', 'gif', 'bmp', 'webp'].includes(ext || '')
}

function getFileUrl(filePath: string): string {
  // In Tauri, convert local file path to asset protocol URL
  // convertFileSrc handles the tauri://asset/ protocol
  if (filePath.startsWith('file://')) {return filePath}
  if (filePath.startsWith('tauri://') || filePath.startsWith('https://asset.localhost')) {return filePath}
  try {
    if (convertFileSrc) {return convertFileSrc(filePath)}
  } catch {}
  // Fallback: try asset protocol URL directly
  return `https://asset.localhost/${encodeURIComponent(filePath)}`
}

async function openPreview(attachment: Attachment, gallery?: Attachment[]) {
  previewGallery.value = gallery || [attachment]
  previewIndex.value = gallery ? gallery.indexOf(attachment) : 0
  previewName.value = attachment.name
  previewIsPdf.value = !isImage(attachment.name)
  previewLoading.value = true
  showPreview.value = true

  try {
    if (getTauriAPI().getAccountingReceiptFile) {
      const result = await getTauriAPI().getAccountingReceiptFile(attachment.path)
      if (result && result.success) {
        previewSrc.value = result.dataUrl
      } else {
        // Fallback to direct file URL
        previewSrc.value = getFileUrl(attachment.path)
      }
    } else {
      previewSrc.value = getFileUrl(attachment.path)
    }
  } catch (_e) {
    previewSrc.value = getFileUrl(attachment.path)
  } finally {
    previewLoading.value = false
  }
}

function closePreview() {
  showPreview.value = false
  previewSrc.value = ''
  previewLoading.value = false
  previewGallery.value = []
}

// Export
async function exportCSV() {
  try {
    const range = getDateRange()
    const params: AccountingRecordsQuery = { startDate: range.startDate, endDate: range.endDate, page: 1, pageSize: 99999 }
    if (typeFilter.value !== 'all') {params.type = typeFilter.value}
    if (categoryFilter.value !== 'all') {params.category = categoryFilter.value}
    if (statusFilter.value !== 'all') {params.status = statusFilter.value}
    if (paymentFilter.value !== 'all') {params.payment_method = paymentFilter.value}
    if (entityFilter.value) {params.entity = entityFilter.value}
    if (projectFilter.value) {params.project = projectFilter.value}
    if (searchQuery.value) {params.search = searchQuery.value}

    const csvContent = await getTauriAPI().exportAccountingCSV(params as unknown as Record<string, unknown>)
    if (!csvContent) {
      toast.warning('没有可导出的数据')
      return
    }

    const blob = new Blob(['\ufeff' + csvContent], { type: 'text/csv;charset=utf-8' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `账本_${range.startDate}_${range.endDate}.csv`
    a.click()
    URL.revokeObjectURL(url)
    toast.success('账本已导出')
  } catch (e: unknown) {
    toast.error('导出失败: ' + getErrorMessage(e))
  }
}

// Category management
async function addNewCategory() {
  if (!newCategory.value.name.trim()) {return}
  try {
    await getTauriAPI().addAccountingCategory({
      name: newCategory.value.name.trim(),
      type: newCategory.value.type,
      icon: newCategory.value.icon || '📌',
      sortOrder: 99
    })
    toast.success('分类已添加')
    newCategory.value.name = ''
    await loadCategories()
  } catch (e: unknown) {
    toast.error('添加失败: ' + getErrorMessage(e))
  }
}

async function deleteCategory(id: string) {
  showConfirm('确定删除此分类？', async () => {
    try {
      await getTauriAPI().deleteAccountingCategory(id)
      toast.success('分类已删除')
      await loadCategories()
    } catch (e: unknown) {
      toast.error('删除失败: ' + getErrorMessage(e))
    }
  })
}

// Utils
function formatMoney(amount: number | null | undefined): string {
  if (amount == null) {return '0.00'}
  return amount.toFixed(2).replace(/\B(?=(\d{3})+(?!\d))/g, ',')
}

function formatDate(date: string): string {
  return date.replace(/-/g, '/')
}

// Init
onMounted(async () => {
  await getTauriAPI().getAccountingCategories?.()
  await Promise.all([loadCategories(), loadData(), loadStats(), loadTrend(), loadBudgets(), loadBudgetAlerts(), loadTemplates()])
})

// Watch for trend data to appear before attaching ResizeObserver
watch(trendData, async (data) => {
  if (data.length > 0) {
    await nextTick()
    initTrendResizeObserver()
  }
}, { once: true })

// Cleanup
onBeforeUnmount(() => {
  if (searchTimer) {clearTimeout(searchTimer)}
  if (trendResizeObserver) {trendResizeObserver.disconnect()}
})

  return {
    records, categories, loading, statsLoading, stats, currentPage, pageSize,
    totalRecords, periodFilter, typeFilter, categoryFilter, statusFilter,
    paymentFilter, entityFilter, projectFilter, searchQuery, customStartDate,
    customEndDate, showRecordForm, editingRecord, form, isDragOver,
    fileInputRef, showPreview, previewSrc, previewName, previewIsPdf,
    previewLoading, previewGallery, previewIndex, showCategoryManager,
    newCategory, showBudgetManager, budgets, newBudget, budgetAlerts,
    showTemplates, showTemplateEditor, editingTemplate, templates, templateForm,
    trendData, trendChartRef, trendChartWidth, trendLoading,
    formCategories, filteredCategories, incomeCategories, expenseCategories,
    topCategories, totalPages, formValid,
    supplierOptions, entityOptions, projectOptions,
    loadData, loadStats, loadBudgets, loadCategories, loadTemplates, loadTrend,
    saveRecord, editRecord, deleteRecord,
    openPreview, closePreview, removeAttachment, handleFileSelect,
    processFiles,
    saveTemplate, saveAsTemplate, useTemplate, editTemplate,
    deleteTemplateConfirm, deleteCategory, deleteBudgetConfirm,
    approveRecord, exportCSV, openAddRecord, onPeriodChange,
    loadBudgetAlerts, formatMoney, formatDate, formatFileSize,
    trendGridLines, trendX, trendDotY, trendBarY, trendBarH, trendLinePoints, trendMaxVal,
    debounceSearch, isImage, getFileUrl, statusLabel,
    canApprove, canReimburse, goToPage, closeRecordForm, handleDrop, triggerFileInput,
    fileToBase64, getEnterpriseCategories, getDateRange, getPreviousPeriodRange,
    addNewCategory, addNewBudget, initTrendResizeObserver,
    confirmDialog, confirmMessage, executeConfirm, cancelConfirm, showConfirm,
  }
}

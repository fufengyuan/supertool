<template>
  <div class="flex flex-col h-full gap-4">
    <!-- Header -->
    <div class="flex items-center justify-between shrink-0">
      <h2 class="text-xl font-bold text-base-content"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20"/><path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z"/></svg> 企业记账</h2>
      <div class="flex gap-2">
        <button @click="showTemplates = true" class="btn btn-ghost btn-sm" title="快捷模板">
          <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"/></svg>  快捷记账
        </button>
        <button @click="openAddRecord" class="btn btn-primary btn-sm">
          <SvgIcon name="plus" :size="14" />
          新增凭证
        </button>
        <button @click="exportCSV" class="btn btn-ghost btn-sm" :disabled="statsLoading">
          <SvgIcon name="download" :size="14" />
          导出账本
        </button>
        <button @click="showBudgetManager = true" class="btn btn-ghost btn-sm" title="预算管理">
          <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><circle cx="12" cy="12" r="6"/><circle cx="12" cy="12" r="2"/></svg>  预算
        </button>
        <button @click="showCategoryManager = true" class="btn btn-ghost btn-sm">
          <SvgIcon name="settings" :size="14" class="inline-block align-text-bottom" /> 分类管理
        </button>
      </div>
    </div>

    <!-- Stats Cards -->
    <div class="grid grid-cols-7 gap-3 shrink-0 max-lg:grid-cols-3 max-sm:grid-cols-2">
      <div class="bg-base-100 border border-base-content/10 rounded-xl p-3 flex flex-col gap-1">
        <div class="text-xs text-base-content/60 font-medium">总收入</div>
        <div class="text-xl font-bold font-mono text-success">¥{{ formatMoney(stats.totalIncome) }}</div>
        <div v-if="stats.incomeMom !== 0" class="text-[10px] font-mono mt-0.5" :class="stats.incomeMom >= 0 ? 'text-error' : 'text-success'">
          {{ stats.incomeMom >= 0 ? '↑' : '↓' }}{{ stats.incomeMom >= 999 ? '新增' : Math.abs(stats.incomeMom) + '%' }} 环比
        </div>
      </div>
      <div class="bg-base-100 border border-base-content/10 rounded-xl p-3 flex flex-col gap-1">
        <div class="text-xs text-base-content/60 font-medium">总支出</div>
        <div class="text-xl font-bold font-mono text-error">¥{{ formatMoney(stats.totalExpense) }}</div>
        <div v-if="stats.expenseMom !== 0" class="text-[10px] font-mono mt-0.5" :class="stats.expenseMom >= 0 ? 'text-error' : 'text-success'">
          {{ stats.expenseMom >= 0 ? '↑' : '↓' }}{{ Math.abs(stats.expenseMom) }}% 环比
        </div>
      </div>
      <div class="bg-base-100 border border-base-content/10 rounded-xl p-3 flex flex-col gap-1">
        <div class="text-xs text-base-content/60 font-medium">结余</div>
        <div class="text-xl font-bold font-mono" :class="stats.balance >= 0 ? 'text-primary' : 'text-error'">¥{{ formatMoney(stats.balance) }}</div>
      </div>
      <div class="bg-base-100 border border-base-content/10 rounded-xl p-3 flex flex-col gap-1">
        <div class="text-xs text-base-content/60 font-medium">日均支出</div>
        <div class="text-xl font-bold font-mono text-warning">¥{{ formatMoney(stats.dailyAvg) }}</div>
      </div>
      <div class="bg-base-100 border border-base-content/10 rounded-xl p-3 flex flex-col gap-1">
        <div class="text-xs text-base-content/60 font-medium">待审批金额</div>
        <div class="text-xl font-bold font-mono text-warning">¥{{ formatMoney(stats.pendingAmount) }}</div>
      </div>
      <div class="bg-base-100 border border-base-content/10 rounded-xl p-3 flex flex-col gap-1">
        <div class="text-xs text-base-content/60 font-medium">已报销金额</div>
        <div class="text-xl font-bold font-mono text-success">¥{{ formatMoney(stats.reimbursedAmount) }}</div>
      </div>
      <div class="bg-base-100 border border-base-content/10 rounded-xl p-3 flex flex-col gap-1">
        <div class="text-xs text-base-content/60 font-medium">统计周期</div>
        <div class="flex items-center gap-1.5">
          <select v-model="periodFilter" @change="onPeriodChange" class="select select-sm select-bordered">
            <option value="month">本月</option>
            <option value="lastMonth">上月</option>
            <option value="quarter">本季度</option>
            <option value="year">本年</option>
            <option value="custom">自定义</option>
          </select>
          <template v-if="periodFilter === 'custom'">
            <input type="date" v-model="customStartDate" @change="loadData" class="input input-sm input-bordered" />
            <span class="text-[11px] text-base-content/60">至</span>
            <input type="date" v-model="customEndDate" @change="loadData" class="input input-sm input-bordered" />
          </template>
        </div>
      </div>
    </div>

    <!-- Category Summary -->
    <div v-if="stats.byCategory.length > 0" class="bg-base-100 border border-base-content/10 rounded-xl p-3 shrink-0">
      <div class="text-xs font-semibold text-base-content/60 mb-2">支出分类占比</div>
      <div class="flex flex-col gap-1.5">
        <div v-for="cat in topCategories" :key="cat.category" class="flex items-center gap-2 text-xs">
          <span class="w-[70px] shrink-0 text-base-content truncate">{{ cat.category }}</span>
          <div class="h-2 bg-base-200 rounded overflow-hidden">
            <div class="h-full bg-error/70 rounded transition-all duration-300" :style="{ width: cat.percent + '%' }"></div>
          </div>
          <span class="w-20 text-right text-base-content/60 font-mono text-[11px]">¥{{ formatMoney(cat.amount) }}</span>
        </div>
      </div>
    </div>

    <!-- Budget Alerts -->
    <div v-if="budgetAlerts.length > 0" class="bg-base-100 border border-warning rounded-xl p-3 shrink-0">
      <div class="flex justify-between items-center mb-2">
        <span class="text-xs font-semibold text-warning"><SvgIcon name="alertTriangle" :size="14" class="inline-block align-text-bottom" /> 预算预警</span>
        <button @click="loadBudgetAlerts" class="btn btn-ghost btn-xs" title="刷新"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="23 4 23 10 17 10"/><polyline points="1 20 1 14 7 14"/><path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"/></svg> </button>
      </div>
      <div class="flex flex-col gap-1.5">
        <div v-for="alert in budgetAlerts" :key="alert.category" class="flex items-center gap-2 text-xs" :class="{ 'bg-error/5 rounded-sm p-1 -mx-1 -my-1': alert.over }">
          <span class="w-[70px] shrink-0 text-base-content truncate">{{ alert.category }}</span>
          <div class="flex-1 h-2 bg-base-200 rounded overflow-hidden">
            <div class="h-full bg-gradient-to-r from-warning to-error rounded transition-all duration-300" :style="{ width: Math.min(alert.percent, 100) + '%' }"></div>
          </div>
          <span class="w-9 text-right font-bold font-mono text-[11px]" :class="alert.over ? 'text-error' : 'text-warning'">{{ alert.percent }}%</span>
          <span class="w-[120px] text-right text-base-content/60 font-mono text-[11px]">¥{{ formatMoney(alert.spent) }} / ¥{{ formatMoney(alert.budget) }}</span>
        </div>
      </div>
    </div>

    <!-- Monthly Trend Chart -->
    <div v-if="trendData.length > 0" class="bg-base-100 border border-base-content/10 rounded-xl p-3 shrink-0">
      <div class="text-xs font-semibold text-base-content/60 mb-2">
        <span><SvgIcon name="trendingUp" :size="14" class="inline-block align-text-bottom" /> 月度趋势（12个月）</span>
      </div>
      <div class="w-full overflow-x-auto" ref="trendChartRef">
        <svg :width="trendChartWidth" :height="200" class="block min-w-[500px]">
          <!-- Grid lines -->
          <g v-for="(line, i) in trendGridLines" :key="'g' + i">
            <line :x1="50" :y1="line.y" :x2="trendChartWidth - 10" :y2="line.y" stroke="color-mix(in oklab, var(--color-base-content) 10%, transparent)" stroke-width="0.5" stroke-dasharray="2,4" />
            <text :x="45" :y="line.y + 4" text-anchor="end" class="fill-base-content/60 text-[9px] font-mono">{{ line.label }}</text>
          </g>
          <!-- X axis labels -->
          <g v-for="(d, i) in trendData" :key="'x' + i">
            <text :x="trendX(i)" :y="190" text-anchor="middle" class="fill-base-content/60 text-[9px] font-mono" :transform="trendData.length > 8 ? `rotate(-30, ${trendX(i)}, 190)` : ''">{{ d.month.slice(2) }}</text>
          </g>
          <!-- Expense bars -->
          <g v-for="(d, i) in trendData" :key="'b' + i">
            <rect :x="trendX(i) - 8" :y="trendBarY(d.expense)" :width="16" :height="trendBarH(d.expense)" fill="var(--color-error)" opacity="0.3" rx="2" />
          </g>
          <!-- Income line -->
          <polyline :points="trendLinePoints('income')" fill="none" stroke="var(--color-success)" stroke-width="2" stroke-linejoin="round" />
          <!-- Expense line -->
          <polyline :points="trendLinePoints('expense')" fill="none" stroke="var(--color-error)" stroke-width="2" stroke-linejoin="round" />
          <!-- Income dots -->
          <g v-for="(d, i) in trendData" :key="'di' + i">
            <circle :cx="trendX(i)" :cy="trendDotY(d.income)" r="3" fill="var(--color-success)" />
          </g>
          <!-- Expense dots -->
          <g v-for="(d, i) in trendData" :key="'de' + i">
            <circle :cx="trendX(i)" :cy="trendDotY(d.expense)" r="3" fill="var(--color-error)" />
          </g>
        </svg>
        <!-- Legend -->
        <div class="flex gap-4 justify-center mt-1 text-[11px] text-base-content/60">
          <span class="flex items-center gap-1"><span class="w-2 h-2 rounded-full bg-success inline-block"></span>收入</span>
          <span class="flex items-center gap-1"><span class="w-2 h-2 rounded-full bg-error inline-block"></span>支出</span>
          <span class="flex items-center gap-1"><span class="w-2 h-2 rounded-full bg-error/30 inline-block"></span>支出柱状</span>
        </div>
      </div>
    </div>

    <!-- Filter Bar -->
    <div class="flex items-center gap-2 shrink-0 flex-wrap max-sm:flex-col max-sm:items-stretch">
      <div class="flex gap-1 max-sm:w-full">
        <button
          class="px-3 py-1 border rounded-full text-xs cursor-pointer transition-all duration-150"
          :class="typeFilter === 'all' ? 'bg-primary text-white border-primary' : 'bg-transparent text-base-content/60 border-base-content/10 hover:bg-primary/10 hover:text-primary'"
          @click="typeFilter = 'all'; loadData()">全部</button>
        <button
          class="px-3 py-1 border rounded-full text-xs cursor-pointer transition-all duration-150"
          :class="typeFilter === 'income' ? 'bg-success text-white border-success' : 'bg-transparent text-base-content/60 border-base-content/10 hover:bg-primary/10 hover:text-primary'"
          @click="typeFilter = 'income'; loadData()">收入</button>
        <button
          class="px-3 py-1 border rounded-full text-xs cursor-pointer transition-all duration-150"
          :class="typeFilter === 'expense' ? 'bg-error text-white border-error' : 'bg-transparent text-base-content/60 border-base-content/10 hover:bg-primary/10 hover:text-primary'"
          @click="typeFilter = 'expense'; loadData()">支出</button>
      </div>
      <div class="flex gap-1 max-sm:w-full">
        <select v-model="categoryFilter" @change="loadData" class="select select-sm select-bordered max-sm:w-full">
          <option value="all">全部分类</option>
          <option v-for="cat in filteredCategories" :key="cat.id" :value="cat.name">{{ cat.name }}</option>
        </select>
      </div>
      <div class="flex gap-1 max-sm:w-full">
        <select v-model="statusFilter" @change="loadData" class="select select-sm select-bordered max-sm:w-full">
          <option value="all">全部状态</option>
          <option value="pending">待审批</option>
          <option value="approved">已审批</option>
          <option value="rejected">已驳回</option>
          <option value="reimbursed">已报销</option>
        </select>
      </div>
      <div class="flex gap-1 max-sm:w-full">
        <select v-model="paymentFilter" @change="loadData" class="select select-sm select-bordered max-sm:w-full">
          <option value="all">全部付款方式</option>
          <option value="银行转账">银行转账</option>
          <option value="支付宝">支付宝</option>
          <option value="微信">微信</option>
          <option value="现金">现金</option>
          <option value="信用卡">信用卡</option>
        </select>
      </div>
      <div class="flex gap-1 max-sm:w-full">
        <input v-model="entityFilter" @input="debounceSearch" placeholder="企业主体/部门" class="input input-sm input-bordered w-[120px] max-sm:w-full" />
      </div>
      <div class="flex gap-1 max-sm:w-full">
        <input v-model="projectFilter" @input="debounceSearch" placeholder="所属项目" class="input input-sm input-bordered w-[120px] max-sm:w-full" />
      </div>
      <div class="flex-1 min-w-[150px] max-sm:w-full">
        <input v-model="searchQuery" @input="debounceSearch" placeholder="搜索备注/发票号/供应商..." class="input input-sm input-bordered w-full" />
      </div>
    </div>

    <!-- Table -->
    <div class="flex-1 overflow-auto border border-base-content/10 rounded-xl">
      <table v-if="records.length > 0" class="w-full border-collapse text-sm min-w-[1400px]">
        <thead>
          <tr>
            <th class="px-3 py-2.5 text-left text-xs font-semibold text-base-content/60 bg-base-200 border-b border-base-content/10 sticky top-0 z-[2] whitespace-nowrap">凭证号</th>
            <th class="px-3 py-2.5 text-left text-xs font-semibold text-base-content/60 bg-base-200 border-b border-base-content/10 sticky top-0 z-[2] whitespace-nowrap">日期</th>
            <th class="px-3 py-2.5 text-left text-xs font-semibold text-base-content/60 bg-base-200 border-b border-base-content/10 sticky top-0 z-[2] whitespace-nowrap">凭证</th>
            <th class="px-3 py-2.5 text-left text-xs font-semibold text-base-content/60 bg-base-200 border-b border-base-content/10 sticky top-0 z-[2] whitespace-nowrap">类型</th>
            <th class="px-3 py-2.5 text-left text-xs font-semibold text-base-content/60 bg-base-200 border-b border-base-content/10 sticky top-0 z-[2] whitespace-nowrap">分类</th>
            <th class="px-3 py-2.5 text-left text-xs font-semibold text-base-content/60 bg-base-200 border-b border-base-content/10 sticky top-0 z-[2] whitespace-nowrap">企业主体</th>
            <th class="px-3 py-2.5 text-left text-xs font-semibold text-base-content/60 bg-base-200 border-b border-base-content/10 sticky top-0 z-[2] whitespace-nowrap">项目</th>
            <th class="px-3 py-2.5 text-left text-xs font-semibold text-base-content/60 bg-base-200 border-b border-base-content/10 sticky top-0 z-[2] whitespace-nowrap">供应商</th>
            <th class="px-3 py-2.5 text-left text-xs font-semibold text-base-content/60 bg-base-200 border-b border-base-content/10 sticky top-0 z-[2] whitespace-nowrap">发票号</th>
            <th class="px-3 py-2.5 text-left text-xs font-semibold text-base-content/60 bg-base-200 border-b border-base-content/10 sticky top-0 z-[2] whitespace-nowrap">金额</th>
            <th class="px-3 py-2.5 text-left text-xs font-semibold text-base-content/60 bg-base-200 border-b border-base-content/10 sticky top-0 z-[2] whitespace-nowrap">税额</th>
            <th class="px-3 py-2.5 text-left text-xs font-semibold text-base-content/60 bg-base-200 border-b border-base-content/10 sticky top-0 z-[2] whitespace-nowrap">付款方式</th>
            <th class="px-3 py-2.5 text-left text-xs font-semibold text-base-content/60 bg-base-200 border-b border-base-content/10 sticky top-0 z-[2] whitespace-nowrap">审批人</th>
            <th class="px-3 py-2.5 text-left text-xs font-semibold text-base-content/60 bg-base-200 border-b border-base-content/10 sticky top-0 z-[2] whitespace-nowrap">审批状态</th>
            <th class="px-3 py-2.5 text-left text-xs font-semibold text-base-content/60 bg-base-200 border-b border-base-content/10 sticky top-0 z-[2] whitespace-nowrap">备注</th>
            <th class="px-3 py-2.5 text-left text-xs font-semibold text-base-content/60 bg-base-200 border-b border-base-content/10 sticky top-0 z-[2] whitespace-nowrap">操作</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="record in records" :key="record.id" class="hover:bg-primary/10">
            <td class="px-3 py-2 border-b border-base-content/10 whitespace-nowrap font-mono text-[11px] text-base-content/60">{{ record.voucher_number || '—' }}</td>
            <td class="px-3 py-2 border-b border-base-content/10 whitespace-nowrap">{{ formatDate(record.date) }}</td>
            <td class="px-3 py-2 border-b border-base-content/10 whitespace-nowrap min-w-[100px]">
              <div class="flex items-center gap-1" v-if="record.attachments_json && record.attachments_json.length">
                <div
                  v-for="(att, idx) in record.attachments_json.slice(0, 3)"
                  :key="idx"
                  class="w-8 h-8 rounded overflow-hidden cursor-pointer border border-base-content/10 flex items-center justify-center bg-base-200 transition-transform duration-100 hover:scale-110"
                  @click="openPreview(att, record.attachments_json)"
                >
                  <img v-if="isImage(att.name)" :src="getFileUrl(att.path)" :alt="att.name" class="w-full h-full object-cover" />
                  <div v-else class="w-full h-full flex items-center justify-center text-error">
                    <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="2">
                      <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
                      <polyline points="14 2 14 8 20 8" />
                    </svg>
                  </div>
                </div>
                <span v-if="record.attachments_json.length > 3" class="text-[10px] text-base-content/60 px-1">+{{ record.attachments_json.length - 3 }}</span>
              </div>
              <span v-else class="text-base-content/60 text-xs">—</span>
            </td>
            <td class="px-3 py-2 border-b border-base-content/10 whitespace-nowrap">
              <span class="px-2 py-0.5 rounded-full text-xs font-medium" :class="record.type === 'income' ? 'bg-success/10 text-success' : 'bg-error/10 text-error'">
                {{ record.type === 'income' ? '收入' : '支出' }}
              </span>
            </td>
            <td class="px-3 py-2 border-b border-base-content/10 whitespace-nowrap">{{ record.category }}</td>
            <td class="px-3 py-2 border-b border-base-content/10 whitespace-nowrap" :title="record.entity">{{ record.entity || '—' }}</td>
            <td class="px-3 py-2 border-b border-base-content/10 whitespace-nowrap" :title="record.project">{{ record.project || '—' }}</td>
            <td class="px-3 py-2 border-b border-base-content/10 whitespace-nowrap" :title="record.supplier">{{ record.supplier || '—' }}</td>
            <td class="px-3 py-2 border-b border-base-content/10 whitespace-nowrap font-mono text-[11px] text-base-content/60">{{ record.invoice_number || '—' }}</td>
            <td class="px-3 py-2 border-b border-base-content/10 whitespace-nowrap font-mono font-semibold" :class="record.type === 'income' ? 'text-success' : 'text-error'">
              {{ record.type === 'income' ? '+' : '-' }}¥{{ formatMoney(record.amount) }}
            </td>
            <td class="px-3 py-2 border-b border-base-content/10 whitespace-nowrap">{{ record.tax_amount ? '¥' + formatMoney(record.tax_amount) : '—' }}</td>
            <td class="px-3 py-2 border-b border-base-content/10 whitespace-nowrap">{{ record.payment_method || '—' }}</td>
            <td class="px-3 py-2 border-b border-base-content/10 whitespace-nowrap">{{ record.approver || '—' }}</td>
            <td class="px-3 py-2 border-b border-base-content/10 whitespace-nowrap">
              <span class="px-2 py-0.5 rounded text-xs font-medium"
                :class="{
                  'bg-warning/15 text-warning': record.status === 'pending',
                  'bg-success/15 text-success': record.status === 'approved' || record.status === 'confirmed',
                  'bg-error/15 text-error': record.status === 'rejected',
                  'bg-primary/15 text-primary': record.status === 'reimbursed',
                }">
                {{ statusLabel(record.status) }}
              </span>
            </td>
            <td class="px-3 py-2 border-b border-base-content/10 whitespace-nowrap" :title="record.description">{{ record.description || '—' }}</td>
            <td class="px-3 py-2 border-b border-base-content/10 whitespace-nowrap">
              <button @click="editRecord(record)" class="border-none bg-transparent cursor-pointer text-sm px-1 py-0.5 rounded transition-colors duration-100 hover:bg-primary/10" title="编辑"><SvgIcon name="pencil" :size="14" /> </button>
              <button v-if="canApprove(record)" @click="approveRecord(record, 'approved')" class="border-none bg-transparent cursor-pointer text-sm px-1 py-0.5 rounded transition-colors duration-100 hover:bg-success/15" title="审批通过"><SvgIcon name="check" :size="14" /> </button>
              <button v-if="canApprove(record)" @click="approveRecord(record, 'rejected')" class="border-none bg-transparent cursor-pointer text-sm px-1 py-0.5 rounded transition-colors duration-100 hover:bg-error/15" title="驳回"><SvgIcon name="x" :size="14" /> </button>
              <button v-if="canReimburse(record)" @click="approveRecord(record, 'reimbursed')" class="border-none bg-transparent cursor-pointer text-sm px-1 py-0.5 rounded transition-colors duration-100 hover:bg-primary/15" title="标记报销"><SvgIcon name="coin" :size="14" class="inline-block align-text-bottom" /></button>
              <button @click="deleteRecord(record)" class="border-none bg-transparent cursor-pointer text-sm px-1 py-0.5 rounded transition-colors duration-100 hover:bg-error/15" title="删除"><SvgIcon name="trash" :size="14" /> </button>
            </td>
          </tr>
        </tbody>
      </table>
      <div v-else-if="loading" class="flex flex-col items-center justify-center py-12 px-4 text-base-content/60 gap-3">加载中...</div>
      <div v-else class="flex flex-col items-center justify-center py-12 px-4 text-base-content/60 gap-3">
        <p class="m-0">暂无记录</p>
        <button @click="openAddRecord" class="btn btn-primary btn-sm">新增第一笔</button>
      </div>
    </div>

    <!-- Pagination -->
    <div v-if="totalRecords > pageSize" class="flex items-center justify-center gap-4 py-2 shrink-0">
      <button class="btn btn-ghost btn-sm" :disabled="currentPage <= 1" @click="goToPage(currentPage - 1)">‹ 上一页</button>
      <span class="text-xs text-base-content/60">第 {{ currentPage }} / {{ totalPages }} 页，共 {{ totalRecords }} 条</span>
      <button class="btn btn-ghost btn-sm" :disabled="currentPage >= totalPages" @click="goToPage(currentPage + 1)">下一页 ›</button>
    </div>

    <!-- Record Form Modal -->
    <Teleport to="body">
      <div v-if="showRecordForm" class="fixed inset-0 bg-black/50 flex items-center justify-center z-[10000]" @click.self="closeRecordForm">
        <div class="bg-base-100 border border-base-content/10 rounded-xl w-[720px] max-h-[90vh] flex flex-col shadow-2xl max-sm:!w-[95vw]">
          <div class="flex items-center justify-between px-5 py-4 border-b border-base-content/10">
            <h3 class="m-0 text-base font-semibold text-base-content">{{ editingRecord ? '编辑凭证' : '新增凭证' }}</h3>
            <button @click="closeRecordForm" class="w-7 h-7 border-none bg-transparent text-base-content/60 text-xl cursor-pointer rounded-lg flex items-center justify-center hover:bg-primary/10 hover:text-primary">×</button>
          </div>
          <div class="px-5 py-4 overflow-y-auto flex-1">
            <!-- Voucher Number (read-only, auto-generated) -->
            <div v-if="form.voucher_number" class="flex flex-col items-start gap-1.5 mb-3">
              <label class="text-sm font-medium text-base-content/60">凭证号</label>
              <div class="font-mono text-sm text-primary font-semibold p-2 bg-primary/10 rounded-lg w-full">{{ form.voucher_number }}</div>
            </div>

            <!-- Type Toggle -->
            <div class="flex items-center gap-3 mb-3">
              <label class="text-sm font-medium text-base-content/60 w-[70px] shrink-0">类型</label>
              <div class="flex gap-1 flex-1">
                <button
                  class="flex-1 px-3 py-2 border rounded-lg text-sm font-medium cursor-pointer transition-all duration-150"
                  :class="form.type === 'expense' ? 'bg-primary text-white border-primary' : 'border-base-content/10 bg-transparent text-base-content/60 hover:bg-primary/10'"
                  @click="form.type = 'expense'; form.category = ''">支出</button>
                <button
                  class="flex-1 px-3 py-2 border rounded-lg text-sm font-medium cursor-pointer transition-all duration-150"
                  :class="form.type === 'income' ? 'bg-primary text-white border-primary' : 'border-base-content/10 bg-transparent text-base-content/60 hover:bg-primary/10'"
                  @click="form.type = 'income'; form.category = ''">收入</button>
              </div>
            </div>

            <!-- Date & Amount Row -->
            <div class="flex items-center gap-3 mb-3">
              <label class="text-sm font-medium text-base-content/60 w-[70px] shrink-0">日期</label>
              <input type="date" v-model="form.date" class="flex-1 px-3 py-2 border border-base-content/10 rounded-lg bg-base-200 text-base-content text-sm outline-none focus:border-primary focus:ring-2 focus:ring-primary/10" />
              <label class="text-sm font-medium text-base-content/60 w-[60px] shrink-0 text-right">金额</label>
              <input type="number" v-model.number="form.amount" step="0.01" min="0" placeholder="0.00" class="flex-1 px-3 py-2 border border-base-content/10 rounded-lg bg-base-200 text-base-content text-sm outline-none focus:border-primary focus:ring-2 focus:ring-primary/10 font-mono text-base font-bold" />
            </div>

            <!-- Category & Status -->
            <div class="flex items-center gap-3 mb-3">
              <label class="text-sm font-medium text-base-content/60 w-[70px] shrink-0">分类</label>
              <select v-model="form.category" class="flex-1 px-3 py-2 border border-base-content/10 rounded-lg bg-base-200 text-base-content text-sm outline-none focus:border-primary focus:ring-2 focus:ring-primary/10">
                <option value="">选择分类</option>
                <option v-for="cat in formCategories" :key="cat.id" :value="cat.name">{{ cat.name }}</option>
              </select>
              <label class="text-sm font-medium text-base-content/60 w-[60px] shrink-0 text-right">审批状态</label>
              <select v-model="form.status" class="flex-1 px-3 py-2 border border-base-content/10 rounded-lg bg-base-200 text-base-content text-sm outline-none focus:border-primary focus:ring-2 focus:ring-primary/10">
                <option value="pending">待审批</option>
                <option value="approved">已审批</option>
                <option value="rejected">已驳回</option>
                <option value="reimbursed">已报销</option>
              </select>
            </div>

            <!-- Enterprise Fields Row 1 -->
            <div class="flex items-center gap-3 mb-3">
              <label class="text-sm font-medium text-base-content/60 w-[70px] shrink-0">企业主体</label>
              <input v-model="form.entity" placeholder="企业主体/部门" class="flex-1 px-3 py-2 border border-base-content/10 rounded-lg bg-base-200 text-base-content text-sm outline-none focus:border-primary focus:ring-2 focus:ring-primary/10" />
              <label class="text-sm font-medium text-base-content/60 w-[60px] shrink-0 text-right">所属项目</label>
              <input v-model="form.project" placeholder="所属项目" class="flex-1 px-3 py-2 border border-base-content/10 rounded-lg bg-base-200 text-base-content text-sm outline-none focus:border-primary focus:ring-2 focus:ring-primary/10" />
            </div>

            <!-- Enterprise Fields Row 2 -->
            <div class="flex items-center gap-3 mb-3">
              <label class="text-sm font-medium text-base-content/60 w-[70px] shrink-0">供应商</label>
              <input v-model="form.supplier" placeholder="供应商/对方" class="flex-1 px-3 py-2 border border-base-content/10 rounded-lg bg-base-200 text-base-content text-sm outline-none focus:border-primary focus:ring-2 focus:ring-primary/10" />
              <label class="text-sm font-medium text-base-content/60 w-[60px] shrink-0 text-right">发票号</label>
              <input v-model="form.invoice_number" placeholder="发票号码" class="flex-1 px-3 py-2 border border-base-content/10 rounded-lg bg-base-200 text-base-content text-sm outline-none focus:border-primary focus:ring-2 focus:ring-primary/10" />
            </div>

            <!-- Enterprise Fields Row 3 -->
            <div class="flex items-center gap-3 mb-3">
              <label class="text-sm font-medium text-base-content/60 w-[70px] shrink-0">税额</label>
              <input type="number" v-model.number="form.tax_amount" step="0.01" min="0" placeholder="0.00" class="flex-1 px-3 py-2 border border-base-content/10 rounded-lg bg-base-200 text-base-content text-sm outline-none focus:border-primary focus:ring-2 focus:ring-primary/10" />
              <label class="text-sm font-medium text-base-content/60 w-[60px] shrink-0 text-right">付款方式</label>
              <select v-model="form.payment_method" class="flex-1 px-3 py-2 border border-base-content/10 rounded-lg bg-base-200 text-base-content text-sm outline-none focus:border-primary focus:ring-2 focus:ring-primary/10">
                <option value="">选择付款方式</option>
                <option value="银行转账">银行转账</option>
                <option value="支付宝">支付宝</option>
                <option value="微信">微信</option>
                <option value="现金">现金</option>
                <option value="信用卡">信用卡</option>
              </select>
            </div>

            <!-- Enterprise Fields Row 4 -->
            <div class="flex items-center gap-3 mb-3">
              <label class="text-sm font-medium text-base-content/60 w-[70px] shrink-0">审批人</label>
              <input v-model="form.approver" placeholder="审批人" class="flex-1 px-3 py-2 border border-base-content/10 rounded-lg bg-base-200 text-base-content text-sm outline-none focus:border-primary focus:ring-2 focus:ring-primary/10" />
              <label class="text-sm font-medium text-base-content/60 w-[60px] shrink-0 text-right">备注</label>
              <input v-model="form.description" placeholder="可选备注说明" class="flex-1 px-3 py-2 border border-base-content/10 rounded-lg bg-base-200 text-base-content text-sm outline-none focus:border-primary focus:ring-2 focus:ring-primary/10" />
            </div>

            <!-- Receipt Upload Area -->
            <div class="flex flex-col items-start gap-1.5 mb-3">
              <label class="text-sm font-medium text-base-content/60">凭证附件</label>
            </div>
            <div
              class="border-2 border-dashed border-base-content/10 rounded-xl p-6 text-center cursor-pointer transition-all duration-200 bg-base-200 hover:border-primary hover:bg-primary/10"
              :class="{ '!border-primary !bg-primary/10': isDragOver }"
              @dragover.prevent="isDragOver = true"
              @dragleave.prevent="isDragOver = false"
              @drop.prevent="handleDrop"
              @click="triggerFileInput"
            >
              <input
                ref="fileInputRef"
                type="file"
                multiple
                accept=".pdf,.png,.jpg,.jpeg,.gif,.bmp,.webp"
                @change="handleFileSelect"
                style="display: none"
              />
              <div v-if="!form.attachments.length" class="flex flex-col items-center gap-2">
                <svg viewBox="0 0 24 24" width="40" height="40" fill="none" stroke="currentColor" stroke-width="1.5" class="text-base-content/60 opacity-50">
                  <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
                  <polyline points="17 8 12 3 7 8" />
                  <line x1="12" y1="3" x2="12" y2="15" />
                </svg>
                <p class="text-sm text-base-content m-0">拖拽文件到此处，或点击选择</p>
                <p class="text-xs text-base-content/60 m-0">支持 PDF、PNG、JPG、GIF 格式</p>
              </div>
              <div v-else class="grid grid-cols-[repeat(auto-fill,minmax(140px,1fr))] gap-3">
                <div v-for="(att, idx) in form.attachments" :key="idx" class="flex flex-col items-center gap-1.5 p-2 border border-base-content/10 rounded-lg bg-base-100 relative">
                  <div class="w-16 h-16 rounded-lg overflow-hidden flex items-center justify-center bg-base-200">
                    <img v-if="att.type === 'image'" :src="att.preview" :alt="att.name" class="w-full h-full object-cover" />
                    <div v-else class="text-error">
                      <svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
                        <polyline points="14 2 14 8 20 8" />
                      </svg>
                    </div>
                  </div>
                  <div class="flex flex-col items-center w-full">
                    <span class="text-[11px] text-base-content truncate max-w-[120px]">{{ att.name }}</span>
                    <span class="text-[10px] text-base-content/60">{{ formatFileSize(att.size) }}</span>
                  </div>
                  <button @click.stop="removeAttachment(idx)" class="absolute top-1 right-1 w-5 h-5 border-none bg-error text-white rounded-full text-sm leading-none cursor-pointer flex items-center justify-center" title="移除">×</button>
                </div>
                <div class="flex items-center justify-center p-6 border-2 border-dashed border-base-content/10 rounded-lg text-base-content/60 text-sm cursor-pointer transition-all duration-150 hover:border-primary hover:text-primary" @click.stop="triggerFileInput">+ 添加更多</div>
              </div>
            </div>
          </div>
          <div class="flex justify-end gap-2 px-5 py-3 border-t border-base-content/10">
            <button @click="closeRecordForm" class="btn btn-ghost">取消</button>
            <button v-if="formValid" @click="saveAsTemplate" class="btn btn-ghost" title="保存为模板"><SvgIcon name="download" :size="14" />  存模板</button>
            <button @click="saveRecord" class="btn btn-primary" :disabled="!formValid">{{ editingRecord ? '保存' : '添加' }}</button>
          </div>
        </div>
      </div>

      <!-- Receipt Preview Modal -->
      <div v-if="showPreview" class="fixed inset-0 bg-black/50 flex items-start justify-center z-[10000] p-10" @click.self="closePreview">
        <div class="bg-base-100 border border-base-content/10 rounded-xl w-[90vw] max-h-[85vh] flex flex-col shadow-2xl" :class="{ 'max-w-[1000px]': previewIsPdf, 'max-w-[900px]': !previewIsPdf }">
          <div class="flex items-center justify-between px-5 py-4 border-b border-base-content/10">
            <h3 class="text-sm max-w-[400px] truncate m-0 font-semibold text-base-content">{{ previewName }}</h3>
            <div class="flex items-center gap-2" v-if="previewGallery.length > 1">
              <button class="border border-base-content/10 bg-base-200 text-base-content w-7 h-7 rounded-lg cursor-pointer flex items-center justify-center text-sm disabled:opacity-30 disabled:cursor-not-allowed" @click="previewIndex = Math.max(0, previewIndex - 1)" :disabled="previewIndex === 0">‹</button>
              <span class="text-xs text-base-content/60 font-mono">{{ previewIndex + 1 }} / {{ previewGallery.length }}</span>
              <button class="border border-base-content/10 bg-base-200 text-base-content w-7 h-7 rounded-lg cursor-pointer flex items-center justify-center text-sm disabled:opacity-30 disabled:cursor-not-allowed" @click="previewIndex = Math.min(previewGallery.length - 1, previewIndex + 1)" :disabled="previewIndex === previewGallery.length - 1">›</button>
            </div>
            <button @click="closePreview" class="w-7 h-7 border-none bg-transparent text-base-content/60 text-xl cursor-pointer rounded-lg flex items-center justify-center hover:bg-primary/10 hover:text-primary">×</button>
          </div>
          <div class="p-4 flex-1 overflow-auto flex items-center justify-center min-h-[300px]">
            <div v-if="previewLoading" class="text-base-content/60 text-sm">加载中...</div>
            <template v-else>
              <img
                v-if="!previewIsPdf"
                :src="previewSrc"
                :alt="previewName"
                class="max-w-full max-h-[70vh] object-contain rounded"
              />
              <embed
                v-else
                :src="previewSrc"
                type="application/pdf"
                class="w-full h-[70vh] border-none rounded"
              />
            </template>
          </div>
        </div>
      </div>

      <!-- Category Manager Modal -->
      <div v-if="showCategoryManager" class="fixed inset-0 bg-black/50 flex items-center justify-center z-[10000]" @click.self="showCategoryManager = false">
        <div class="bg-base-100 border border-base-content/10 rounded-xl w-[560px] max-h-[80vh] flex flex-col shadow-2xl max-sm:!w-[95vw]">
          <div class="flex items-center justify-between px-5 py-4 border-b border-base-content/10">
            <h3 class="m-0 text-base font-semibold text-base-content">分类管理</h3>
            <button @click="showCategoryManager = false" class="w-7 h-7 border-none bg-transparent text-base-content/60 text-xl cursor-pointer rounded-lg flex items-center justify-center hover:bg-primary/10 hover:text-primary">×</button>
          </div>
          <div class="px-5 py-4 overflow-y-auto flex-1">
            <div class="flex flex-col gap-4">
              <div>
                <h4 class="text-sm font-semibold text-base-content m-0 mb-2"><SvgIcon name="trendingUp" :size="14" class="inline-block align-text-bottom" /> 收入分类</h4>
                <div class="flex flex-wrap gap-1.5">
                  <div v-for="cat in incomeCategories" :key="cat.id" class="flex items-center gap-1.5 px-2.5 py-1.5 border border-base-content/10 rounded-lg bg-base-200 text-sm text-base-content">
                    <span class="text-base">{{ cat.icon }}</span>
                    <span>{{ cat.name }}</span>
                    <button v-if="!cat.builtin" @click="deleteCategory(cat.id)" class="border-none bg-transparent text-base-content/60 text-base cursor-pointer px-0.5 leading-none hover:text-error">×</button>
                  </div>
                </div>
              </div>
              <div>
                <h4 class="text-sm font-semibold text-base-content m-0 mb-2"><SvgIcon name="trendingDown" :size="14" class="inline-block align-text-bottom" /> 支出分类</h4>
                <div class="flex flex-wrap gap-1.5">
                  <div v-for="cat in expenseCategories" :key="cat.id" class="flex items-center gap-1.5 px-2.5 py-1.5 border border-base-content/10 rounded-lg bg-base-200 text-sm text-base-content">
                    <span class="text-base">{{ cat.icon }}</span>
                    <span>{{ cat.name }}</span>
                    <button v-if="!cat.builtin" @click="deleteCategory(cat.id)" class="border-none bg-transparent text-base-content/60 text-base cursor-pointer px-0.5 leading-none hover:text-error">×</button>
                  </div>
                </div>
              </div>
              <div class="flex gap-2 pt-3 border-t border-base-content/10">
                <select v-model="newCategory.type" class="select select-sm select-bordered">
                  <option value="expense">支出</option>
                  <option value="income">收入</option>
                </select>
                <input v-model="newCategory.name" placeholder="分类名称" class="input input-sm input-bordered flex-1" @keyup.enter="addNewCategory" />
                <input v-model="newCategory.icon" placeholder="Emoji图标" class="input input-sm input-bordered w-[50px] shrink-0 text-center" maxlength="2" />
                <button @click="addNewCategory" class="btn btn-primary btn-sm">添加</button>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Quick Templates Modal -->
      <div v-if="showTemplates" class="fixed inset-0 bg-black/50 flex items-center justify-center z-[10000]" @click.self="showTemplates = false">
        <div class="bg-base-100 border border-base-content/10 rounded-xl w-[560px] max-h-[80vh] flex flex-col shadow-2xl max-sm:!w-[95vw]">
          <div class="flex items-center justify-between px-5 py-4 border-b border-base-content/10">
            <h3 class="m-0 text-base font-semibold text-base-content"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"/></svg>  快捷记账模板</h3>
            <button @click="showTemplates = false" class="w-7 h-7 border-none bg-transparent text-base-content/60 text-xl cursor-pointer rounded-lg flex items-center justify-center hover:bg-primary/10 hover:text-primary">×</button>
          </div>
          <div class="px-5 py-4 overflow-y-auto flex-1">
            <div v-if="templates.length === 0" class="text-center text-base-content/60 p-6 text-sm">
              <p class="m-0">暂无模板，在新增凭证时可保存为模板</p>
            </div>
            <div v-else class="flex flex-col gap-2">
              <div v-for="tpl in templates" :key="tpl.id" class="flex items-center justify-between px-3.5 py-2.5 border border-base-content/10 rounded-lg bg-base-200 transition-colors duration-100 hover:bg-base-100">
                <div class="flex items-center gap-2.5 flex-1 min-w-0">
                  <span class="px-2 py-0.5 rounded-full text-[11px] font-medium shrink-0" :class="tpl.type === 'income' ? 'bg-success/10 text-success' : 'bg-error/10 text-error'">{{ tpl.type === 'income' ? '收入' : '支出' }}</span>
                  <span class="font-semibold text-base-content">{{ tpl.name }}</span>
                  <span class="text-xs text-base-content/60">{{ tpl.category }}</span>
                  <span class="font-mono font-semibold text-base-content">¥{{ formatMoney(tpl.amount) }}</span>
                  <span v-if="tpl.useCount > 0" class="text-[10px] text-base-content/60">已用 {{ tpl.useCount }} 次</span>
                </div>
                <div class="flex gap-1 shrink-0">
                  <button @click="useTemplate(tpl)" class="btn btn-primary btn-sm">使用</button>
                  <button @click="editTemplate(tpl)" class="btn btn-ghost btn-sm">编辑</button>
                  <button @click="deleteTemplateConfirm(tpl)" class="btn btn-ghost btn-sm hover:bg-error/15">删除</button>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Budget Manager Modal -->
      <div v-if="showBudgetManager" class="fixed inset-0 bg-black/50 flex items-center justify-center z-[10000]" @click.self="showBudgetManager = false">
        <div class="bg-base-100 border border-base-content/10 rounded-xl w-[560px] max-h-[80vh] flex flex-col shadow-2xl max-sm:!w-[95vw]">
          <div class="flex items-center justify-between px-5 py-4 border-b border-base-content/10">
            <h3 class="m-0 text-base font-semibold text-base-content"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><circle cx="12" cy="12" r="6"/><circle cx="12" cy="12" r="2"/></svg>  预算管理</h3>
            <button @click="showBudgetManager = false" class="w-7 h-7 border-none bg-transparent text-base-content/60 text-xl cursor-pointer rounded-lg flex items-center justify-center hover:bg-primary/10 hover:text-primary">×</button>
          </div>
          <div class="px-5 py-4 overflow-y-auto flex-1">
            <div class="flex flex-col gap-3">
              <div class="flex gap-2">
                <select v-model="newBudget.category" class="select select-sm select-bordered">
                  <option value="">选择分类</option>
                  <option v-for="cat in expenseCategories" :key="cat.id" :value="cat.name">{{ cat.name }}</option>
                </select>
                <input v-model.number="newBudget.amount" type="number" step="100" min="0" placeholder="预算金额" class="input input-sm input-bordered flex-1" />
                <button @click="addNewBudget" class="btn btn-primary btn-sm">添加预算</button>
              </div>
              <div v-if="budgets.length === 0" class="text-center text-base-content/60 p-6 text-sm">
                <p class="m-0">暂无预算，添加分类预算后将自动监控超支情况</p>
              </div>
              <div v-else class="flex flex-col gap-1.5">
                <div v-for="b in budgets" :key="b.id" class="flex items-center gap-3 px-3 py-2 border border-base-content/10 rounded-lg bg-base-200">
                  <span class="flex-1 font-medium">{{ b.category }}</span>
                  <span class="font-mono font-semibold">¥{{ formatMoney(b.amount) }}</span>
                  <span class="text-[11px] text-base-content/60">{{ b.period === 'monthly' ? '月度' : '年度' }}</span>
                  <button @click="deleteBudgetConfirm(b)" class="btn btn-ghost btn-sm hover:bg-error/15">删除</button>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Template Editor Modal -->
      <div v-if="showTemplateEditor" class="fixed inset-0 bg-black/50 flex items-center justify-center z-[10000]" @click.self="showTemplateEditor = false">
        <div class="bg-base-100 border border-base-content/10 rounded-xl w-[560px] max-h-[80vh] flex flex-col shadow-2xl max-sm:!w-[95vw]">
          <div class="flex items-center justify-between px-5 py-4 border-b border-base-content/10">
            <h3 class="m-0 text-base font-semibold text-base-content">{{ editingTemplate ? '编辑模板' : '保存为模板' }}</h3>
            <button @click="showTemplateEditor = false" class="w-7 h-7 border-none bg-transparent text-base-content/60 text-xl cursor-pointer rounded-lg flex items-center justify-center hover:bg-primary/10 hover:text-primary">×</button>
          </div>
          <div class="px-5 py-4 overflow-y-auto flex-1">
            <div class="flex items-center gap-3 mb-3">
              <label class="text-sm font-medium text-base-content/60 w-[70px] shrink-0">模板名称</label>
              <input v-model="templateForm.name" placeholder="如：月租服务器" class="flex-1 px-3 py-2 border border-base-content/10 rounded-lg bg-base-200 text-base-content text-sm outline-none focus:border-primary focus:ring-2 focus:ring-primary/10" />
            </div>
            <div class="flex items-center gap-3 mb-3">
              <label class="text-sm font-medium text-base-content/60 w-[70px] shrink-0">类型</label>
              <select v-model="templateForm.type" class="flex-1 px-3 py-2 border border-base-content/10 rounded-lg bg-base-200 text-base-content text-sm outline-none focus:border-primary focus:ring-2 focus:ring-primary/10">
                <option value="expense">支出</option>
                <option value="income">收入</option>
              </select>
              <label class="text-sm font-medium text-base-content/60 w-[60px] shrink-0 text-right">分类</label>
              <select v-model="templateForm.category" class="flex-1 px-3 py-2 border border-base-content/10 rounded-lg bg-base-200 text-base-content text-sm outline-none focus:border-primary focus:ring-2 focus:ring-primary/10">
                <option value="">选择分类</option>
                <option v-for="cat in formCategories" :key="cat.id" :value="cat.name">{{ cat.name }}</option>
              </select>
            </div>
            <div class="flex items-center gap-3 mb-3">
              <label class="text-sm font-medium text-base-content/60 w-[70px] shrink-0">金额</label>
              <input type="number" v-model.number="templateForm.amount" step="0.01" min="0" class="flex-1 px-3 py-2 border border-base-content/10 rounded-lg bg-base-200 text-base-content text-sm outline-none focus:border-primary focus:ring-2 focus:ring-primary/10 font-mono text-base font-bold" />
            </div>
            <div class="flex items-center gap-3 mb-3">
              <label class="text-sm font-medium text-base-content/60 w-[70px] shrink-0">供应商</label>
              <input v-model="templateForm.supplier" placeholder="供应商" class="flex-1 px-3 py-2 border border-base-content/10 rounded-lg bg-base-200 text-base-content text-sm outline-none focus:border-primary focus:ring-2 focus:ring-primary/10" />
              <label class="text-sm font-medium text-base-content/60 w-[60px] shrink-0 text-right">付款方式</label>
              <select v-model="templateForm.payment_method" class="flex-1 px-3 py-2 border border-base-content/10 rounded-lg bg-base-200 text-base-content text-sm outline-none focus:border-primary focus:ring-2 focus:ring-primary/10">
                <option value="">选择</option>
                <option value="银行转账">银行转账</option>
                <option value="支付宝">支付宝</option>
                <option value="微信">微信</option>
                <option value="现金">现金</option>
                <option value="信用卡">信用卡</option>
              </select>
            </div>
            <div class="flex flex-col items-start gap-1.5 mb-3">
              <label class="text-sm font-medium text-base-content/60">备注</label>
              <input v-model="templateForm.description" placeholder="说明" class="w-full px-3 py-2 border border-base-content/10 rounded-lg bg-base-200 text-base-content text-sm outline-none focus:border-primary focus:ring-2 focus:ring-primary/10" />
            </div>
          </div>
          <div class="flex justify-end gap-2 px-5 py-3 border-t border-base-content/10">
            <button @click="showTemplateEditor = false" class="btn btn-ghost">取消</button>
            <button @click="saveTemplate" class="btn btn-primary">保存</button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import SvgIcon from '@/components/ui/SvgIcon.vue'
import { useAccountingBook } from '../../composables/useAccountingBook'

const ac = useAccountingBook()

const {
  records, categories, loading, statsLoading, stats, currentPage, pageSize,
  totalRecords, periodFilter, typeFilter, categoryFilter, statusFilter,
  paymentFilter, entityFilter, projectFilter, searchQuery, customStartDate,
  customEndDate, showRecordForm, editingRecord, form, isDragOver,
  fileInputRef, showPreview, previewSrc, previewName, previewIsPdf,
  previewLoading, previewGallery, previewIndex, showCategoryManager,
  newCategory, showBudgetManager, budgets, newBudget, budgetAlerts,
  showTemplates, showTemplateEditor, editingTemplate, templates, templateForm,
  trendData, trendChartRef, trendChartWidth,
  formCategories, filteredCategories, incomeCategories, expenseCategories,
  topCategories, totalPages, formValid,
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
  addNewCategory, addNewBudget,
} = ac
</script>

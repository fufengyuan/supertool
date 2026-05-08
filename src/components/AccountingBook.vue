<template>
  <div class="accounting-book">
    <!-- Header -->
    <div class="accounting-header">
      <h2 class="accounting-title">📒 企业记账</h2>
      <div class="header-actions">
        <button @click="showTemplates = true" class="btn btn-ghost btn-sm" title="快捷模板">
          ⚡ 快捷记账
        </button>
        <button @click="openAddRecord" class="btn btn-primary btn-sm">
          <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="12" y1="5" x2="12" y2="19" /><line x1="5" y1="12" x2="19" y2="12" />
          </svg>
          新增凭证
        </button>
        <button @click="exportCSV" class="btn btn-ghost btn-sm" :disabled="statsLoading">
          <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
            <polyline points="7 10 12 15 17 10" />
            <line x1="12" y1="15" x2="12" y2="3" />
          </svg>
          导出账本
        </button>
        <button @click="showBudgetManager = true" class="btn btn-ghost btn-sm" title="预算管理">
          🎯 预算
        </button>
        <button @click="showCategoryManager = true" class="btn btn-ghost btn-sm">
          ⚙️ 分类管理
        </button>
      </div>
    </div>

    <!-- Stats Cards -->
    <div class="stats-row">
      <div class="stat-card stat-income">
        <div class="stat-label">总收入</div>
        <div class="stat-value">¥{{ formatMoney(stats.totalIncome) }}</div>
        <div v-if="stats.incomeMom !== 0" class="stat-mom" :class="stats.incomeMom >= 0 ? 'mom-up' : 'mom-down'">
          {{ stats.incomeMom >= 0 ? '↑' : '↓' }}{{ stats.incomeMom >= 999 ? '新增' : Math.abs(stats.incomeMom) + '%' }} 环比
        </div>
      </div>
      <div class="stat-card stat-expense">
        <div class="stat-label">总支出</div>
        <div class="stat-value">¥{{ formatMoney(stats.totalExpense) }}</div>
        <div v-if="stats.expenseMom !== 0" class="stat-mom" :class="stats.expenseMom >= 0 ? 'mom-up' : 'mom-down'">
          {{ stats.expenseMom >= 0 ? '↑' : '↓' }}{{ Math.abs(stats.expenseMom) }}% 环比
        </div>
      </div>
      <div class="stat-card" :class="stats.balance >= 0 ? 'stat-balance-pos' : 'stat-balance-neg'">
        <div class="stat-label">结余</div>
        <div class="stat-value">¥{{ formatMoney(stats.balance) }}</div>
      </div>
      <div class="stat-card stat-daily">
        <div class="stat-label">日均支出</div>
        <div class="stat-value">¥{{ formatMoney(stats.dailyAvg) }}</div>
      </div>
      <div class="stat-card stat-pending">
        <div class="stat-label">待审批金额</div>
        <div class="stat-value">¥{{ formatMoney(stats.pendingAmount) }}</div>
      </div>
      <div class="stat-card stat-reimbursed">
        <div class="stat-label">已报销金额</div>
        <div class="stat-value">¥{{ formatMoney(stats.reimbursedAmount) }}</div>
      </div>
      <div class="stat-card stat-period">
        <div class="stat-label">统计周期</div>
        <div class="stat-period-selector">
          <select v-model="periodFilter" @change="onPeriodChange" class="period-select">
            <option value="month">本月</option>
            <option value="lastMonth">上月</option>
            <option value="quarter">本季度</option>
            <option value="year">本年</option>
            <option value="custom">自定义</option>
          </select>
          <template v-if="periodFilter === 'custom'">
            <input type="date" v-model="customStartDate" @change="loadData" class="period-date" />
            <span class="period-sep">至</span>
            <input type="date" v-model="customEndDate" @change="loadData" class="period-date" />
          </template>
        </div>
      </div>
    </div>

    <!-- Category Summary -->
    <div v-if="stats.byCategory.length > 0" class="category-summary">
      <div class="category-summary-title">支出分类占比</div>
      <div class="category-bars">
        <div v-for="cat in topCategories" :key="cat.category" class="category-bar-row">
          <span class="category-bar-name">{{ cat.category }}</span>
          <div class="category-bar-track">
            <div class="category-bar-fill" :style="{ width: cat.percent + '%' }"></div>
          </div>
          <span class="category-bar-amount">¥{{ formatMoney(cat.amount) }}</span>
        </div>
      </div>
    </div>

    <!-- Budget Alerts -->
    <div v-if="budgetAlerts.length > 0" class="budget-alerts">
      <div class="budget-alerts-header">
        <span class="budget-alerts-title">⚠️ 预算预警</span>
        <button @click="loadBudgetAlerts" class="btn btn-ghost btn-xs" title="刷新">🔄</button>
      </div>
      <div class="budget-alert-list">
        <div v-for="alert in budgetAlerts" :key="alert.category" class="budget-alert-item" :class="{ 'budget-over': alert.over }">
          <span class="budget-alert-cat">{{ alert.category }}</span>
          <div class="budget-alert-track">
            <div class="budget-alert-fill" :style="{ width: Math.min(alert.percent, 100) + '%' }"></div>
          </div>
          <span class="budget-alert-pct">{{ alert.percent }}%</span>
          <span class="budget-alert-amount">¥{{ formatMoney(alert.spent) }} / ¥{{ formatMoney(alert.budget) }}</span>
        </div>
      </div>
    </div>

    <!-- Monthly Trend Chart -->
    <div v-if="trendData.length > 0" class="trend-chart">
      <div class="trend-chart-header">
        <span class="trend-chart-title">📈 月度趋势（12个月）</span>
      </div>
      <div class="trend-svg-container" ref="trendChartRef">
        <svg :width="trendChartWidth" :height="200" class="trend-svg">
          <!-- Grid lines -->
          <g class="trend-grid" v-for="(line, i) in trendGridLines" :key="'g' + i">
            <line :x1="50" :y1="line.y" :x2="trendChartWidth - 10" :y2="line.y" stroke="color-mix(in oklab, var(--color-base-content) 10%, transparent)" stroke-width="0.5" stroke-dasharray="2,4" />
            <text :x="45" :y="line.y + 4" text-anchor="end" class="trend-axis-label">{{ line.label }}</text>
          </g>
          <!-- X axis labels -->
          <g v-for="(d, i) in trendData" :key="'x' + i">
            <text :x="trendX(i)" :y="190" text-anchor="middle" class="trend-axis-label" :transform="trendData.length > 8 ? `rotate(-30, ${trendX(i)}, 190)` : ''">{{ d.month.slice(2) }}</text>
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
        <div class="trend-legend">
          <span class="trend-legend-item"><span class="trend-legend-dot trend-legend-income"></span>收入</span>
          <span class="trend-legend-item"><span class="trend-legend-dot trend-legend-expense"></span>支出</span>
          <span class="trend-legend-item"><span class="trend-legend-dot trend-legend-bar"></span>支出柱状</span>
        </div>
      </div>
    </div>

    <!-- Filter Bar -->
    <div class="filter-bar">
      <div class="filter-group">
        <button class="filter-chip" :class="{ active: typeFilter === 'all' }" @click="typeFilter = 'all'; loadData()">全部</button>
        <button class="filter-chip income-chip" :class="{ active: typeFilter === 'income' }" @click="typeFilter = 'income'; loadData()">收入</button>
        <button class="filter-chip expense-chip" :class="{ active: typeFilter === 'expense' }" @click="typeFilter = 'expense'; loadData()">支出</button>
      </div>
      <div class="filter-group">
        <select v-model="categoryFilter" @change="loadData" class="filter-select">
          <option value="all">全部分类</option>
          <option v-for="cat in filteredCategories" :key="cat.id" :value="cat.name">{{ cat.name }}</option>
        </select>
      </div>
      <div class="filter-group">
        <select v-model="statusFilter" @change="loadData" class="filter-select">
          <option value="all">全部状态</option>
          <option value="pending">待审批</option>
          <option value="approved">已审批</option>
          <option value="rejected">已驳回</option>
          <option value="reimbursed">已报销</option>
        </select>
      </div>
      <div class="filter-group">
        <select v-model="paymentFilter" @change="loadData" class="filter-select">
          <option value="all">全部付款方式</option>
          <option value="银行转账">银行转账</option>
          <option value="支付宝">支付宝</option>
          <option value="微信">微信</option>
          <option value="现金">现金</option>
          <option value="信用卡">信用卡</option>
        </select>
      </div>
      <div class="filter-group">
        <input v-model="entityFilter" @input="debounceSearch" placeholder="企业主体/部门" class="search-input filter-input-sm" />
      </div>
      <div class="filter-group">
        <input v-model="projectFilter" @input="debounceSearch" placeholder="所属项目" class="search-input filter-input-sm" />
      </div>
      <div class="filter-search">
        <input v-model="searchQuery" @input="debounceSearch" placeholder="搜索备注/发票号/供应商..." class="search-input" />
      </div>
    </div>

    <!-- Table -->
    <div class="records-table-wrapper">
      <table v-if="records.length > 0" class="records-table">
        <thead>
          <tr>
            <th class="col-voucher">凭证号</th>
            <th class="col-date">日期</th>
            <th class="col-receipt">凭证</th>
            <th class="col-type">类型</th>
            <th class="col-category">分类</th>
            <th class="col-entity">企业主体</th>
            <th class="col-project">项目</th>
            <th class="col-supplier">供应商</th>
            <th class="col-invoice">发票号</th>
            <th class="col-amount">金额</th>
            <th class="col-tax">税额</th>
            <th class="col-payment">付款方式</th>
            <th class="col-approver">审批人</th>
            <th class="col-status">审批状态</th>
            <th class="col-description">备注</th>
            <th class="col-actions">操作</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="record in records" :key="record.id" class="record-row" :class="record.type">
            <td class="col-voucher voucher-code">{{ record.voucher_number || '—' }}</td>
            <td class="col-date">{{ formatDate(record.date) }}</td>
            <td class="col-receipt">
              <div class="receipt-thumbs" v-if="record.attachments_json && record.attachments_json.length">
                <div
                  v-for="(att, idx) in record.attachments_json.slice(0, 3)"
                  :key="idx"
                  class="receipt-thumb"
                  @click="openPreview(att, record.attachments_json)"
                >
                  <img v-if="isImage(att.name)" :src="getFileUrl(att.path)" :alt="att.name" />
                  <div v-else class="pdf-thumb">
                    <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="2">
                      <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
                      <polyline points="14 2 14 8 20 8" />
                    </svg>
                  </div>
                </div>
                <span v-if="record.attachments_json.length > 3" class="more-badge">+{{ record.attachments_json.length - 3 }}</span>
              </div>
              <span v-else class="no-receipt">—</span>
            </td>
            <td class="col-type">
              <span class="type-badge" :class="record.type">
                {{ record.type === 'income' ? '收入' : '支出' }}
              </span>
            </td>
            <td class="col-category">{{ record.category }}</td>
            <td class="col-entity" :title="record.entity">{{ record.entity || '—' }}</td>
            <td class="col-project" :title="record.project">{{ record.project || '—' }}</td>
            <td class="col-supplier" :title="record.supplier">{{ record.supplier || '—' }}</td>
            <td class="col-invoice invoice-code">{{ record.invoice_number || '—' }}</td>
            <td class="col-amount" :class="record.type">
              {{ record.type === 'income' ? '+' : '-' }}¥{{ formatMoney(record.amount) }}
            </td>
            <td class="col-tax">{{ record.tax_amount ? '¥' + formatMoney(record.tax_amount) : '—' }}</td>
            <td class="col-payment">{{ record.payment_method || '—' }}</td>
            <td class="col-approver">{{ record.approver || '—' }}</td>
            <td class="col-status">
              <span class="status-badge" :class="record.status">
                {{ statusLabel(record.status) }}
              </span>
            </td>
            <td class="col-description" :title="record.description">{{ record.description || '—' }}</td>
            <td class="col-actions">
              <button @click="editRecord(record)" class="action-btn" title="编辑">✏️</button>
              <button v-if="canApprove(record)" @click="approveRecord(record, 'approved')" class="action-btn approve-btn" title="审批通过">✅</button>
              <button v-if="canApprove(record)" @click="approveRecord(record, 'rejected')" class="action-btn reject-btn" title="驳回">❌</button>
              <button v-if="canReimburse(record)" @click="approveRecord(record, 'reimbursed')" class="action-btn reimburse-btn" title="标记报销">💰</button>
              <button @click="deleteRecord(record)" class="action-btn delete-btn" title="删除">🗑️</button>
            </td>
          </tr>
        </tbody>
      </table>
      <div v-else-if="loading" class="table-loading">加载中...</div>
      <div v-else class="table-empty">
        <p>暂无记录</p>
        <button @click="openAddRecord" class="btn btn-primary btn-sm">新增第一笔</button>
      </div>
    </div>

    <!-- Pagination -->
    <div v-if="totalRecords > pageSize" class="pagination">
      <button class="btn btn-ghost btn-sm" :disabled="currentPage <= 1" @click="goToPage(currentPage - 1)">‹ 上一页</button>
      <span class="page-info">第 {{ currentPage }} / {{ totalPages }} 页，共 {{ totalRecords }} 条</span>
      <button class="btn btn-ghost btn-sm" :disabled="currentPage >= totalPages" @click="goToPage(currentPage + 1)">下一页 ›</button>
    </div>

    <!-- Record Form Modal -->
    <Teleport to="body">
      <div v-if="showRecordForm" class="modal-overlay" @click.self="closeRecordForm">
        <div class="modal-dialog modal-xl">
          <div class="modal-header">
            <h3>{{ editingRecord ? '编辑凭证' : '新增凭证' }}</h3>
            <button @click="closeRecordForm" class="modal-close">×</button>
          </div>
          <div class="modal-body">
            <!-- Voucher Number (read-only, auto-generated) -->
            <div v-if="form.voucher_number" class="form-row form-row-wide">
              <label class="form-label-wide">凭证号</label>
              <div class="voucher-display">{{ form.voucher_number }}</div>
            </div>

            <!-- Type Toggle -->
            <div class="form-row">
              <label class="form-label-wide">类型</label>
              <div class="type-toggle">
                <button class="type-btn" :class="{ active: form.type === 'expense' }" @click="form.type = 'expense'; form.category = ''">支出</button>
                <button class="type-btn" :class="{ active: form.type === 'income' }" @click="form.type = 'income'; form.category = ''">收入</button>
              </div>
            </div>

            <!-- Date & Amount Row -->
            <div class="form-row">
              <label class="form-label-wide">日期</label>
              <input type="date" v-model="form.date" class="form-input" />
              <label class="form-label-wide">金额</label>
              <input type="number" v-model.number="form.amount" step="0.01" min="0" placeholder="0.00" class="form-input amount-input" />
            </div>

            <!-- Category & Status -->
            <div class="form-row">
              <label class="form-label-wide">分类</label>
              <select v-model="form.category" class="form-input">
                <option value="">选择分类</option>
                <option v-for="cat in formCategories" :key="cat.id" :value="cat.name">{{ cat.name }}</option>
              </select>
              <label class="form-label-wide">审批状态</label>
              <select v-model="form.status" class="form-input">
                <option value="pending">待审批</option>
                <option value="approved">已审批</option>
                <option value="rejected">已驳回</option>
                <option value="reimbursed">已报销</option>
              </select>
            </div>

            <!-- Enterprise Fields Row 1 -->
            <div class="form-row">
              <label class="form-label-wide">企业主体</label>
              <input v-model="form.entity" placeholder="企业主体/部门" class="form-input" />
              <label class="form-label-wide">所属项目</label>
              <input v-model="form.project" placeholder="所属项目" class="form-input" />
            </div>

            <!-- Enterprise Fields Row 2 -->
            <div class="form-row">
              <label class="form-label-wide">供应商</label>
              <input v-model="form.supplier" placeholder="供应商/对方" class="form-input" />
              <label class="form-label-wide">发票号</label>
              <input v-model="form.invoice_number" placeholder="发票号码" class="form-input" />
            </div>

            <!-- Enterprise Fields Row 3 -->
            <div class="form-row">
              <label class="form-label-wide">税额</label>
              <input type="number" v-model.number="form.tax_amount" step="0.01" min="0" placeholder="0.00" class="form-input" />
              <label class="form-label-wide">付款方式</label>
              <select v-model="form.payment_method" class="form-input">
                <option value="">选择付款方式</option>
                <option value="银行转账">银行转账</option>
                <option value="支付宝">支付宝</option>
                <option value="微信">微信</option>
                <option value="现金">现金</option>
                <option value="信用卡">信用卡</option>
              </select>
            </div>

            <!-- Enterprise Fields Row 4 -->
            <div class="form-row">
              <label class="form-label-wide">审批人</label>
              <input v-model="form.approver" placeholder="审批人" class="form-input" />
              <label class="form-label-wide">备注</label>
              <input v-model="form.description" placeholder="可选备注说明" class="form-input" />
            </div>

            <!-- Receipt Upload Area -->
            <div class="form-row form-row-wide">
              <label class="form-label-wide">凭证附件</label>
            </div>
            <div
              class="upload-area"
              :class="{ 'upload-dragover': isDragOver }"
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
              <div v-if="!form.attachments.length" class="upload-placeholder">
                <svg viewBox="0 0 24 24" width="40" height="40" fill="none" stroke="currentColor" stroke-width="1.5" class="upload-icon">
                  <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
                  <polyline points="17 8 12 3 7 8" />
                  <line x1="12" y1="3" x2="12" y2="15" />
                </svg>
                <p class="upload-text">拖拽文件到此处，或点击选择</p>
                <p class="upload-hint">支持 PDF、PNG、JPG、GIF 格式</p>
              </div>
              <div v-else class="upload-preview-list">
                <div v-for="(att, idx) in form.attachments" :key="idx" class="upload-preview-item">
                  <div class="upload-preview-thumb">
                    <img v-if="att.type === 'image'" :src="att.preview" :alt="att.name" />
                    <div v-else class="pdf-preview-icon">
                      <svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
                        <polyline points="14 2 14 8 20 8" />
                      </svg>
                    </div>
                  </div>
                  <div class="upload-preview-info">
                    <span class="upload-preview-name">{{ att.name }}</span>
                    <span class="upload-preview-size">{{ formatFileSize(att.size) }}</span>
                  </div>
                  <button @click.stop="removeAttachment(idx)" class="upload-remove-btn" title="移除">×</button>
                </div>
                <div class="upload-add-more" @click.stop="triggerFileInput">+ 添加更多</div>
              </div>
            </div>
          </div>
          <div class="modal-footer">
            <button @click="closeRecordForm" class="btn btn-ghost">取消</button>
            <button v-if="formValid" @click="saveAsTemplate" class="btn btn-ghost" title="保存为模板">💾 存模板</button>
            <button @click="saveRecord" class="btn btn-primary" :disabled="!formValid">{{ editingRecord ? '保存' : '添加' }}</button>
          </div>
        </div>
      </div>

      <!-- Receipt Preview Modal -->
      <div v-if="showPreview" class="modal-overlay preview-overlay" @click.self="closePreview">
        <div class="preview-dialog" :class="{ 'preview-pdf': previewIsPdf }">
          <div class="modal-header">
            <h3 class="preview-title">{{ previewName }}</h3>
            <div class="preview-nav" v-if="previewGallery.length > 1">
              <button class="preview-nav-btn" @click="previewIndex = Math.max(0, previewIndex - 1)" :disabled="previewIndex === 0">‹</button>
              <span class="preview-counter">{{ previewIndex + 1 }} / {{ previewGallery.length }}</span>
              <button class="preview-nav-btn" @click="previewIndex = Math.min(previewGallery.length - 1, previewIndex + 1)" :disabled="previewIndex === previewGallery.length - 1">›</button>
            </div>
            <button @click="closePreview" class="modal-close">×</button>
          </div>
          <div class="preview-body">
            <div v-if="previewLoading" class="preview-loading">加载中...</div>
            <template v-else>
              <img
                v-if="!previewIsPdf"
                :src="previewSrc"
                :alt="previewName"
                class="preview-image"
              />
              <embed
                v-else
                :src="previewSrc"
                type="application/pdf"
                class="preview-pdf-viewer"
              />
            </template>
          </div>
        </div>
      </div>

      <!-- Category Manager Modal -->
      <div v-if="showCategoryManager" class="modal-overlay" @click.self="showCategoryManager = false">
        <div class="modal-dialog modal-lg">
          <div class="modal-header">
            <h3>分类管理</h3>
            <button @click="showCategoryManager = false" class="modal-close">×</button>
          </div>
          <div class="modal-body">
            <div class="category-manager">
              <div class="category-section">
                <h4>📈 收入分类</h4>
                <div class="category-list">
                  <div v-for="cat in incomeCategories" :key="cat.id" class="category-item">
                    <span class="cat-icon">{{ cat.icon }}</span>
                    <span class="cat-name">{{ cat.name }}</span>
                    <button v-if="!cat.builtin" @click="deleteCategory(cat.id)" class="cat-delete">×</button>
                  </div>
                </div>
              </div>
              <div class="category-section">
                <h4>📉 支出分类</h4>
                <div class="category-list">
                  <div v-for="cat in expenseCategories" :key="cat.id" class="category-item">
                    <span class="cat-icon">{{ cat.icon }}</span>
                    <span class="cat-name">{{ cat.name }}</span>
                    <button v-if="!cat.builtin" @click="deleteCategory(cat.id)" class="cat-delete">×</button>
                  </div>
                </div>
              </div>
              <div class="category-add-row">
                <select v-model="newCategory.type" class="form-input form-sm">
                  <option value="expense">支出</option>
                  <option value="income">收入</option>
                </select>
                <input v-model="newCategory.name" placeholder="分类名称" class="form-input form-sm" @keyup.enter="addNewCategory" />
                <input v-model="newCategory.icon" placeholder="Emoji图标" class="form-input form-sm form-icon" maxlength="2" />
                <button @click="addNewCategory" class="btn btn-primary btn-sm">添加</button>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Quick Templates Modal -->
      <div v-if="showTemplates" class="modal-overlay" @click.self="showTemplates = false">
        <div class="modal-dialog modal-lg">
          <div class="modal-header">
            <h3>⚡ 快捷记账模板</h3>
            <button @click="showTemplates = false" class="modal-close">×</button>
          </div>
          <div class="modal-body">
            <div v-if="templates.length === 0" class="templates-empty">
              <p>暂无模板，在新增凭证时可保存为模板</p>
            </div>
            <div v-else class="template-list">
              <div v-for="tpl in templates" :key="tpl.id" class="template-item">
                <div class="template-main">
                  <span class="template-type" :class="tpl.type">{{ tpl.type === 'income' ? '收入' : '支出' }}</span>
                  <span class="template-name">{{ tpl.name }}</span>
                  <span class="template-cat">{{ tpl.category }}</span>
                  <span class="template-amount">¥{{ formatMoney(tpl.amount) }}</span>
                  <span v-if="tpl.useCount > 0" class="template-usecount">已用 {{ tpl.useCount }} 次</span>
                </div>
                <div class="template-actions">
                  <button @click="useTemplate(tpl)" class="btn btn-primary btn-sm">使用</button>
                  <button @click="editTemplate(tpl)" class="btn btn-ghost btn-sm">编辑</button>
                  <button @click="deleteTemplateConfirm(tpl)" class="btn btn-ghost btn-sm delete-btn">删除</button>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Budget Manager Modal -->
      <div v-if="showBudgetManager" class="modal-overlay" @click.self="showBudgetManager = false">
        <div class="modal-dialog modal-lg">
          <div class="modal-header">
            <h3>🎯 预算管理</h3>
            <button @click="showBudgetManager = false" class="modal-close">×</button>
          </div>
          <div class="modal-body">
            <div class="budget-manager">
              <div class="budget-add-row">
                <select v-model="newBudget.category" class="form-input form-sm">
                  <option value="">选择分类</option>
                  <option v-for="cat in expenseCategories" :key="cat.id" :value="cat.name">{{ cat.name }}</option>
                </select>
                <input v-model.number="newBudget.amount" type="number" step="100" min="0" placeholder="预算金额" class="form-input form-sm" />
                <button @click="addNewBudget" class="btn btn-primary btn-sm">添加预算</button>
              </div>
              <div v-if="budgets.length === 0" class="budgets-empty">
                <p>暂无预算，添加分类预算后将自动监控超支情况</p>
              </div>
              <div v-else class="budget-list">
                <div v-for="b in budgets" :key="b.id" class="budget-item">
                  <span class="budget-cat">{{ b.category }}</span>
                  <span class="budget-amount">¥{{ formatMoney(b.amount) }}</span>
                  <span class="budget-period">{{ b.period === 'monthly' ? '月度' : '年度' }}</span>
                  <button @click="deleteBudgetConfirm(b)" class="btn btn-ghost btn-sm delete-btn">删除</button>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Template Editor Modal -->
      <div v-if="showTemplateEditor" class="modal-overlay" @click.self="showTemplateEditor = false">
        <div class="modal-dialog">
          <div class="modal-header">
            <h3>{{ editingTemplate ? '编辑模板' : '保存为模板' }}</h3>
            <button @click="showTemplateEditor = false" class="modal-close">×</button>
          </div>
          <div class="modal-body">
            <div class="form-row">
              <label class="form-label-wide">模板名称</label>
              <input v-model="templateForm.name" placeholder="如：月租服务器" class="form-input" />
            </div>
            <div class="form-row">
              <label class="form-label-wide">类型</label>
              <select v-model="templateForm.type" class="form-input">
                <option value="expense">支出</option>
                <option value="income">收入</option>
              </select>
              <label class="form-label-wide">分类</label>
              <select v-model="templateForm.category" class="form-input">
                <option value="">选择分类</option>
                <option v-for="cat in formCategories" :key="cat.id" :value="cat.name">{{ cat.name }}</option>
              </select>
            </div>
            <div class="form-row">
              <label class="form-label-wide">金额</label>
              <input type="number" v-model.number="templateForm.amount" step="0.01" min="0" class="form-input amount-input" />
            </div>
            <div class="form-row">
              <label class="form-label-wide">供应商</label>
              <input v-model="templateForm.supplier" placeholder="供应商" class="form-input" />
              <label class="form-label-wide">付款方式</label>
              <select v-model="templateForm.payment_method" class="form-input">
                <option value="">选择</option>
                <option value="银行转账">银行转账</option>
                <option value="支付宝">支付宝</option>
                <option value="微信">微信</option>
                <option value="现金">现金</option>
                <option value="信用卡">信用卡</option>
              </select>
            </div>
            <div class="form-row form-row-wide">
              <label class="form-label-wide">备注</label>
              <input v-model="templateForm.description" placeholder="说明" class="form-input" />
            </div>
          </div>
          <div class="modal-footer">
            <button @click="showTemplateEditor = false" class="btn btn-ghost">取消</button>
            <button @click="saveTemplate" class="btn btn-primary">保存</button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>


<script setup lang="ts">
import { useAccountingBook } from '../composables/useAccountingBook'

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


<style scoped>
.accounting-book {
  display: flex;
  flex-direction: column;
  height: 100%;
  gap: 16px;
}

.accounting-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-shrink: 0;
}

.accounting-title {
  font-size: 20px;
  font-weight: 700;
  color: var(--color-base-content);
  margin: 0;
}

.header-actions {
  display: flex;
  gap: 8px;
}

/* Stats */
.stats-row {
  display: grid;
  grid-template-columns: repeat(7, 1fr);
  gap: 12px;
  flex-shrink: 0;
}

.stat-card {
  background: var(--color-base-100);
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 10px;
  padding: 12px 16px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.stat-label {
  font-size: 12px;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  font-weight: 500;
}

.stat-value {
  font-size: 20px;
  font-weight: 700;
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
}

.stat-income .stat-value { color: var(--color-success); }
.stat-expense .stat-value { color: var(--color-error); }
.stat-balance-pos .stat-value { color: var(--color-primary); }
.stat-balance-neg .stat-value { color: var(--color-error); }
.stat-pending .stat-value { color: var(--color-warning); }
.stat-reimbursed .stat-value { color: var(--color-success); }

.stat-period-selector {
  display: flex;
  align-items: center;
  gap: 6px;
}

.period-select {
  padding: 4px 8px;
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 6px;
  background: var(--color-base-200);
  color: var(--color-base-content);
  font-size: 12px;
  cursor: pointer;
}

.period-date {
  padding: 3px 6px;
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 4px;
  background: var(--color-base-200);
  color: var(--color-base-content);
  font-size: 11px;
}

.period-sep {
  font-size: 11px;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
}

/* Category summary */
.category-summary {
  background: var(--color-base-100);
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 10px;
  padding: 12px 16px;
  flex-shrink: 0;
}

.category-summary-title {
  font-size: 12px;
  font-weight: 600;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  margin-bottom: 8px;
}

.category-bars {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.category-bar-row {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
}

.category-bar-name {
  width: 70px;
  flex-shrink: 0;
  color: var(--color-base-content);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.category-bar-track {
  flex: 1;
  height: 8px;
  background: var(--color-base-200);
  border-radius: 4px;
  overflow: hidden;
}

.category-bar-fill {
  height: 100%;
  background: linear-gradient(90deg, var(--color-error), #f97316);
  border-radius: 4px;
  transition: width 0.3s ease;
}

.category-bar-amount {
  width: 80px;
  text-align: right;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  font-family: 'JetBrains Mono', monospace;
  font-size: 11px;
}

/* Filter bar */
.filter-bar {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
  flex-wrap: wrap;
}

.filter-group {
  display: flex;
  gap: 4px;
}

.filter-chip {
  padding: 5px 12px;
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 16px;
  background: transparent;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  font-size: 12px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.filter-chip:hover { background: color-mix(in oklab, var(--color-primary) 10%, transparent); color: var(--color-primary); }
.filter-chip.active { background: var(--color-primary); color: white; border-color: var(--color-primary); }
.income-chip.active { background: var(--color-success); border-color: var(--color-success); }
.expense-chip.active { background: var(--color-error); border-color: var(--color-error); }

.filter-select {
  padding: 5px 10px;
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 6px;
  background: var(--color-base-200);
  color: var(--color-base-content);
  font-size: 12px;
}

.filter-search {
  flex: 1;
  min-width: 150px;
}

.search-input {
  width: 100%;
  padding: 5px 10px;
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 6px;
  background: var(--color-base-200);
  color: var(--color-base-content);
  font-size: 12px;
  outline: none;
}

.search-input:focus {
  border-color: var(--color-primary);
  box-shadow: 0 0 0 2px color-mix(in oklab, var(--color-primary) 10%, transparent);
}

.filter-input-sm {
  width: 120px;
  padding: 5px 8px;
  font-size: 12px;
}

/* Table */
.records-table-wrapper {
  flex: 1;
  overflow: auto;
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 10px;
}

.records-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 13px;
  min-width: 1400px;
}

.records-table thead {
  position: sticky;
  top: 0;
  z-index: 2;
}

.records-table th {
  padding: 10px 12px;
  text-align: left;
  font-size: 12px;
  font-weight: 600;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  background: var(--color-base-200);
  border-bottom: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  position: sticky;
  top: 0;
  z-index: 2;
  white-space: nowrap;
}

.records-table td {
  padding: 8px 12px;
  border-bottom: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  white-space: nowrap;
}

.records-table tbody tr:hover {
  background: color-mix(in oklab, var(--color-primary) 10%, transparent);
}

.col-amount {
  font-family: 'JetBrains Mono', monospace;
  font-weight: 600;
}

.col-amount.income { color: var(--color-success); }
.col-amount.expense { color: var(--color-error); }

.type-badge {
  padding: 2px 8px;
  border-radius: 10px;
  font-size: 11px;
  font-weight: 500;
}

.type-badge.income {
  background: rgba(16, 185, 129, 0.1);
  color: var(--color-success);
}

.type-badge.expense {
  background: rgba(239, 68, 68, 0.1);
  color: var(--color-error);
}

.status-badge {
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 11px;
  font-weight: 500;
}

.status-badge.pending {
  background: rgba(245, 158, 11, 0.15);
  color: var(--color-warning);
}

.status-badge.approved {
  background: rgba(16, 185, 129, 0.15);
  color: var(--color-success);
}

.status-badge.rejected {
  background: rgba(239, 68, 68, 0.15);
  color: var(--color-error);
}

.status-badge.reimbursed {
  background: rgba(59, 130, 246, 0.15);
  color: var(--color-primary);
}

.status-badge.confirmed {
  background: rgba(16, 185, 129, 0.15);
  color: var(--color-success);
}

.voucher-code {
  font-family: 'JetBrains Mono', monospace;
  font-size: 11px;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
}

.invoice-code {
  font-family: 'JetBrains Mono', monospace;
  font-size: 11px;
}

/* Receipt thumbnails in table */
.col-receipt {
  min-width: 100px;
}

.receipt-thumbs {
  display: flex;
  align-items: center;
  gap: 4px;
}

.receipt-thumb {
  width: 32px;
  height: 32px;
  border-radius: 4px;
  overflow: hidden;
  cursor: pointer;
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--color-base-200);
  transition: transform 0.1s ease;
}

.receipt-thumb:hover {
  transform: scale(1.1);
}

.receipt-thumb img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.pdf-thumb {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-error);
}

.more-badge {
  font-size: 10px;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  padding: 2px 4px;
}

.no-receipt {
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  font-size: 12px;
}

.action-btn {
  border: none;
  background: transparent;
  cursor: pointer;
  font-size: 13px;
  padding: 2px 4px;
  border-radius: 4px;
  transition: background 0.1s ease;
}

.action-btn:hover { background: color-mix(in oklab, var(--color-primary) 10%, transparent); }
.approve-btn:hover { background: rgba(16, 185, 129, 0.15); }
.reject-btn:hover { background: rgba(239, 68, 68, 0.15); }
.reimburse-btn:hover { background: rgba(59, 130, 246, 0.15); }
.delete-btn:hover { background: rgba(239, 68, 68, 0.15); }

.table-loading, .table-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 48px 16px;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  gap: 12px;
}

/* Pagination */
.pagination {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 16px;
  padding: 8px 0;
  flex-shrink: 0;
}

.page-info {
  font-size: 12px;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
}

/* Upload Area */
.upload-area {
  border: 2px dashed color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 10px;
  padding: 24px;
  text-align: center;
  cursor: pointer;
  transition: all 0.2s ease;
  background: var(--color-base-200);
}

.upload-area:hover,
.upload-dragover {
  border-color: var(--color-primary);
  background: color-mix(in oklab, var(--color-primary) 10%, transparent);
}

.upload-placeholder {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
}

.upload-icon {
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  opacity: 0.5;
}

.upload-text {
  font-size: 14px;
  color: var(--color-base-content);
  margin: 0;
}

.upload-hint {
  font-size: 12px;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  margin: 0;
}

.upload-preview-list {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
  gap: 12px;
}

.upload-preview-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  padding: 8px;
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 8px;
  background: var(--color-base-100);
  position: relative;
}

.upload-preview-thumb {
  width: 64px;
  height: 64px;
  border-radius: 6px;
  overflow: hidden;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--color-base-200);
}

.upload-preview-thumb img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.pdf-preview-icon {
  color: var(--color-error);
}

.upload-preview-info {
  display: flex;
  flex-direction: column;
  align-items: center;
  width: 100%;
}

.upload-preview-name {
  font-size: 11px;
  color: var(--color-base-content);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 120px;
}

.upload-preview-size {
  font-size: 10px;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
}

.upload-remove-btn {
  position: absolute;
  top: 4px;
  right: 4px;
  width: 20px;
  height: 20px;
  border: none;
  background: var(--color-error);
  color: white;
  border-radius: 50%;
  font-size: 14px;
  line-height: 1;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
}

.upload-add-more {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px;
  border: 2px dashed color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 8px;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  font-size: 13px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.upload-add-more:hover {
  border-color: var(--color-primary);
  color: var(--color-primary);
}

/* Modal */
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 10000;
}

.modal-dialog {
  background: var(--color-base-100);
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 12px;
  width: 560px;
  max-height: 80vh;
  display: flex;
  flex-direction: column;
  box-shadow: 0 16px 48px rgba(0, 0, 0, 0.3);
}

.modal-lg {
  width: 560px;
}

.modal-xl {
  width: 720px;
  max-height: 90vh;
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
}

.modal-header h3 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: var(--color-base-content);
}

.modal-close {
  width: 28px;
  height: 28px;
  border: none;
  background: transparent;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  font-size: 20px;
  cursor: pointer;
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.modal-close:hover { background: color-mix(in oklab, var(--color-primary) 10%, transparent); color: var(--color-primary); }

.modal-body {
  padding: 16px 20px;
  overflow-y: auto;
  flex: 1;
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 12px 20px;
  border-top: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
}

/* Form */
.form-row {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 12px;
}

.form-row-wide {
  flex-direction: column;
  align-items: flex-start;
  gap: 6px;
}

.form-label-wide {
  font-size: 13px;
  font-weight: 500;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
}

.form-input {
  flex: 1;
  padding: 8px 12px;
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 6px;
  background: var(--color-base-200);
  color: var(--color-base-content);
  font-size: 13px;
  outline: none;
}

.form-input:focus {
  border-color: var(--color-primary);
  box-shadow: 0 0 0 2px color-mix(in oklab, var(--color-primary) 10%, transparent);
}

.amount-input {
  font-family: 'JetBrains Mono', monospace;
  font-size: 16px;
  font-weight: 600;
}

.type-toggle {
  display: flex;
  gap: 4px;
  flex: 1;
}

.type-btn {
  flex: 1;
  padding: 8px;
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 6px;
  background: transparent;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s ease;
}

.type-btn:hover { background: color-mix(in oklab, var(--color-primary) 10%, transparent); }
.type-btn.active { background: var(--color-primary); color: white; border-color: var(--color-primary); }

.voucher-display {
  font-family: 'JetBrains Mono', monospace;
  font-size: 14px;
  color: var(--color-primary);
  font-weight: 600;
  padding: 8px 12px;
  background: color-mix(in oklab, var(--color-primary) 10%, transparent);
  border-radius: 6px;
  width: 100%;
}

/* Category Manager */
.category-manager {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.category-section h4 {
  margin: 0 0 8px;
  font-size: 14px;
  font-weight: 600;
  color: var(--color-base-content);
}

.category-list {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.category-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 10px;
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 8px;
  background: var(--color-base-200);
  font-size: 13px;
  color: var(--color-base-content);
}

.cat-icon {
  font-size: 16px;
}

.cat-delete {
  border: none;
  background: transparent;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  font-size: 16px;
  cursor: pointer;
  padding: 0 2px;
  line-height: 1;
}

.cat-delete:hover { color: var(--color-error); }

.category-add-row {
  display: flex;
  gap: 8px;
  padding-top: 12px;
  border-top: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
}

.form-sm {
  padding: 6px 8px !important;
  font-size: 12px !important;
}

.form-icon {
  width: 50px;
  flex: none;
  text-align: center;
}

/* Preview Modal */
.preview-overlay {
  align-items: flex-start;
  padding: 40px;
}

.preview-dialog {
  background: var(--color-base-100);
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 12px;
  width: 90vw;
  max-width: 900px;
  max-height: 85vh;
  display: flex;
  flex-direction: column;
  box-shadow: 0 16px 48px rgba(0, 0, 0, 0.4);
}

.preview-dialog.preview-pdf {
  max-width: 1000px;
}

.preview-title {
  font-size: 14px;
  max-width: 400px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.preview-nav {
  display: flex;
  align-items: center;
  gap: 8px;
}

.preview-nav-btn {
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  background: var(--color-base-200);
  color: var(--color-base-content);
  width: 28px;
  height: 28px;
  border-radius: 6px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 14px;
}

.preview-nav-btn:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

.preview-counter {
  font-size: 12px;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  font-family: 'JetBrains Mono', monospace;
}

.preview-body {
  padding: 16px;
  flex: 1;
  overflow: auto;
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 300px;
}

.preview-loading {
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  font-size: 14px;
}

.preview-image {
  max-width: 100%;
  max-height: 70vh;
  object-fit: contain;
  border-radius: 4px;
}

.preview-pdf-viewer {
  width: 100%;
  height: 70vh;
  border: none;
  border-radius: 4px;
}

/* Responsive */
@media (max-width: 900px) {
  .stats-row {
    grid-template-columns: repeat(3, 1fr);
  }
}

@media (max-width: 600px) {
  .stats-row {
    grid-template-columns: repeat(2, 1fr);
  }
  .filter-bar {
    flex-direction: column;
    align-items: stretch;
  }
  .filter-group {
    width: 100%;
  }
  .filter-select,
  .filter-input-sm,
  .search-input {
    width: 100%;
  }
  .modal-xl {
    width: 95vw;
  }
}

/* Trend Chart */
.trend-chart {
  background: var(--color-base-100);
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 10px;
  padding: 12px 16px;
  flex-shrink: 0;
}

.trend-chart-header {
  font-size: 12px;
  font-weight: 600;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  margin-bottom: 8px;
}

.trend-svg-container {
  width: 100%;
  overflow-x: auto;
}

.trend-svg {
  display: block;
  min-width: 500px;
}

.trend-axis-label {
  fill: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  font-size: 9px;
  font-family: 'JetBrains Mono', monospace;
}

.trend-legend {
  display: flex;
  gap: 16px;
  justify-content: center;
  margin-top: 4px;
  font-size: 11px;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
}

.trend-legend-item {
  display: flex;
  align-items: center;
  gap: 4px;
}

.trend-legend-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  display: inline-block;
}

.trend-legend-income { background: var(--color-success); }
.trend-legend-expense { background: var(--color-error); }
.trend-legend-bar { background: var(--color-error); opacity: 0.3; }

/* Budget Alerts */
.budget-alerts {
  background: var(--color-base-100);
  border: 1px solid var(--color-warning);
  border-radius: 10px;
  padding: 12px 16px;
  flex-shrink: 0;
}

.budget-alerts-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.budget-alerts-title {
  font-size: 12px;
  font-weight: 600;
  color: var(--color-warning);
}

.budget-alert-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.budget-alert-item {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
}

.budget-alert-item.budget-over {
  background: rgba(239, 68, 68, 0.05);
  border-radius: 4px;
  padding: 4px 8px;
  margin: -4px -8px;
}

.budget-alert-cat {
  width: 70px;
  flex-shrink: 0;
  color: var(--color-base-content);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.budget-alert-track {
  flex: 1;
  height: 8px;
  background: var(--color-base-200);
  border-radius: 4px;
  overflow: hidden;
}

.budget-alert-fill {
  height: 100%;
  background: linear-gradient(90deg, var(--color-warning), var(--color-error));
  border-radius: 4px;
  transition: width 0.3s ease;
}

.budget-alert-pct {
  width: 36px;
  text-align: right;
  font-weight: 600;
  font-family: 'JetBrains Mono', monospace;
  font-size: 11px;
  color: var(--color-warning);
}

.budget-alert-item.budget-over .budget-alert-pct {
  color: var(--color-error);
}

.budget-alert-amount {
  width: 120px;
  text-align: right;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  font-family: 'JetBrains Mono', monospace;
  font-size: 11px;
}

/* Stat MoM */
.stat-mom {
  font-size: 10px;
  font-family: 'JetBrains Mono', monospace;
  margin-top: 2px;
}

.stat-mom.mom-up { color: var(--color-error); }
.stat-mom.mom-down { color: var(--color-success); }

.stat-daily .stat-value { color: var(--color-warning); }

/* Templates */
.template-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.template-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 14px;
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 8px;
  background: var(--color-base-200);
  transition: background 0.1s ease;
}

.template-item:hover { background: var(--color-base-100); }

.template-main {
  display: flex;
  align-items: center;
  gap: 10px;
  flex: 1;
  min-width: 0;
}

.template-type {
  padding: 2px 8px;
  border-radius: 10px;
  font-size: 11px;
  font-weight: 500;
  flex-shrink: 0;
}

.template-type.income { background: rgba(16, 185, 129, 0.1); color: var(--color-success); }
.template-type.expense { background: rgba(239, 68, 68, 0.1); color: var(--color-error); }

.template-name { font-weight: 600; color: var(--color-base-content); }
.template-cat { color: color-mix(in oklab, var(--color-base-content) 60%, transparent); font-size: 12px; }
.template-amount { font-family: 'JetBrains Mono', monospace; font-weight: 600; color: var(--color-base-content); }
.template-usecount { font-size: 10px; color: color-mix(in oklab, var(--color-base-content) 60%, transparent); }

.template-actions {
  display: flex;
  gap: 4px;
  flex-shrink: 0;
}

.templates-empty, .budgets-empty {
  text-align: center;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  padding: 24px;
  font-size: 13px;
}

/* Budget Manager */
.budget-manager {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.budget-add-row {
  display: flex;
  gap: 8px;
}

.budget-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.budget-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 12px;
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 8px;
  background: var(--color-base-200);
}

.budget-cat { flex: 1; font-weight: 500; }
.budget-amount { font-family: 'JetBrains Mono', monospace; font-weight: 600; }
.budget-period { font-size: 11px; color: color-mix(in oklab, var(--color-base-content) 60%, transparent); }

.btn-xs {
  padding: 2px 6px;
  font-size: 11px;
}
</style>

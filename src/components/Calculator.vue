<template>
  <div class="calculator-container">
    <div class="calculator-layout">
      <!-- 主计算器区域 -->
      <div class="calculator-main">
        <div class="calc-display">
          <div class="calc-expression">{{ expression || '0' }}</div>
          <div class="calc-result">{{ result || '' }}</div>
        </div>
        
        <div class="calc-buttons">
          <button class="calc-btn func" @click="clear">AC</button>
          <button class="calc-btn func" @click="toggleSign">±</button>
          <button class="calc-btn func" @click="percent">%</button>
          <button class="calc-btn op" @click="appendOp('/')">÷</button>
          
          <button class="calc-btn num" @click="appendNum('7')">7</button>
          <button class="calc-btn num" @click="appendNum('8')">8</button>
          <button class="calc-btn num" @click="appendNum('9')">9</button>
          <button class="calc-btn op" @click="appendOp('*')">×</button>
          
          <button class="calc-btn num" @click="appendNum('4')">4</button>
          <button class="calc-btn num" @click="appendNum('5')">5</button>
          <button class="calc-btn num" @click="appendNum('6')">6</button>
          <button class="calc-btn op" @click="appendOp('-')">−</button>
          
          <button class="calc-btn num" @click="appendNum('1')">1</button>
          <button class="calc-btn num" @click="appendNum('2')">2</button>
          <button class="calc-btn num" @click="appendNum('3')">3</button>
          <button class="calc-btn op" @click="appendOp('+')">+</button>
          
          <button class="calc-btn num zero" @click="appendNum('0')">0</button>
          <button class="calc-btn num" @click="appendDot">.</button>
          <button class="calc-btn eq" @click="calculate">=</button>
        </div>
      </div>

      <!-- 历史记录区域 -->
      <div class="calc-history">
        <div class="history-header">
          <span>历史记录</span>
          <button class="clear-history-btn" @click="clearHistory" v-if="history.length > 0">清空</button>
        </div>
        <div class="history-list">
          <div v-if="history.length === 0" class="history-empty">暂无历史记录</div>
          <div 
            v-for="item in history" 
            :key="item.id" 
            class="history-item"
            @click="loadFromHistory(item)"
          >
            <div class="history-expr">{{ item.expression }}</div>
            <div class="history-res">= {{ item.result }}</div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { getTauriAPI } from '../utils/tauri-api'

const expression = ref('')
const result = ref('')
const history = ref<Array<{ id: string; expression: string; result: string; createdAt: string }>>([])

function appendNum(n: string) {
  expression.value += n
}

function appendOp(op: string) {
  if (expression.value && !isNaN(Number(expression.value.slice(-1)))) {
    expression.value += ` ${op} `
  }
}

function appendDot() {
  const parts = expression.value.split(/[\s\+\-\*\/]+/)
  const last = parts[parts.length - 1]
  if (!last.includes('.')) {
    expression.value += last ? '.' : '0.'
  }
}

function clear() {
  expression.value = ''
  result.value = ''
}

function toggleSign() {
  if (expression.value) {
    if (expression.value.startsWith('-')) {
      expression.value = expression.value.slice(1)
    } else {
      expression.value = '-' + expression.value
    }
  }
}

function percent() {
  if (expression.value) {
    try {
      const val = eval(expression.value)
      expression.value = String(val / 100)
    } catch {}
  }
}

async function calculate() {
  if (!expression.value) return
  try {
    // 安全计算，替换显示符号
    const expr = expression.value.replace(/×/g, '*').replace(/÷/g, '/').replace(/−/g, '-')
    const val = Function('"use strict"; return (' + expr + ')')()
    result.value = String(Number(val.toFixed(8)))
    
    // 保存历史
    try {
      const history = JSON.parse(localStorage.getItem("calc_history") || "[]")
      history.unshift({ expression: expression.value, result: result.value, time: Date.now() })
      localStorage.setItem("calc_history", JSON.stringify(history.slice(0, 100)))
      loadHistory()
    } catch {}
    expression.value = result.value
  } catch {
    result.value = 'Error'
  }
}

async function loadHistory() {
  history.value = await getTauriAPI().getCalculatorHistory(100)
}

function loadFromHistory(item: { expression: string; result: string }) {
  expression.value = item.result
  result.value = ''
}

async function clearHistory() {
  localStorage.removeItem("calc_history")
  history.value = []
}

onMounted(() => {
    console.log("[components/Calculator.vue] mounted")
  loadHistory()
})
</script>

<style scoped>
.calculator-container {
  padding: 20px;
  height: 100%;
}

.calculator-layout {
  display: flex;
  gap: 20px;
  height: 100%;
  max-height: 600px;
}

.calculator-main {
  flex: 0 0 320px;
  display: flex;
  flex-direction: column;
  background: var(--bg-card, #fff);
  border-radius: 16px;
  overflow: hidden;
  box-shadow: 0 4px 12px rgba(0,0,0,0.1);
}

.calc-display {
  padding: 24px;
  text-align: right;
  background: var(--bg-secondary, #f5f5f5);
  min-height: 100px;
  display: flex;
  flex-direction: column;
  justify-content: flex-end;
}

.calc-expression {
  font-size: 18px;
  color: var(--text-secondary, #666);
  min-height: 24px;
  word-break: break-all;
}

.calc-result {
  font-size: 32px;
  font-weight: 600;
  color: var(--text-primary, #333);
  min-height: 40px;
}

.calc-buttons {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 1px;
  background: color-mix(in oklab, var(--color-base-content) 10%, transparent);
}

.calc-btn {
  border: none;
  padding: 20px;
  font-size: 20px;
  cursor: pointer;
  background: var(--bg-card, #fff);
  transition: background 0.1s;
}

.calc-btn:hover {
  background: var(--bg-hover, #f0f0f0);
}

.calc-btn:active {
  background: var(--bg-active, #e0e0e0);
}

.calc-btn.func {
  background: var(--bg-secondary, #f5f5f5);
  color: var(--primary, #6366f1);
}

.calc-btn.op {
  background: var(--bg-secondary, #f5f5f5);
  color: var(--primary, #6366f1);
  font-size: 24px;
}

.calc-btn.eq {
  background: var(--primary, #6366f1);
  color: white;
  grid-column: span 2;
}

.calc-btn.eq:hover {
  background: var(--primary-dark, #4f46e5);
}

.calc-btn.zero {
  grid-column: span 2;
}

.calc-history {
  flex: 1;
  background: var(--bg-card, #fff);
  border-radius: 16px;
  padding: 16px;
  box-shadow: 0 4px 12px rgba(0,0,0,0.1);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.history-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
  font-weight: 600;
  color: var(--text-primary, #333);
}

.clear-history-btn {
  border: none;
  background: transparent;
  color: var(--danger, #ef4444);
  cursor: pointer;
  font-size: 13px;
  padding: 4px 8px;
  border-radius: 4px;
}

.clear-history-btn:hover {
  background: var(--danger-light, #fee2e2);
}

.history-list {
  flex: 1;
  overflow-y: auto;
}

.history-item {
  padding: 10px 12px;
  border-radius: 8px;
  cursor: pointer;
  margin-bottom: 6px;
  transition: background 0.1s;
}

.history-item:hover {
  background: var(--bg-hover, #f0f0f0);
}

.history-expr {
  font-size: 14px;
  color: var(--text-secondary, #666);
}

.history-res {
  font-size: 18px;
  font-weight: 600;
  color: var(--text-primary, #333);
}

.history-empty {
  text-align: center;
  color: var(--text-muted, #999);
  padding: 40px 0;
}

.dark .calc-display { background: #1e1e1e; }
.dark .calc-result { color: #fff; }
.dark .calc-buttons { background: #333; }
.dark .calc-btn { background: #2a2a2a; color: #fff; }
.dark .calc-btn:hover { background: #3a3a3a; }
.dark .calc-btn.func, .dark .calc-btn.op { background: #333; color: #a78bfa; }
.dark .calc-btn.eq { background: #6366f1; }
.dark .calc-history { background: #2a2a2a; }
.dark .history-item:hover { background: #3a3a3a; }
.dark .history-res { color: #fff; }
</style>

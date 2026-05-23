<template>
  <div class="p-5 h-full">
    <div class="flex gap-5 h-full max-h-[600px]">
      <!-- 主计算器区域 -->
      <div class="flex-[0_0_320px] flex flex-col rounded-2xl bg-base-100 shadow-md overflow-hidden">
        <div class="p-6 text-right bg-base-200 min-h-[100px] flex flex-col justify-end">
          <div class="text-lg text-base-content/60 min-h-[24px] break-all">{{ expression || '0' }}</div>
          <div class="text-3xl font-semibold text-base-content min-h-[40px]">{{ result || '' }}</div>
        </div>

        <div class="grid grid-cols-4 gap-px bg-base-content/10">
          <button class="border-none p-5 text-xl cursor-pointer bg-base-200 text-primary transition-colors duration-100 hover:bg-base-300 active:bg-base-300" @click="clear">AC</button>
          <button class="border-none p-5 text-xl cursor-pointer bg-base-200 text-primary transition-colors duration-100 hover:bg-base-300 active:bg-base-300" @click="toggleSign">±</button>
          <button class="border-none p-5 text-xl cursor-pointer bg-base-200 text-primary transition-colors duration-100 hover:bg-base-300 active:bg-base-300" @click="percent">%</button>
          <button class="border-none p-5 text-xl cursor-pointer bg-base-200 text-primary text-2xl transition-colors duration-100 hover:bg-base-300 active:bg-base-300" @click="appendOp('/')">÷</button>

          <button class="border-none p-5 text-xl cursor-pointer bg-base-100 transition-colors duration-100 hover:bg-base-200 active:bg-base-300" @click="appendNum('7')">7</button>
          <button class="border-none p-5 text-xl cursor-pointer bg-base-100 transition-colors duration-100 hover:bg-base-200 active:bg-base-300" @click="appendNum('8')">8</button>
          <button class="border-none p-5 text-xl cursor-pointer bg-base-100 transition-colors duration-100 hover:bg-base-200 active:bg-base-300" @click="appendNum('9')">9</button>
          <button class="border-none p-5 text-xl cursor-pointer bg-base-200 text-primary text-2xl transition-colors duration-100 hover:bg-base-300 active:bg-base-300" @click="appendOp('*')">×</button>

          <button class="border-none p-5 text-xl cursor-pointer bg-base-100 transition-colors duration-100 hover:bg-base-200 active:bg-base-300" @click="appendNum('4')">4</button>
          <button class="border-none p-5 text-xl cursor-pointer bg-base-100 transition-colors duration-100 hover:bg-base-200 active:bg-base-300" @click="appendNum('5')">5</button>
          <button class="border-none p-5 text-xl cursor-pointer bg-base-100 transition-colors duration-100 hover:bg-base-200 active:bg-base-300" @click="appendNum('6')">6</button>
          <button class="border-none p-5 text-xl cursor-pointer bg-base-200 text-primary text-2xl transition-colors duration-100 hover:bg-base-300 active:bg-base-300" @click="appendOp('-')">−</button>

          <button class="border-none p-5 text-xl cursor-pointer bg-base-100 transition-colors duration-100 hover:bg-base-200 active:bg-base-300" @click="appendNum('1')">1</button>
          <button class="border-none p-5 text-xl cursor-pointer bg-base-100 transition-colors duration-100 hover:bg-base-200 active:bg-base-300" @click="appendNum('2')">2</button>
          <button class="border-none p-5 text-xl cursor-pointer bg-base-100 transition-colors duration-100 hover:bg-base-200 active:bg-base-300" @click="appendNum('3')">3</button>
          <button class="border-none p-5 text-xl cursor-pointer bg-base-200 text-primary text-2xl transition-colors duration-100 hover:bg-base-300 active:bg-base-300" @click="appendOp('+')">+</button>

          <button class="border-none p-5 text-xl cursor-pointer bg-base-100 transition-colors duration-100 hover:bg-base-200 active:bg-base-300 col-span-2" @click="appendNum('0')">0</button>
          <button class="border-none p-5 text-xl cursor-pointer bg-base-100 transition-colors duration-100 hover:bg-base-200 active:bg-base-300" @click="appendDot">.</button>
          <button class="border-none p-5 text-xl cursor-pointer bg-primary text-white col-span-2 transition-colors duration-100 hover:bg-primary-focus" @click="calculate">=</button>
        </div>
      </div>

      <!-- 历史记录区域 -->
      <div class="flex-1 rounded-2xl bg-base-100 p-4 shadow-md flex flex-col overflow-hidden">
        <div class="flex justify-between items-center mb-3 font-semibold text-base-content">
          <span>历史记录</span>
          <button v-if="history.length > 0" class="btn btn-ghost btn-xs text-error hover:bg-error/10" @click="clearHistory">清空</button>
        </div>
        <div class="flex-1 overflow-y-auto">
          <div v-if="history.length === 0" class="text-center text-base-content/40 py-10">暂无历史记录</div>
          <div
            v-for="item in history"
            :key="item.id"
            class="px-3 py-2.5 rounded-lg cursor-pointer mb-1.5 transition-colors duration-100 hover:bg-base-200"
            @click="loadFromHistory(item)"
          >
            <div class="text-sm text-base-content/60">{{ item.expression }}</div>
            <div class="text-lg font-semibold text-base-content">= {{ item.result }}</div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { getTauriAPI } from '../../utils/tauri-api'

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
  const parts = expression.value.split(/[\s+\-*/]+/)
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
  if (!expression.value) {return}
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

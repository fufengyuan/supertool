<template>
  <ToolPage
    icon="calculator"
    name="计算器"
    description="基本运算与科学计算（三角、对数、阶乘等），带历史记录"
    @back="$emit('back')"
  >
    <!-- 显示屏 -->
    <div class="bg-base-300/70 border border-base-content/10 rounded-xl p-4 mb-4">
      <div class="text-right">
        <div v-if="expression" class="text-xs text-base-content/60 mb-1 font-mono truncate">{{ expression }}</div>
        <div class="text-3xl font-mono font-bold text-base-content break-all">{{ display }}</div>
      </div>
    </div>

    <!-- 模式切换 -->
    <div class="flex gap-2 mb-4">
      <button
        class="btn btn-sm"
        :class="mode === 'basic' ? 'btn-primary' : 'btn-ghost'"
        @click="mode = 'basic'"
      >基本</button>
      <button
        class="btn btn-sm"
        :class="mode === 'scientific' ? 'btn-primary' : 'btn-ghost'"
        @click="mode = 'scientific'"
      >科学</button>
    </div>

    <!-- 基本计算器按钮 -->
    <div class="grid grid-cols-4 gap-2">
      <button class="btn btn-ghost btn-lg" @click="clear">C</button>
      <button class="btn btn-ghost btn-lg" @click="toggleSign">±</button>
      <button class="btn btn-ghost btn-lg" @click="percentage">%</button>
      <button class="btn btn-primary btn-lg" @click="inputOperator('/')">÷</button>

      <button class="btn btn-outline btn-lg" @click="inputNumber('7')">7</button>
      <button class="btn btn-outline btn-lg" @click="inputNumber('8')">8</button>
      <button class="btn btn-outline btn-lg" @click="inputNumber('9')">9</button>
      <button class="btn btn-primary btn-lg" @click="inputOperator('*')">×</button>

      <button class="btn btn-outline btn-lg" @click="inputNumber('4')">4</button>
      <button class="btn btn-outline btn-lg" @click="inputNumber('5')">5</button>
      <button class="btn btn-outline btn-lg" @click="inputNumber('6')">6</button>
      <button class="btn btn-primary btn-lg" @click="inputOperator('-')">−</button>

      <button class="btn btn-outline btn-lg" @click="inputNumber('1')">1</button>
      <button class="btn btn-outline btn-lg" @click="inputNumber('2')">2</button>
      <button class="btn btn-outline btn-lg" @click="inputNumber('3')">3</button>
      <button class="btn btn-primary btn-lg" @click="inputOperator('+')">+</button>

      <button class="btn btn-outline btn-lg col-span-2" @click="inputNumber('0')">0</button>
      <button class="btn btn-outline btn-lg" @click="inputDecimal">.</button>
      <button class="btn btn-accent btn-lg" @click="calculate">=</button>
    </div>

    <!-- 科学计算按钮 -->
    <div v-if="mode === 'scientific'" class="grid grid-cols-5 gap-2 mt-3">
      <button class="btn btn-ghost btn-sm" @click="scientificFunc('sin')">sin</button>
      <button class="btn btn-ghost btn-sm" @click="scientificFunc('cos')">cos</button>
      <button class="btn btn-ghost btn-sm" @click="scientificFunc('tan')">tan</button>
      <button class="btn btn-ghost btn-sm" @click="scientificFunc('log')">log</button>
      <button class="btn btn-ghost btn-sm" @click="scientificFunc('ln')">ln</button>

      <button class="btn btn-ghost btn-sm" @click="scientificFunc('asin')">sin⁻¹</button>
      <button class="btn btn-ghost btn-sm" @click="scientificFunc('acos')">cos⁻¹</button>
      <button class="btn btn-ghost btn-sm" @click="scientificFunc('atan')">tan⁻¹</button>
      <button class="btn btn-ghost btn-sm" @click="scientificFunc('sqrt')">√</button>
      <button class="btn btn-ghost btn-sm" @click="scientificFunc('square')">x²</button>

      <button class="btn btn-ghost btn-sm" @click="scientificFunc('cube')">x³</button>
      <button class="btn btn-ghost btn-sm" @click="scientificFunc('pow')">xʸ</button>
      <button class="btn btn-ghost btn-sm" @click="scientificFunc('exp')">eˣ</button>
      <button class="btn btn-ghost btn-sm" @click="inputConstant('pi')">π</button>
      <button class="btn btn-ghost btn-sm" @click="inputConstant('e')">e</button>

      <button class="btn btn-ghost btn-sm" @click="scientificFunc('abs')">|x|</button>
      <button class="btn btn-ghost btn-sm" @click="scientificFunc('fact')">n!</button>
      <button class="btn btn-ghost btn-sm" @click="scientificFunc('inv')">1/x</button>
      <button class="btn btn-ghost btn-sm" @click="inputParenthesis('(')">(</button>
      <button class="btn btn-ghost btn-sm" @click="inputParenthesis(')')">)</button>
    </div>

    <!-- 历史记录 -->
    <div class="bg-base-100 border border-base-content/10 rounded-xl p-4">
      <div class="flex items-center justify-between mb-2">
        <h4 class="text-xs font-semibold text-base-content/70 flex items-center gap-1.5"><SvgIcon name="clock" size="12" /> 历史记录</h4>
        <button class="btn btn-ghost btn-xs" @click="clearHistory" :disabled="history.length === 0">清除</button>
      </div>
      <div class="max-h-40 overflow-y-auto flex flex-col gap-0.5">
        <div
          v-for="(item, index) in history"
          :key="index"
          class="flex justify-between items-center py-1.5 px-2.5 bg-base-200/50 border border-base-content/10 rounded-lg cursor-pointer text-sm font-mono hover:border-primary/40 transition-colors"
          @click="loadHistory(item)"
        >
          <span class="text-base-content/60 truncate">{{ item.expression }} =</span>
          <span class="text-primary font-semibold">{{ item.result }}</span>
        </div>
        <div v-if="history.length === 0" class="text-center text-base-content/40 text-xs py-4">暂无历史记录</div>
      </div>
    </div>
  </ToolPage>
</template>

<script setup lang="ts">
import SvgIcon from '@/components/ui/SvgIcon.vue'
import ToolPage from '../components/ToolPage.vue'
import { ref, watch } from 'vue'

defineEmits<{ back: [] }>()

const display = ref('0')
const expression = ref('')
const currentNumber = ref('')
const currentOperator = ref('')
const previousNumber = ref('')
const mode = ref<'basic' | 'scientific'>('basic')
const waitingForOperand = ref(false)
const history = ref<{ expression: string; result: string }[]>([])
const parenthesisStack = ref<string[]>([])

function inputNumber(num: string) {
  if (waitingForOperand.value) {
    currentNumber.value = num
    waitingForOperand.value = false
  } else {
    currentNumber.value = currentNumber.value === '0' ? num : currentNumber.value + num
  }
  display.value = currentNumber.value
}

function inputDecimal() {
  if (waitingForOperand.value) {
    currentNumber.value = '0.'
    waitingForOperand.value = false
  } else if (!currentNumber.value.includes('.')) {
    currentNumber.value += '.'
  }
  display.value = currentNumber.value
}

function inputOperator(op: string) {
  if (currentOperator.value && !waitingForOperand.value) {
    calculateIntermediate()
  }
  previousNumber.value = currentNumber.value || display.value
  currentOperator.value = op
  waitingForOperand.value = true
  updateExpression()
}

function updateExpression() {
  const opDisplay: Record<string, string> = { '+': '+', '-': '−', '*': '×', '/': '÷' }
  expression.value = `${previousNumber.value} ${opDisplay[currentOperator.value] || currentOperator.value}`
}

function calculateIntermediate() {
  const prev = parseFloat(previousNumber.value)
  const curr = parseFloat(currentNumber.value)
  let result = 0
  switch (currentOperator.value) {
    case '+': result = prev + curr; break
    case '-': result = prev - curr; break
    case '*': result = prev * curr; break
    case '/': result = curr !== 0 ? prev / curr : 0; break
  }
  currentNumber.value = formatResult(result)
  display.value = currentNumber.value
  previousNumber.value = ''
  currentOperator.value = ''
}

function calculate() {
  if (!currentOperator.value && !expression.value) {return}
  
  const prev = parseFloat(previousNumber.value || display.value)
  const curr = parseFloat(currentNumber.value || display.value)
  let result = 0
  switch (currentOperator.value) {
    case '+': result = prev + curr; break
    case '-': result = prev - curr; break
    case '*': result = prev * curr; break
    case '/': result = curr !== 0 ? prev / curr : 0; break
    default: result = curr
  }
  
  const opDisplay: Record<string, string> = { '+': '+', '-': '−', '*': '×', '/': '÷' }
  const fullExpression = `${previousNumber.value || display.value} ${opDisplay[currentOperator.value] || ''} ${currentNumber.value || ''}`
  
  result = parseFloat(formatResult(result))
  display.value = formatResult(result)
  
  // 添加到历史
  if (fullExpression.trim()) {
    history.value.unshift({
      expression: fullExpression.trim(),
      result: display.value
    })
    if (history.value.length > 20) {history.value.pop()}
  }
  
  expression.value = ''
  currentNumber.value = ''
  currentOperator.value = ''
  previousNumber.value = ''
  waitingForOperand.value = true
}

function clear() {
  display.value = '0'
  expression.value = ''
  currentNumber.value = ''
  currentOperator.value = ''
  previousNumber.value = ''
  waitingForOperand.value = false
  parenthesisStack.value = []
}

function toggleSign() {
  const num = parseFloat(display.value)
  display.value = formatResult(-num)
  currentNumber.value = display.value
}

function percentage() {
  const num = parseFloat(display.value)
  display.value = formatResult(num / 100)
  currentNumber.value = display.value
}

function scientificFunc(func: string) {
  const num = parseFloat(display.value)
  let result = 0
  
  try {
    switch (func) {
      case 'sin': result = Math.sin(num); break
      case 'cos': result = Math.cos(num); break
      case 'tan': result = Math.tan(num); break
      case 'asin': result = Math.asin(num); break
      case 'acos': result = Math.acos(num); break
      case 'atan': result = Math.atan(num); break
      case 'log': result = Math.log10(num); break
      case 'ln': result = Math.log(num); break
      case 'sqrt': result = Math.sqrt(num); break
      case 'square': result = num * num; break
      case 'cube': result = num * num * num; break
      case 'exp': result = Math.exp(num); break
      case 'abs': result = Math.abs(num); break
      case 'inv': result = num !== 0 ? 1 / num : 0; break
      case 'fact': result = factorial(Math.floor(num)); break
      case 'pow': 
        previousNumber.value = display.value
        currentOperator.value = '^'
        waitingForOperand.value = true
        expression.value = `${num} ^`
        return
    }
    
    display.value = formatResult(result)
    currentNumber.value = display.value
    waitingForOperand.value = true
  } catch {
    display.value = 'Error'
  }
}

function factorial(n: number): number {
  if (n < 0) {return NaN}
  if (n === 0 || n === 1) {return 1}
  if (n > 170) {return Infinity}
  let result = 1
  for (let i = 2; i <= n; i++) {result *= i}
  return result
}

function inputConstant(constant: string) {
  const value = constant === 'pi' ? Math.PI : Math.E
  display.value = formatResult(value)
  currentNumber.value = display.value
  waitingForOperand.value = false
}

function inputParenthesis(p: string) {
  if (p === '(') {
    parenthesisStack.value.push('(')
    expression.value += '('
    waitingForOperand.value = true
  } else if (p === ')' && parenthesisStack.value.length > 0) {
    parenthesisStack.value.pop()
    expression.value += `${currentNumber.value})`
    waitingForOperand.value = true
  }
}

function formatResult(num: number): string {
  if (isNaN(num)) {return 'Error'}
  if (!isFinite(num)) {return num > 0 ? '∞' : '-∞'}
  
  const abs = Math.abs(num)
  if (abs >= 1e15 || (abs < 1e-10 && abs !== 0)) {
    return num.toExponential(8)
  }
  
  // 保留合理精度
  const rounded = parseFloat(num.toPrecision(12))
  return rounded.toString()
}

function clearHistory() {
  history.value = []
}

function loadHistory(item: { expression: string; result: string }) {
  display.value = item.result
  currentNumber.value = item.result
  waitingForOperand.value = true
}

// 键盘支持
watch(mode, () => {
  // 模式切换时不做额外处理
})

// 全局键盘事件（可选，需要在组件挂载时添加）
function handleKeydown(e: KeyboardEvent) {
  if (e.key >= '0' && e.key <= '9') {inputNumber(e.key)}
  else if (e.key === '.') {inputDecimal()}
  else if (e.key === '+') {inputOperator('+')}
  else if (e.key === '-') {inputOperator('-')}
  else if (e.key === '*') {inputOperator('*')}
  else if (e.key === '/') {inputOperator('/')}
  else if (e.key === 'Enter' || e.key === '=') {calculate()}
  else if (e.key === 'Escape' || e.key === 'c' || e.key === 'C') {clear()}
  else if (e.key === 'Backspace') {
    if (currentNumber.value.length > 1) {
      currentNumber.value = currentNumber.value.slice(0, -1)
      display.value = currentNumber.value
    } else {
      currentNumber.value = ''
      display.value = '0'
    }
  }
}

// 挂载时添加键盘监听
if (typeof window !== 'undefined') {
  window.addEventListener('keydown', handleKeydown)
}
</script>
/**
 * 计算器核心逻辑
 */

export interface CalculatorState {
  display: string
  expression: string
  currentNumber: string
  currentOperator: string
  previousNumber: string
  waitingForOperand: boolean
}

export function createInitialState(): CalculatorState {
  return {
    display: '0',
    expression: '',
    currentNumber: '',
    currentOperator: '',
    previousNumber: '',
    waitingForOperand: false,
  }
}

export function formatResult(num: number): string {
  if (isNaN(num)) {return 'Error'}
  if (!isFinite(num)) {return num > 0 ? '∞' : '-∞'}

  const abs = Math.abs(num)
  if (abs >= 1e15 || (abs < 1e-10 && abs !== 0)) {
    return num.toExponential(8)
  }

  const rounded = parseFloat(num.toPrecision(12))
  return rounded.toString()
}

export function factorial(n: number): number {
  if (n < 0) {return NaN}
  if (n === 0 || n === 1) {return 1}
  if (n > 170) {return Infinity}
  let result = 1
  for (let i = 2; i <= n; i++) {result *= i}
  return result
}

export function calculateBasic(prev: number, curr: number, operator: string): number {
  switch (operator) {
    case '+': return prev + curr
    case '-': return prev - curr
    case '*': return prev * curr
    case '/': return curr !== 0 ? prev / curr : NaN
    default: return curr
  }
}

export function scientificFunc(num: number, func: string): number {
  switch (func) {
    case 'sin': return Math.sin(num)
    case 'cos': return Math.cos(num)
    case 'tan': return Math.tan(num)
    case 'asin': return Math.asin(num)
    case 'acos': return Math.acos(num)
    case 'atan': return Math.atan(num)
    case 'log': return Math.log10(num)
    case 'ln': return Math.log(num)
    case 'sqrt': return Math.sqrt(num)
    case 'square': return num * num
    case 'cube': return num * num * num
    case 'exp': return Math.exp(num)
    case 'abs': return Math.abs(num)
    case 'inv': return num !== 0 ? 1 / num : NaN
    case 'fact': return factorial(Math.floor(num))
    default: return NaN
  }
}

export function getConstantValue(constant: string): number {
  switch (constant) {
    case 'pi': return Math.PI
    case 'e': return Math.E
    default: return NaN
  }
}

export function inputNumber(state: CalculatorState, num: string): CalculatorState {
  if (state.waitingForOperand) {
    return {
      ...state,
      currentNumber: num,
      display: num,
      waitingForOperand: false,
    }
  }
  const newNumber = state.currentNumber === '0' ? num : state.currentNumber + num
  return {
    ...state,
    currentNumber: newNumber,
    display: newNumber,
  }
}

export function inputDecimal(state: CalculatorState): CalculatorState {
  if (state.waitingForOperand) {
    return {
      ...state,
      currentNumber: '0.',
      display: '0.',
      waitingForOperand: false,
    }
  }
  if (state.currentNumber.includes('.')) {
    return state
  }
  const newNumber = state.currentNumber + '.'
  return {
    ...state,
    currentNumber: newNumber,
    display: newNumber,
  }
}

export function inputOperator(state: CalculatorState, op: string): CalculatorState {
  if (state.currentOperator && !state.waitingForOperand) {
    const prev = parseFloat(state.previousNumber)
    const curr = parseFloat(state.currentNumber)
    const result = calculateBasic(prev, curr, state.currentOperator)
    const formattedResult = formatResult(result)
    return {
      ...state,
      currentNumber: formattedResult,
      display: formattedResult,
      previousNumber: formattedResult,
      currentOperator: op,
      waitingForOperand: true,
    }
  }
  return {
    ...state,
    previousNumber: state.currentNumber || state.display,
    currentOperator: op,
    waitingForOperand: true,
  }
}

export function calculate(state: CalculatorState): { result: CalculatorState; expression: string; resultValue: string } {
  if (!state.currentOperator && !state.expression) {
    return { result: state, expression: '', resultValue: state.display }
  }

  const prev = parseFloat(state.previousNumber || state.display)
  const curr = parseFloat(state.currentNumber || state.display)
  const calculated = calculateBasic(prev, curr, state.currentOperator)
  const formattedResult = formatResult(calculated)

  const opDisplay: Record<string, string> = { '+': '+', '-': '−', '*': '×', '/': '÷' }
  const fullExpression = `${state.previousNumber || state.display} ${opDisplay[state.currentOperator] || ''} ${state.currentNumber || ''}`

  const newState: CalculatorState = {
    display: formattedResult,
    expression: '',
    currentNumber: '',
    currentOperator: '',
    previousNumber: '',
    waitingForOperand: true,
  }

  return { result: newState, expression: fullExpression.trim(), resultValue: formattedResult }
}

export function clear(): CalculatorState {
  return createInitialState()
}

export function toggleSign(state: CalculatorState): CalculatorState {
  const num = parseFloat(state.display)
  const newDisplay = formatResult(-num)
  return {
    ...state,
    display: newDisplay,
    currentNumber: newDisplay,
  }
}

export function percentage(state: CalculatorState): CalculatorState {
  const num = parseFloat(state.display)
  const newDisplay = formatResult(num / 100)
  return {
    ...state,
    display: newDisplay,
    currentNumber: newDisplay,
  }
}
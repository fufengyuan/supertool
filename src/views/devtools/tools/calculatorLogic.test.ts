import { describe, it, expect } from 'vitest'
import {
  createInitialState,
  formatResult,
  factorial,
  calculateBasic,
  scientificFunc,
  getConstantValue,
  inputNumber,
  inputDecimal,
  inputOperator,
  calculate,
  clear,
  toggleSign,
  percentage,
} from './calculatorLogic'

describe('calculatorLogic', () => {
  describe('formatResult', () => {
    it('should return Error for NaN', () => {
      expect(formatResult(NaN)).toBe('Error')
    })

    it('should return ∞ for positive Infinity', () => {
      expect(formatResult(Infinity)).toBe('∞')
    })

    it('should return -∞ for negative Infinity', () => {
      expect(formatResult(-Infinity)).toBe('-∞')
    })

    it('should format large numbers with exponential', () => {
      expect(formatResult(1e20)).toBe('1.00000000e+20')
    })

    it('should format very small numbers with exponential', () => {
      expect(formatResult(1e-15)).toBe('1.00000000e-15')
    })

    it('should handle integers correctly', () => {
      expect(formatResult(123456789)).toBe('123456789')
    })

    it('should handle decimals with reasonable precision', () => {
      expect(formatResult(0.123456789)).toBe('0.123456789')
    })

    it('should handle zero', () => {
      expect(formatResult(0)).toBe('0')
    })
  })

  describe('factorial', () => {
    it('should return 1 for 0', () => {
      expect(factorial(0)).toBe(1)
    })

    it('should return 1 for 1', () => {
      expect(factorial(1)).toBe(1)
    })

    it('should calculate factorial correctly', () => {
      expect(factorial(5)).toBe(120)
      expect(factorial(10)).toBe(3628800)
    })

    it('should return NaN for negative numbers', () => {
      expect(factorial(-1)).toBeNaN()
    })

    it('should return Infinity for numbers > 170', () => {
      expect(factorial(171)).toBe(Infinity)
    })
  })

  describe('calculateBasic', () => {
    it('should add correctly', () => {
      expect(calculateBasic(5, 3, '+')).toBe(8)
      expect(calculateBasic(-5, 3, '+')).toBe(-2)
      expect(calculateBasic(0.1, 0.2, '+')).toBeCloseTo(0.3)
    })

    it('should subtract correctly', () => {
      expect(calculateBasic(5, 3, '-')).toBe(2)
      expect(calculateBasic(3, 5, '-')).toBe(-2)
    })

    it('should multiply correctly', () => {
      expect(calculateBasic(5, 3, '*')).toBe(15)
      expect(calculateBasic(-5, 3, '*')).toBe(-15)
      expect(calculateBasic(0.1, 0.2, '*')).toBeCloseTo(0.02)
    })

    it('should divide correctly', () => {
      expect(calculateBasic(6, 3, '/')).toBe(2)
      expect(calculateBasic(5, 2, '/')).toBe(2.5)
    })

    it('should return NaN for division by zero', () => {
      expect(calculateBasic(5, 0, '/')).toBeNaN()
    })

    it('should return current value for unknown operator', () => {
      expect(calculateBasic(5, 3, 'unknown')).toBe(3)
    })
  })

  describe('scientificFunc', () => {
    it('should calculate sin', () => {
      expect(scientificFunc(0, 'sin')).toBe(0)
      expect(scientificFunc(Math.PI / 2, 'sin')).toBeCloseTo(1)
    })

    it('should calculate cos', () => {
      expect(scientificFunc(0, 'cos')).toBe(1)
      expect(scientificFunc(Math.PI, 'cos')).toBeCloseTo(-1)
    })

    it('should calculate tan', () => {
      expect(scientificFunc(0, 'tan')).toBe(0)
      expect(scientificFunc(Math.PI / 4, 'tan')).toBeCloseTo(1)
    })

    it('should calculate sqrt', () => {
      expect(scientificFunc(9, 'sqrt')).toBe(3)
      expect(scientificFunc(2, 'sqrt')).toBeCloseTo(Math.sqrt(2))
    })

    it('should calculate square', () => {
      expect(scientificFunc(5, 'square')).toBe(25)
      expect(scientificFunc(-3, 'square')).toBe(9)
    })

    it('should calculate cube', () => {
      expect(scientificFunc(3, 'cube')).toBe(27)
    })

    it('should calculate log (base 10)', () => {
      expect(scientificFunc(100, 'log')).toBe(2)
      expect(scientificFunc(10, 'log')).toBe(1)
    })

    it('should calculate ln (natural log)', () => {
      expect(scientificFunc(Math.E, 'ln')).toBeCloseTo(1)
    })

    it('should calculate abs', () => {
      expect(scientificFunc(-5, 'abs')).toBe(5)
      expect(scientificFunc(5, 'abs')).toBe(5)
    })

    it('should calculate inv (1/x)', () => {
      expect(scientificFunc(4, 'inv')).toBe(0.25)
      expect(scientificFunc(0, 'inv')).toBeNaN()
    })

    it('should calculate fact', () => {
      expect(scientificFunc(5, 'fact')).toBe(120)
      expect(scientificFunc(0, 'fact')).toBe(1)
    })

    it('should calculate exp (e^x)', () => {
      expect(scientificFunc(0, 'exp')).toBe(1)
      expect(scientificFunc(1, 'exp')).toBeCloseTo(Math.E)
    })
  })

  describe('getConstantValue', () => {
    it('should return Math.PI for pi', () => {
      expect(getConstantValue('pi')).toBe(Math.PI)
    })

    it('should return Math.E for e', () => {
      expect(getConstantValue('e')).toBe(Math.E)
    })

    it('should return NaN for unknown constant', () => {
      expect(getConstantValue('unknown')).toBeNaN()
    })
  })

  describe('state operations', () => {
    it('should create initial state', () => {
      const state = createInitialState()
      expect(state.display).toBe('0')
      expect(state.expression).toBe('')
      expect(state.currentNumber).toBe('')
      expect(state.currentOperator).toBe('')
      expect(state.previousNumber).toBe('')
      expect(state.waitingForOperand).toBe(false)
    })

    it('should input number correctly', () => {
      const state = createInitialState()
      const newState = inputNumber(state, '5')
      expect(newState.display).toBe('5')
      expect(newState.currentNumber).toBe('5')
    })

    it('should append numbers correctly', () => {
      const state = createInitialState()
      const state1 = inputNumber(state, '5')
      const state2 = inputNumber(state1, '3')
      expect(state2.display).toBe('53')
      expect(state2.currentNumber).toBe('53')
    })

    it('should handle waitingForOperand state', () => {
      const state = { ...createInitialState(), waitingForOperand: true }
      const newState = inputNumber(state, '5')
      expect(newState.display).toBe('5')
      expect(newState.currentNumber).toBe('5')
      expect(newState.waitingForOperand).toBe(false)
    })

    it('should input decimal correctly', () => {
      const state = inputNumber(createInitialState(), '5')
      const newState = inputDecimal(state)
      expect(newState.display).toBe('5.')
      expect(newState.currentNumber).toBe('5.')
    })

    it('should not add duplicate decimal', () => {
      const state = inputDecimal(inputNumber(createInitialState(), '5'))
      const newState = inputDecimal(state)
      expect(newState.display).toBe('5.')
    })

    it('should input operator correctly', () => {
      const state = inputNumber(createInitialState(), '5')
      const newState = inputOperator(state, '+')
      expect(newState.currentOperator).toBe('+')
      expect(newState.previousNumber).toBe('5')
      expect(newState.waitingForOperand).toBe(true)
    })

    it('should perform intermediate calculation', () => {
      const state1 = inputNumber(createInitialState(), '5')
      const state2 = inputOperator(state1, '+')
      const state3 = inputNumber(state2, '3')
      const state4 = inputOperator(state3, '*')
      expect(parseFloat(state4.display)).toBeCloseTo(8)
      expect(state4.currentOperator).toBe('*')
    })

    it('should calculate final result', () => {
      const state1 = inputNumber(createInitialState(), '5')
      const state2 = inputOperator(state1, '+')
      const state3 = inputNumber(state2, '3')
      const result = calculate(state3)
      expect(result.resultValue).toBe('8')
      expect(result.expression).toBe('5 + 3')
    })

    it('should clear state', () => {
      const state1 = inputNumber(createInitialState(), '5')
      const cleared = clear()
      expect(cleared.display).toBe('0')
      expect(cleared.currentNumber).toBe('')
    })

    it('should toggle sign', () => {
      const state = inputNumber(createInitialState(), '5')
      const newState = toggleSign(state)
      expect(newState.display).toBe('-5')
      const toggledAgain = toggleSign(newState)
      expect(toggledAgain.display).toBe('5')
    })

    it('should calculate percentage', () => {
      const state = inputNumber(createInitialState(), '50')
      const newState = percentage(state)
      expect(newState.display).toBe('0.5')
    })
  })

  describe('full calculation flow', () => {
    it('should handle addition: 5 + 3 = 8', () => {
      let state = createInitialState()
      state = inputNumber(state, '5')
      state = inputOperator(state, '+')
      state = inputNumber(state, '3')
      const result = calculate(state)
      expect(result.resultValue).toBe('8')
    })

    it('should handle subtraction: 10 - 4 = 6', () => {
      let state = createInitialState()
      state = inputNumber(state, '1')
      state = inputNumber(state, '0')
      state = inputOperator(state, '-')
      state = inputNumber(state, '4')
      const result = calculate(state)
      expect(result.resultValue).toBe('6')
    })

    it('should handle multiplication: 6 * 7 = 42', () => {
      let state = createInitialState()
      state = inputNumber(state, '6')
      state = inputOperator(state, '*')
      state = inputNumber(state, '7')
      const result = calculate(state)
      expect(result.resultValue).toBe('42')
    })

    it('should handle division: 20 / 4 = 5', () => {
      let state = createInitialState()
      state = inputNumber(state, '2')
      state = inputNumber(state, '0')
      state = inputOperator(state, '/')
      state = inputNumber(state, '4')
      const result = calculate(state)
      expect(result.resultValue).toBe('5')
    })

    it('should handle chained operations: 5 + 3 + 2 = 10', () => {
      let state = createInitialState()
      state = inputNumber(state, '5')
      state = inputOperator(state, '+')
      state = inputNumber(state, '3')
      state = inputOperator(state, '+')  // intermediate: 5+3=8
      state = inputNumber(state, '2')
      const result = calculate(state)
      expect(result.resultValue).toBe('10')
    })

    it('should handle decimals: 0.1 + 0.2 = 0.3', () => {
      let state = createInitialState()
      state = inputNumber(state, '0')
      state = inputDecimal(state)
      state = inputNumber(state, '1')
      state = inputOperator(state, '+')
      state = inputNumber(state, '0')
      state = inputDecimal(state)
      state = inputNumber(state, '2')
      const result = calculate(state)
      expect(parseFloat(result.resultValue)).toBeCloseTo(0.3)
    })

    it('should handle percentage: 50 % = 0.5', () => {
      let state = createInitialState()
      state = inputNumber(state, '5')
      state = inputNumber(state, '0')
      state = percentage(state)
      expect(state.display).toBe('0.5')
    })

    it('should handle sign toggle: 5 ± = -5', () => {
      let state = createInitialState()
      state = inputNumber(state, '5')
      state = toggleSign(state)
      expect(state.display).toBe('-5')
    })
  })
})
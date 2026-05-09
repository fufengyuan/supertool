<template>
  <div class="max-w-[700px]">
    <h3 class="text-lg font-bold text-base-content mb-5"><SvgIcon name="settings" size="14" class="inline-block align-text-bottom" /> ARM / HEX 互转</h3>

    <!-- Hex → ARM-like dump -->
    <div class="mb-5">
      <h4 class="text-sm font-semibold text-base-content mb-2.5 flex items-center gap-1.5">Hex → ARM 指令 (解码)</h4>
      <label class="text-xs font-medium text-base-content/60 mb-1 block">Hex 输入 (空格分隔的 32-bit 指令)</label>
      <textarea
        v-model="hexInput"
        class="textarea textarea-bordered w-full min-h-[120px] font-mono text-xs"
        placeholder="输入 ARM 机器码 Hex，如: E3A00000 E28F0000 ..."
        rows="3"
      ></textarea>

      <div class="flex gap-2.5 mb-3 flex-wrap items-center mt-3">
        <button class="btn btn-primary btn-sm" @click="hexToArm">解码 →</button>
        <button class="btn btn-ghost btn-sm" @click="copyArmResult"><SvgIcon name="file" size="14" class="align-text-bottom" /> 复制</button>
        <button class="btn btn-ghost btn-sm" @click="clearHex">清空</button>
      </div>

      <div v-if="armOutput" class="mt-2.5 p-2.5 bg-base-200 border border-base-content/10 rounded-lg font-mono text-xs whitespace-pre-wrap break-all max-h-[300px] overflow-y-auto">{{ armOutput }}</div>
    </div>

    <hr class="border-t border-base-content/10 my-5" />

    <!-- ARM Assembly → Hex -->
    <div class="mb-5">
      <h4 class="text-sm font-semibold text-base-content mb-2.5 flex items-center gap-1.5">ARM 汇编 → Hex (编码)</h4>
      <label class="text-xs font-medium text-base-content/60 mb-1 block">ARM 汇编输入 (简易模式)</label>
      <textarea
        v-model="armInput"
        class="textarea textarea-bordered w-full min-h-[120px] font-mono text-xs"
        placeholder="输入 ARM 汇编指令，如: MOV R0, #0 ..."
        rows="3"
      ></textarea>

      <div class="flex gap-2.5 mb-3 flex-wrap items-center mt-3">
        <button class="btn btn-primary btn-sm" @click="armToHex">编码 →</button>
        <button class="btn btn-ghost btn-sm" @click="copyHexResult"><SvgIcon name="file" size="14" class="align-text-bottom" /> 复制</button>
        <button class="btn btn-ghost btn-sm" @click="clearArm">清空</button>
      </div>

      <div v-if="hexOutput" class="mt-2.5 p-2.5 bg-base-200 border border-base-content/10 rounded-lg font-mono text-xs whitespace-pre-wrap break-all max-h-[300px] overflow-y-auto">{{ hexOutput }}</div>
      <div v-if="armInfo" class="info-box">{{ armInfo }}</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import SvgIcon from '@/components/ui/SvgIcon.vue'
import { ref } from 'vue'
import { copyText } from '../toolUtils'
import { useToast } from '@/composables/useToast'

const toast = useToast()
const hexInput = ref('')
const armOutput = ref('')
const armInput = ref('')
const hexOutput = ref('')
const armInfo = ref('')

// ARM instruction set basic decoder (ARM32)
interface ARMInstruction {
  condition: string
  opcode: string
  mnemonic: string
  details: string
}

function decodeCondition(cond: number): string {
  const conditions = ['EQ', 'NE', 'CS', 'CC', 'MI', 'PL', 'VS', 'VC', 'HI', 'LS', 'GE', 'LT', 'GT', 'LE', '', 'NV']
  return conditions[cond] || ''
}

function getRegisterName(reg: number): string {
  if (reg === 15) return 'PC'
  if (reg === 14) return 'LR'
  if (reg === 13) return 'SP'
  if (reg === 12) return 'IP'
  if (reg === 11) return 'FP'
  return `R${reg}`
}

function decodeARMInstruction(hex: string): ARMInstruction {
  const num = parseInt(hex, 16)
  const cond = (num >>> 28) & 0xF
  const bits = num & 0x0FFFFFFF

  const instruction: ARMInstruction = {
    condition: decodeCondition(cond),
    opcode: '',
    mnemonic: '',
    details: '',
  }

  // Decode common ARM32 instruction patterns
  const op = (bits >>> 24) & 0xF
  const op2 = (bits >>> 20) & 0xF

  // Data processing (bits 27-26 = 00, bit 25 = 0)
  if ((bits & 0x0C000000) === 0 && (bits & 0x02000000) === 0) {
    const dpOp = (bits >>> 21) & 0xF
    const mnemonics = ['AND', 'EOR', 'SUB', 'RSB', 'ADD', 'ADC', 'SBC', 'RSC', 'TST', 'TEQ', 'CMP', 'CMN', 'ORR', 'MOV', 'BIC', 'MVN']
    instruction.mnemonic = mnemonics[dpOp] || 'UNKNOWN'
    instruction.opcode = 'DP'
    const rn = getRegisterName((bits >>> 16) & 0xF)
    const rd = getRegisterName((bits >>> 12) & 0xF)

    if (bits & 0x02000000) {
      // Immediate
      const imm = (bits & 0xFF)
      const rot = ((bits >>> 8) & 0xF) * 2
      const rotated = (imm >>> rot) | (imm << (32 - rot))
      instruction.details = `${rd}, ${rn}, #${rotated >>> 0}`
    } else {
      // Register
      const rm = getRegisterName(bits & 0xF)
      const shiftType = (bits >>> 5) & 0x3
      const shiftVal = (bits >>> 7) & 0x1F
      const shiftTypes = ['LSL', 'LSR', 'ASR', 'ROR']
      instruction.details = `${rd}, ${rn}, ${rm}, ${shiftTypes[shiftType]} #${shiftVal}`
    }
  }
  // Load/Store (bits 27-26 = 01)
  else if ((bits & 0x0C000000) === 0x04000000) {
    const isLoad = bits & 0x00100000
    const isWord = bits & 0x00400000
    instruction.mnemonic = isLoad ? (isWord ? 'LDR' : 'LDRB') : (isWord ? 'STR' : 'STRB')
    instruction.opcode = 'LS'
    const rn = getRegisterName((bits >>> 16) & 0xF)
    const rd = getRegisterName((bits >>> 12) & 0xF)
    const offset = bits & 0xFFF
    instruction.details = `${rd}, [${rn}, #${offset}]`
  }
  // Branch (bits 27-25 = 101)
  else if ((bits & 0x0E000000) === 0x0A000000) {
    const isLink = bits & 0x01000000
    instruction.mnemonic = isLink ? 'BL' : 'B'
    instruction.opcode = 'BR'
    const offset = (bits & 0x00FFFFFF)
    const signedOffset = offset & 0x00800000 ? -(~(offset & 0x00FFFFFF) + 1) : offset
    instruction.details = `#0x${(signedOffset * 4).toString(16).toUpperCase()}`
  }
  // Multiply (bits 27-23 = 00000)
  else if ((bits & 0x0FC00000) === 0) {
    instruction.mnemonic = 'MUL'
    instruction.opcode = 'ML'
    instruction.details = 'multiply operation'
  }
  // SWI/SVC
  else if ((bits & 0x0F000000) === 0x0F000000) {
    instruction.mnemonic = 'SVC'
    instruction.opcode = 'SWI'
    instruction.details = `#${bits & 0x00FFFFFF}`
  }
  else {
    instruction.mnemonic = '???'
    instruction.opcode = 'UNK'
    instruction.details = `0x${hex}`
  }

  return instruction
}

function hexToArm() {
  if (!hexInput.value.trim()) {
    toast.warning('请输入 Hex 数据')
    return
  }

  const hexes = hexInput.value.trim().split(/[\s,;]+/).filter(Boolean)
  let output = ''
  let address = 0x00008000

  for (const hex of hexes) {
    const clean = hex.replace(/^0x/i, '')
    if (!/^[0-9a-fA-F]+$/.test(clean)) {
      output += `  ${address.toString(16).toUpperCase().padStart(8, '0')}  ${clean.padStart(8, '0').toUpperCase()}  [无效的 Hex]\n`
      address += 4
      continue
    }

    const padded = clean.padStart(8, '0')
    const instr = decodeARMInstruction(padded)
    const condStr = instr.condition ? ` ${instr.condition}` : ''

    output += `  ${address.toString(16).toUpperCase().padStart(8, '0')}  ${padded.toUpperCase()}  ${instr.mnemonic}${condStr}  ${instr.details}\n`
    address += 4
  }

  armOutput.value = `Address      Hex         Mnemonic        Operands\n${'─'.repeat(65)}\n${output}`
}

// Simple ARM assembly to hex encoder (limited set)
function armToHex() {
  if (!armInput.value.trim()) {
    toast.warning('请输入 ARM 汇编指令')
    return
  }

  const lines = armInput.value.trim().split('\n').filter(Boolean)
  const results: string[] = []

  for (const line of lines) {
    const trimmed = line.trim().toUpperCase()
    const encoded = encodeARMInstruction(trimmed)
    if (encoded) {
      results.push(encoded)
    }
  }

  if (results.length > 0) {
    hexOutput.value = results.join(' ')
    armInfo.value = `已编码 ${results.length} 条指令`
  } else {
    hexOutput.value = ''
    armInfo.value = '无法识别的指令'
  }
}

function encodeARMInstruction(instr: string): string {
  // Remove condition code suffix for matching
  const match = instr.match(/^([A-Z]+)(EQ|NE|CS|CC|MI|PL|VS|VC|HI|LS|GE|LT|GT|LE)?\s*(.*)/)
  if (!match) return ''

  const mnemonic = match[1]
  const cond = match[2] || ''
  const operands = match[3].trim()

  const condCode = cond ? Object.values(['EQ', 'NE', 'CS', 'CC', 'MI', 'PL', 'VS', 'VC', 'HI', 'LS', 'GE', 'LT', 'GT', 'LE', '', 'NV']).indexOf(cond) : 14

  // MOV Rd, #imm
  if (mnemonic === 'MOV') {
    const movMatch = operands.match(/(R\d+|PC|LR|SP|IP|FP)\s*,\s*#?(\d+)/)
    if (movMatch) {
      const rd = parseRegister(movMatch[1])
      const imm = parseInt(movMatch[2])
      return ((condCode << 28) | (0xE << 21) | (1 << 25) | (rd << 12) | (imm & 0xFF)).toString(16).toUpperCase().padStart(8, '0')
    }
  }

  // ADD Rd, Rn, #imm
  if (mnemonic === 'ADD') {
    const addMatch = operands.match(/(R\d+|PC|LR|SP|IP|FP)\s*,\s*(R\d+|PC|LR|SP|IP|FP)\s*,\s*#?(\d+)/)
    if (addMatch) {
      const rd = parseRegister(addMatch[1])
      const rn = parseRegister(addMatch[2])
      const imm = parseInt(addMatch[3])
      return ((condCode << 28) | (0x4 << 21) | (1 << 25) | (rn << 16) | (rd << 12) | (imm & 0xFF)).toString(16).toUpperCase().padStart(8, '0')
    }
  }

  // SUB Rd, Rn, #imm
  if (mnemonic === 'SUB') {
    const subMatch = operands.match(/(R\d+|PC|LR|SP|IP|FP)\s*,\s*(R\d+|PC|LR|SP|IP|FP)\s*,\s*#?(\d+)/)
    if (subMatch) {
      const rd = parseRegister(subMatch[1])
      const rn = parseRegister(subMatch[2])
      const imm = parseInt(subMatch[3])
      return ((condCode << 28) | (0x2 << 21) | (1 << 25) | (rn << 16) | (rd << 12) | (imm & 0xFF)).toString(16).toUpperCase().padStart(8, '0')
    }
  }

  // B #offset
  if (mnemonic === 'B') {
    const bMatch = operands.match(/#?0x([0-9a-fA-F]+)/)
    if (bMatch) {
      const offset = parseInt(bMatch[1], 16) / 4
      return ((condCode << 28) | (0xA << 24) | (offset & 0xFFFFFF)).toString(16).toUpperCase().padStart(8, '0')
    }
  }

  // BL #offset
  if (mnemonic === 'BL') {
    const blMatch = operands.match(/#?0x([0-9a-fA-F]+)/)
    if (blMatch) {
      const offset = parseInt(blMatch[1], 16) / 4
      return ((condCode << 28) | (0xB << 24) | (offset & 0xFFFFFF)).toString(16).toUpperCase().padStart(8, '0')
    }
  }

  // NOP = MOV R0, R0
  if (mnemonic === 'NOP') {
    return ((condCode << 28) | (0xE << 21) | (0 << 16) | (0 << 12)).toString(16).toUpperCase().padStart(8, '0')
  }

  return ''
}

function parseRegister(name: string): number {
  if (name === 'PC') return 15
  if (name === 'LR') return 14
  if (name === 'SP') return 13
  if (name === 'IP') return 12
  if (name === 'FP') return 11
  const match = name.match(/R(\d+)/)
  return match ? parseInt(match[1]) : 0
}

function copyArmResult() {
  if (!armOutput.value) {
    toast.warning('没有可复制的结果')
    return
  }
  copyText(armOutput.value, toast)
}

function copyHexResult() {
  if (!hexOutput.value) {
    toast.warning('没有可复制的结果')
    return
  }
  copyText(hexOutput.value, toast)
}

function clearHex() {
  hexInput.value = ''
  armOutput.value = ''
}

function clearArm() {
  armInput.value = ''
  hexOutput.value = ''
  armInfo.value = ''
}
</script>
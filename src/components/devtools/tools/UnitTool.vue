<template>
  <div class="tool-panel">
    <h3>📐 单位换算</h3>

    <div class="tool-section">
      <label class="tool-label">类别</label>
      <select v-model="category" class="tool-select" @change="onCategoryChange">
        <option v-for="cat in categories" :key="cat.key" :value="cat.key">{{ cat.label }}</option>
      </select>

      <div class="converter-row">
        <div class="converter-side">
          <label class="tool-label">从</label>
          <select v-model="fromUnit" class="tool-select" @change="convert">
            <option v-for="u in currentUnits" :key="u.key" :value="u.key">{{ u.label }}</option>
          </select>
          <input
            v-model.number="inputValue"
            class="tool-input"
            type="number"
            placeholder="输入值..."
            @input="convert"
          />
        </div>

        <div class="converter-arrow">→</div>

        <div class="converter-side">
          <label class="tool-label">到</label>
          <select v-model="toUnit" class="tool-select" @change="convert">
            <option v-for="u in currentUnits" :key="u.key" :value="u.key">{{ u.label }}</option>
          </select>
          <div class="tool-input result-value">{{ outputValue }}</div>
        </div>
      </div>

      <div class="tool-row" style="margin-top: 12px">
        <button class="tool-btn" @click="swapUnits">🔄 交换</button>
        <button class="tool-btn" @click="copyResult">📋 复制结果</button>
      </div>

      <!-- All conversions table -->
      <div v-if="allResults.length > 0" class="all-results">
        <h4>全部换算结果</h4>
        <div class="all-results-grid">
          <div
            v-for="r in allResults"
            :key="r.key"
            class="all-result-item"
            @click="copyValue(r.value)"
          >
            <span class="all-result-label">{{ r.label }}</span>
            <span class="all-result-value">{{ r.value }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { copyText } from '../toolUtils'
import { useToast } from '@/composables/useToast'

const toast = useToast()
const category = ref('length')
const fromUnit = ref('m')
const toUnit = ref('km')
const inputValue = ref<number | null>(null)
const outputValue = ref('')

interface Unit {
  key: string
  label: string
  factor?: number  // relative to base unit
}

interface CategoryDef {
  key: string
  label: string
  baseUnit: string
  units: Unit[]
  isTemperature?: boolean
}

const categories: CategoryDef[] = [
  {
    key: 'length', label: '长度', baseUnit: 'm',
    units: [
      { key: 'nm', label: '纳米 (nm)', factor: 1e-9 },
      { key: 'um', label: '微米 (μm)', factor: 1e-6 },
      { key: 'mm', label: '毫米 (mm)', factor: 0.001 },
      { key: 'cm', label: '厘米 (cm)', factor: 0.01 },
      { key: 'dm', label: '分米 (dm)', factor: 0.1 },
      { key: 'm', label: '米 (m)', factor: 1 },
      { key: 'km', label: '千米 (km)', factor: 1000 },
      { key: 'in', label: '英寸 (in)', factor: 0.0254 },
      { key: 'ft', label: '英尺 (ft)', factor: 0.3048 },
      { key: 'yd', label: '码 (yd)', factor: 0.9144 },
      { key: 'mi', label: '英里 (mi)', factor: 1609.344 },
      { key: 'nmi', label: '海里 (nmi)', factor: 1852 },
      { key: 'ly', label: '光年 (ly)', factor: 9.461e15 },
      { key: 'au', label: '天文单位 (AU)', factor: 1.496e11 },
    ],
  },
  {
    key: 'area', label: '面积', baseUnit: 'm2',
    units: [
      { key: 'mm2', label: '平方毫米 (mm²)', factor: 1e-6 },
      { key: 'cm2', label: '平方厘米 (cm²)', factor: 1e-4 },
      { key: 'm2', label: '平方米 (m²)', factor: 1 },
      { key: 'km2', label: '平方千米 (km²)', factor: 1e6 },
      { key: 'ha', label: '公顷 (ha)', factor: 1e4 },
      { key: 'acre', label: '英亩 (acre)', factor: 4046.86 },
      { key: 'ft2', label: '平方英尺 (ft²)', factor: 0.0929 },
      { key: 'in2', label: '平方英寸 (in²)', factor: 6.452e-4 },
    ],
  },
  {
    key: 'volume', label: '体积', baseUnit: 'L',
    units: [
      { key: 'ml', label: '毫升 (mL)', factor: 0.001 },
      { key: 'cl', label: '厘升 (cL)', factor: 0.01 },
      { key: 'dl', label: '分升 (dL)', factor: 0.1 },
      { key: 'L', label: '升 (L)', factor: 1 },
      { key: 'm3', label: '立方米 (m³)', factor: 1000 },
      { key: 'gal', label: '加仑 (gal)', factor: 3.785 },
      { key: 'qt', label: '夸脱 (qt)', factor: 0.9464 },
      { key: 'pt', label: '品脱 (pt)', factor: 0.4732 },
      { key: 'cup', label: '杯 (cup)', factor: 0.2366 },
      { key: 'floz', label: '液盎司 (fl oz)', factor: 0.02957 },
      { key: 'tbsp', label: '汤匙 (tbsp)', factor: 0.01479 },
      { key: 'tsp', label: '茶匙 (tsp)', factor: 0.004929 },
    ],
  },
  {
    key: 'mass', label: '质量', baseUnit: 'kg',
    units: [
      { key: 'mg', label: '毫克 (mg)', factor: 1e-6 },
      { key: 'g', label: '克 (g)', factor: 0.001 },
      { key: 'kg', label: '千克 (kg)', factor: 1 },
      { key: 't', label: '吨 (t)', factor: 1000 },
      { key: 'oz', label: '盎司 (oz)', factor: 0.02835 },
      { key: 'lb', label: '磅 (lb)', factor: 0.4536 },
      { key: 'st', label: '英石 (st)', factor: 6.35 },
      { key: 'ct', label: '克拉 (ct)', factor: 0.0002 },
    ],
  },
  {
    key: 'temperature', label: '温度', baseUnit: 'C',
    isTemperature: true,
    units: [
      { key: 'C', label: '摄氏度 (°C)' },
      { key: 'F', label: '华氏度 (°F)' },
      { key: 'K', label: '开尔文 (K)' },
    ],
  },
  {
    key: 'pressure', label: '压强', baseUnit: 'Pa',
    units: [
      { key: 'Pa', label: '帕斯卡 (Pa)', factor: 1 },
      { key: 'kPa', label: '千帕 (kPa)', factor: 1000 },
      { key: 'MPa', label: '兆帕 (MPa)', factor: 1e6 },
      { key: 'bar', label: '巴 (bar)', factor: 1e5 },
      { key: 'mbar', label: '毫巴 (mbar)', factor: 100 },
      { key: 'atm', label: '标准大气压 (atm)', factor: 101325 },
      { key: 'psi', label: '磅/平方英寸 (psi)', factor: 6894.76 },
      { key: 'mmHg', label: '毫米汞柱 (mmHg)', factor: 133.322 },
      { key: 'Torr', label: '托 (Torr)', factor: 133.322 },
    ],
  },
  {
    key: 'power', label: '功率', baseUnit: 'W',
    units: [
      { key: 'mW', label: '毫瓦 (mW)', factor: 0.001 },
      { key: 'W', label: '瓦特 (W)', factor: 1 },
      { key: 'kW', label: '千瓦 (kW)', factor: 1000 },
      { key: 'MW', label: '兆瓦 (MW)', factor: 1e6 },
      { key: 'hp', label: '马力 (hp)', factor: 745.7 },
      { key: 'BTUh', label: 'BTU/h', factor: 0.2931 },
    ],
  },
  {
    key: 'energy', label: '能量', baseUnit: 'J',
    units: [
      { key: 'J', label: '焦耳 (J)', factor: 1 },
      { key: 'kJ', label: '千焦 (kJ)', factor: 1000 },
      { key: 'cal', label: '卡路里 (cal)', factor: 4.184 },
      { key: 'kcal', label: '千卡 (kcal)', factor: 4184 },
      { key: 'Wh', label: '瓦时 (Wh)', factor: 3600 },
      { key: 'kWh', label: '千瓦时 (kWh)', factor: 3.6e6 },
      { key: 'BTU', label: 'BTU', factor: 1055.06 },
      { key: 'eV', label: '电子伏特 (eV)', factor: 1.602e-19 },
    ],
  },
  {
    key: 'density', label: '密度', baseUnit: 'kgm3',
    units: [
      { key: 'kgm3', label: 'kg/m³', factor: 1 },
      { key: 'gcm3', label: 'g/cm³', factor: 1000 },
      { key: 'kgL', label: 'kg/L', factor: 1000 },
      { key: 'lbft3', label: 'lb/ft³', factor: 16.0185 },
      { key: 'lbin3', label: 'lb/in³', factor: 27679.9 },
    ],
  },
  {
    key: 'force', label: '力', baseUnit: 'N',
    units: [
      { key: 'N', label: '牛顿 (N)', factor: 1 },
      { key: 'kN', label: '千牛 (kN)', factor: 1000 },
      { key: 'dyn', label: '达因 (dyn)', factor: 1e-5 },
      { key: 'kgf', label: '千克力 (kgf)', factor: 9.80665 },
      { key: 'lbf', label: '磅力 (lbf)', factor: 4.44822 },
      { key: 'pdl', label: '磅达 (pdl)', factor: 0.138255 },
    ],
  },
  {
    key: 'time', label: '时间', baseUnit: 's',
    units: [
      { key: 'ns', label: '纳秒 (ns)', factor: 1e-9 },
      { key: 'us', label: '微秒 (μs)', factor: 1e-6 },
      { key: 'ms', label: '毫秒 (ms)', factor: 0.001 },
      { key: 's', label: '秒 (s)', factor: 1 },
      { key: 'min', label: '分钟 (min)', factor: 60 },
      { key: 'h', label: '小时 (h)', factor: 3600 },
      { key: 'd', label: '天 (d)', factor: 86400 },
      { key: 'wk', label: '周 (wk)', factor: 604800 },
      { key: 'mo', label: '月 (mo)', factor: 2592000 },
      { key: 'yr', label: '年 (yr)', factor: 31536000 },
    ],
  },
  {
    key: 'speed', label: '速度', baseUnit: 'ms',
    units: [
      { key: 'ms', label: '米/秒 (m/s)', factor: 1 },
      { key: 'kmh', label: '千米/时 (km/h)', factor: 0.2778 },
      { key: 'mph', label: '英里/时 (mph)', factor: 0.447 },
      { key: 'kn', label: '节 (kn)', factor: 0.5144 },
      { key: 'fts', label: '英尺/秒 (ft/s)', factor: 0.3048 },
      { key: 'mach', label: '马赫 (Ma)', factor: 340.3 },
    ],
  },
  {
    key: 'data', label: '数据存储', baseUnit: 'B',
    units: [
      { key: 'bit', label: '比特 (bit)', factor: 0.125 },
      { key: 'B', label: '字节 (B)', factor: 1 },
      { key: 'KB', label: '千字节 (KB)', factor: 1024 },
      { key: 'MB', label: '兆字节 (MB)', factor: 1048576 },
      { key: 'GB', label: '吉字节 (GB)', factor: 1073741824 },
      { key: 'TB', label: '太字节 (TB)', factor: 1099511627776 },
      { key: 'PB', label: '拍字节 (PB)', factor: 1125899906842624 },
    ],
  },
  {
    key: 'angle', label: '角度', baseUnit: 'deg',
    units: [
      { key: 'deg', label: '度 (°)', factor: 1 },
      { key: 'rad', label: '弧度 (rad)', factor: 180 / Math.PI },
      { key: 'grad', label: '梯度 (grad)', factor: 0.9 },
      { key: 'arcmin', label: '角分 (′)', factor: 1 / 60 },
      { key: 'arcsec', label: '角秒 (″)', factor: 1 / 3600 },
      { key: 'rev', label: '转 (rev)', factor: 360 },
    ],
  },
]

const currentCategory = computed(() => categories.find(c => c.key === category.value)!)
const currentUnits = computed(() => currentCategory.value.units)

const allResults = computed(() => {
  if (inputValue.value === null || inputValue.value === undefined) return []
  const base = toBase(inputValue.value, fromUnit.value)
  return currentUnits.value
    .filter(u => u.key !== fromUnit.value)
    .map(u => ({
      key: u.key,
      label: u.label,
      value: formatNumber(fromBase(base, u.key)),
    }))
})

function toBase(value: number, unitKey: string): number {
  if (currentCategory.value.isTemperature) {
    return tempToC(value, unitKey)
  }
  const unit = currentUnits.value.find(u => u.key === unitKey)
  return value * (unit?.factor ?? 1)
}

function fromBase(baseValue: number, unitKey: string): number {
  if (currentCategory.value.isTemperature) {
    return tempFromC(baseValue, unitKey)
  }
  const unit = currentUnits.value.find(u => u.key === unitKey)
  return baseValue / (unit?.factor ?? 1)
}

function tempToC(value: number, unit: string): number {
  switch (unit) {
    case 'C': return value
    case 'F': return (value - 32) * 5 / 9
    case 'K': return value - 273.15
    default: return value
  }
}

function tempFromC(celsius: number, unit: string): number {
  switch (unit) {
    case 'C': return celsius
    case 'F': return celsius * 9 / 5 + 32
    case 'K': return celsius + 273.15
    default: return celsius
  }
}

function formatNumber(n: number): string {
  if (n === 0) return '0'
  const abs = Math.abs(n)
  if (abs >= 1e15 || (abs < 1e-10 && abs !== 0)) {
    return n.toExponential(6)
  }
  if (Number.isInteger(n) && abs < 1e15) {
    return n.toString()
  }
  // Use enough precision
  return parseFloat(n.toPrecision(10)).toString()
}

function convert() {
  if (inputValue.value === null || inputValue.value === undefined) {
    outputValue.value = ''
    return
  }
  const base = toBase(inputValue.value, fromUnit.value)
  const result = fromBase(base, toUnit.value)
  outputValue.value = formatNumber(result)
}

function onCategoryChange() {
  const units = currentUnits.value
  fromUnit.value = units[0]?.key || ''
  toUnit.value = units[1]?.key || ''
  inputValue.value = null
  outputValue.value = ''
}

function swapUnits() {
  const temp = fromUnit.value
  fromUnit.value = toUnit.value
  toUnit.value = temp
  convert()
}

function copyResult() {
  if (!outputValue.value) {
    toast.warning('没有可复制的结果')
    return
  }
  copyText(outputValue.value, toast)
}

function copyValue(value: string) {
  copyText(value, toast)
}
</script>

<style scoped>
.tool-panel h3 {
  font-size: 18px;
  font-weight: 700;
  color: var(--color-base-content);
  margin: 0 0 20px 0;
}

.tool-panel h4 {
  font-size: 14px;
  font-weight: 600;
  color: var(--color-base-content);
  margin: 0 0 10px 0;
}

.converter-row {
  display: flex;
  align-items: flex-end;
  gap: 16px;
  margin-top: 12px;
}

.converter-side {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.converter-arrow {
  font-size: 24px;
  color: var(--color-primary);
  font-weight: 700;
  padding-bottom: 8px;
}

.result-value {
  padding: 8px 12px;
  background: var(--color-base-100);
  border: 1px solid var(--color-primary);
  border-radius: 6px;
  font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 14px;
  color: var(--color-primary);
  font-weight: 600;
}

.all-results {
  margin-top: 20px;
}

.all-results-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: 8px;
}

.all-result-item {
  display: flex;
  flex-direction: column;
  padding: 8px 12px;
  background: var(--color-base-200);
  border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent);
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.15s;
}

.all-result-item:hover {
  border-color: var(--color-primary);
}

.all-result-label {
  font-size: 11px;
  font-weight: 500;
  color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  margin-bottom: 2px;
}

.all-result-value {
  font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 13px;
  color: var(--color-base-content);
  word-break: break-all;
}

.tool-section { margin-bottom: 20px; }
.tool-section h4 { font-size: 14px; font-weight: 600; color: var(--color-base-content); margin: 0 0 10px 0; display: flex; align-items: center; gap: 6px; }
.tool-textarea { width: 100%; min-height: 120px; padding: 10px 12px; border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent); border-radius: 8px; font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace; font-size: 13px; background: var(--color-base-200); color: var(--color-base-content); resize: vertical; outline: none; }
.tool-textarea:focus { border-color: var(--color-primary); }
.tool-input { width: 100%; padding: 8px 12px; border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent); border-radius: 6px; font-size: 13px; background: var(--color-base-200); color: var(--color-base-content); outline: none; }
.tool-input:focus { border-color: var(--color-primary); }
.tool-row { display: flex; gap: 10px; margin-bottom: 12px; flex-wrap: wrap; }
.tool-btn { padding: 7px 16px; border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent); border-radius: 6px; font-size: 13px; font-weight: 500; cursor: pointer; background: var(--color-base-100); color: var(--color-base-content); transition: all 0.15s; display: inline-flex; align-items: center; gap: 4px; }
.tool-btn:hover { border-color: var(--color-primary); color: var(--color-primary); }
.tool-btn.primary { background: var(--color-primary); color: white; border-color: var(--color-primary); }
.tool-btn.primary:hover { opacity: 0.9; }
.tool-btn-group { display: flex; gap: 4px; }
.tool-btn-group .tool-btn { border-radius: 0; }
.tool-btn-group .tool-btn:first-child { border-radius: 6px 0 0 6px; }
.tool-btn-group .tool-btn:last-child { border-radius: 0 6px 6px 0; }
.tool-btn-group .tool-btn.active { background: var(--color-primary); color: white; border-color: var(--color-primary); position: relative; z-index: 1; }
.tool-result { margin-top: 10px; padding: 10px 12px; background: var(--color-base-200); border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent); border-radius: 8px; font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace; font-size: 13px; color: var(--color-base-content); white-space: pre-wrap; word-break: break-all; max-height: 300px; overflow-y: auto; }
.tool-label { font-size: 12px; font-weight: 500; color: color-mix(in oklab, var(--color-base-content) 60%, transparent); margin-bottom: 4px; display: block; }
.tool-select { padding: 7px 10px; border: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent); border-radius: 6px; font-size: 13px; background: var(--color-base-200); color: var(--color-base-content); outline: none; }
.tool-select:focus { border-color: var(--color-primary); }
.tool-checkbox { display: flex; align-items: center; gap: 6px; font-size: 13px; color: var(--color-base-content); cursor: pointer; }
.tool-divider { border: none; border-top: 1px solid color-mix(in oklab, var(--color-base-content) 10%, transparent); margin: 20px 0; }
</style>

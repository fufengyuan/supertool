<template>
  <div class="pinyin-tool">
    <h3>🔤 汉字转拼音</h3>

    <div class="tool-section">
      <h4>输入汉字</h4>
      <textarea
        v-model="chineseInput"
        class="tool-textarea"
        placeholder="请输入中文文本..."
        rows="3"
        @input="convertPinyin"
      ></textarea>

      <div class="tool-row" style="margin-top: 12px">
        <div>
          <label class="tool-label">转换模式</label>
          <select v-model="pinyinMode" class="tool-select" @change="convertPinyin">
            <option value="tone">带声调 (nǐ hǎo)</option>
            <option value="notone">无声调 (ni hao)</option>
            <option value="initials">首字母 (nh)</option>
          </select>
        </div>
        <div>
          <label class="tool-label">分隔符</label>
          <select v-model="separator" class="tool-select" @change="convertPinyin">
            <option value=" ">空格</option>
            <option value=",">逗号</option>
            <option value="-">横线</option>
            <option value="">无分隔</option>
          </select>
        </div>
        <button class="tool-btn primary" @click="convertPinyin">转换</button>
        <button class="tool-btn" @click="copyText(pinyinOutput, toast)">📋 复制</button>
      </div>

      <div v-if="pinyinOutput" class="tool-result">{{ pinyinOutput }}</div>
    </div>

    <hr class="tool-divider" />

    <!-- Examples -->
    <div class="tool-section">
      <h4>示例</h4>
      <div class="examples-grid">
        <div
          v-for="ex in examples"
          :key="ex.text"
          class="example-item"
          @click="chineseInput = ex.text; convertPinyin()"
        >
          <span class="example-text">{{ ex.text }}</span>
          <span class="example-pinyin">{{ ex.pinyin }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { pinyin } from 'pinyin-pro'
import { copyText } from '../toolUtils'
import { useToast } from '@/composables/useToast'

const toast = useToast()

const chineseInput = ref('你好世界')
const pinyinMode = ref<'tone' | 'notone' | 'initials'>('tone')
const separator = ref(' ')
const pinyinOutput = ref('')

function convertPinyin() {
  if (!chineseInput.value.trim()) { pinyinOutput.value = ''; return }
  try {
    let result: string
    if (pinyinMode.value === 'initials') {
      result = pinyin(chineseInput.value, {
        pattern: 'first',
        toneType: 'none',
        type: 'array',
      }).join(separator.value)
    } else {
      result = pinyin(chineseInput.value, {
        pattern: 'pinyin',
        toneType: pinyinMode.value === 'tone' ? 'symbol' : 'none',
        type: 'array',
      }).join(separator.value)
    }
    pinyinOutput.value = result
  } catch (e: any) {
    pinyinOutput.value = `错误: ${e.message || '转换失败'}`
    toast.error('拼音转换失败')
  }
}

const examples = [
  { text: '你好世界', pinyin: 'nǐ hǎo shì jiè' },
  { text: '今天天气真好', pinyin: 'jīn tiān tiān qì zhēn hǎo' },
  { text: '中华人民共和国', pinyin: 'zhōng huá rén mín gòng hé guó' },
  { text: '北京欢迎你', pinyin: 'běi jīng huān yíng nǐ' },
]
</script>

<style scoped>

.pinyin-tool {
  max-width: 700px;
}

.pinyin-tool h3 {
  font-size: 18px;
  font-weight: 700;
  color: oklch(var(--bc));
  margin: 0 0 20px 0;
}

.tool-section h4 {
  font-size: 14px;
  font-weight: 600;
  color: oklch(var(--bc));
  margin: 0 0 10px 0;
}

.tool-row {
  display: flex;
  gap: 10px;
  margin-bottom: 12px;
  flex-wrap: wrap;
  align-items: flex-end;
}

.tool-result {
  margin-top: 10px;
  padding: 10px 12px;
  background: oklch(var(--b2));
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 8px;
  font-size: 16px;
  color: oklch(var(--bc));
  white-space: pre-wrap;
  word-break: break-all;
}

.examples-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: 8px;
}

.example-item {
  padding: 8px 12px;
  background: oklch(var(--b2));
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.15s;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.example-item:hover {
  border-color: oklch(var(--p));
  background: oklch(var(--p) / 0.1);
}

.example-text {
  font-size: 14px;
  color: oklch(var(--bc));
  font-weight: 500;
}

.example-pinyin {
  font-size: 12px;
  color: oklch(var(--bc) / 0.6);
}

.tool-btn {
  padding: 7px 16px;
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 6px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  background: oklch(var(--b1));
  color: oklch(var(--bc));
  transition: all 0.15s;
  display: inline-flex;
  align-items: center;
  gap: 4px;
}

.tool-btn:hover {
  border-color: oklch(var(--p));
  color: oklch(var(--p));
}

.tool-btn.primary {
  background: oklch(var(--p));
  color: white;
  border-color: oklch(var(--p));
}

.tool-btn.primary:hover {
  opacity: 0.9;
}

.tool-divider {
  border: none;
  border-top: 1px solid oklch(var(--bc) / 0.1);
  margin: 20px 0;
}

.tool-select {
  padding: 7px 10px;
  border: 1px solid oklch(var(--bc) / 0.1);
  border-radius: 6px;
  font-size: 13px;
  background: oklch(var(--b2));
  color: oklch(var(--bc));
  outline: none;
}

.tool-label {
  font-size: 12px;
  font-weight: 500;
  color: oklch(var(--bc) / 0.6);
  margin-bottom: 4px;
  display: block;
}

.tool-section { margin-bottom: 20px; }
.tool-section h4 { font-size: 14px; font-weight: 600; color: oklch(var(--bc)); margin: 0 0 10px 0; display: flex; align-items: center; gap: 6px; }
.tool-textarea { width: 100%; min-height: 120px; padding: 10px 12px; border: 1px solid oklch(var(--bc) / 0.1); border-radius: 8px; font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace; font-size: 13px; background: oklch(var(--b2)); color: oklch(var(--bc)); resize: vertical; outline: none; }
.tool-textarea:focus { border-color: oklch(var(--p)); }
.tool-input { width: 100%; padding: 8px 12px; border: 1px solid oklch(var(--bc) / 0.1); border-radius: 6px; font-size: 13px; background: oklch(var(--b2)); color: oklch(var(--bc)); outline: none; }
.tool-input:focus { border-color: oklch(var(--p)); }
.tool-row { display: flex; gap: 10px; margin-bottom: 12px; flex-wrap: wrap; }
.tool-btn { padding: 7px 16px; border: 1px solid oklch(var(--bc) / 0.1); border-radius: 6px; font-size: 13px; font-weight: 500; cursor: pointer; background: oklch(var(--b1)); color: oklch(var(--bc)); transition: all 0.15s; display: inline-flex; align-items: center; gap: 4px; }
.tool-btn:hover { border-color: oklch(var(--p)); color: oklch(var(--p)); }
.tool-btn.primary { background: oklch(var(--p)); color: white; border-color: oklch(var(--p)); }
.tool-btn.primary:hover { opacity: 0.9; }
.tool-btn-group { display: flex; gap: 4px; }
.tool-btn-group .tool-btn { border-radius: 0; }
.tool-btn-group .tool-btn:first-child { border-radius: 6px 0 0 6px; }
.tool-btn-group .tool-btn:last-child { border-radius: 0 6px 6px 0; }
.tool-btn-group .tool-btn.active { background: oklch(var(--p)); color: white; border-color: oklch(var(--p)); position: relative; z-index: 1; }
.tool-result { margin-top: 10px; padding: 10px 12px; background: oklch(var(--b2)); border: 1px solid oklch(var(--bc) / 0.1); border-radius: 8px; font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace; font-size: 13px; color: oklch(var(--bc)); white-space: pre-wrap; word-break: break-all; max-height: 300px; overflow-y: auto; }
.tool-label { font-size: 12px; font-weight: 500; color: oklch(var(--bc) / 0.6); margin-bottom: 4px; display: block; }
.tool-select { padding: 7px 10px; border: 1px solid oklch(var(--bc) / 0.1); border-radius: 6px; font-size: 13px; background: oklch(var(--b2)); color: oklch(var(--bc)); outline: none; }
.tool-select:focus { border-color: oklch(var(--p)); }
.tool-checkbox { display: flex; align-items: center; gap: 6px; font-size: 13px; color: oklch(var(--bc)); cursor: pointer; }
.tool-divider { border: none; border-top: 1px solid oklch(var(--bc) / 0.1); margin: 20px 0; }
</style>

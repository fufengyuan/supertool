<template>
  <div class="tool-panel">
    <h3>🌍 IP 地址查询</h3>

    <div class="tool-section">
      <label class="tool-label">IP 地址</label>
      <div class="ip-input-row">
        <input
          v-model="input"
          class="tool-input"
          placeholder="输入 IP 地址或留空查询本机 IP..."
          @keyup.enter="query"
        />
        <button class="tool-btn" @click="queryCurrentIp" style="margin-left: 8px">
          📡 查询本机 IP
        </button>
      </div>

      <div class="tool-row" style="margin-top: 12px">
        <button class="tool-btn primary" @click="query">查询</button>
        <button class="tool-btn" @click="clear">清空</button>
      </div>

      <div v-if="loading" class="loading-box">
        <div class="spinner"></div>
        <span>正在查询...</span>
      </div>

      <div v-if="error" class="error-box">{{ error }}</div>

      <div v-if="result" class="result-grid">
        <div class="result-card">
          <div class="result-label">🌐 IP 地址</div>
          <div class="result-value">{{ result.ip }}</div>
          <button class="mini-copy-btn" @click="doCopy(result.ip)">📋</button>
        </div>
        <div class="result-card">
          <div class="result-label">🏳️ 国家</div>
          <div class="result-value">{{ result.country }}</div>
        </div>
        <div class="result-card">
          <div class="result-label">🏙️ 城市</div>
          <div class="result-value">{{ result.city }}</div>
        </div>
        <div class="result-card">
          <div class="result-label">📍 省份/地区</div>
          <div class="result-value">{{ result.region }}</div>
        </div>
        <div class="result-card">
          <div class="result-label">📡 ISP / 运营商</div>
          <div class="result-value">{{ result.isp }}</div>
        </div>
        <div class="result-card">
          <div class="result-label">🏢 组织</div>
          <div class="result-value">{{ result.org }}</div>
        </div>
        <div class="result-card">
          <div class="result-label">🕐 时区</div>
          <div class="result-value">{{ result.timezone }}</div>
        </div>
        <div class="result-card">
          <div class="result-label">🌐 ASN</div>
          <div class="result-value">{{ result.as }}</div>
        </div>
        <div class="result-card">
          <div class="result-label">📐 纬度</div>
          <div class="result-value">{{ result.lat }}</div>
        </div>
        <div class="result-card">
          <div class="result-label">📐 经度</div>
          <div class="result-value">{{ result.lon }}</div>
        </div>
      </div>

      <div v-if="result" class="map-link" style="margin-top: 12px">
        <a :href="result.mapUrl" target="_blank" rel="noopener" class="map-link-btn">
          🗺️ 在地图上查看
        </a>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { copyText } from '../toolUtils'
import { useToast } from '@/composables/useToast'

const toast = useToast()
const input = ref('')
const loading = ref(false)
const error = ref('')

interface IpInfo {
  ip: string
  country: string
  city: string
  region: string
  isp: string
  org: string
  timezone: string
  as: string
  lat: string
  lon: string
  mapUrl: string
}

const result = ref<IpInfo | null>(null)

async function query() {
  loading.value = true
  error.value = ''
  result.value = null

  const ip = input.value.trim() || undefined

  try {
    // Use HTTPS-only APIs to avoid mixed-content blocking
    const apis = [
      {
        name: 'ip-api.com',
        url: ip
          ? `https://ip-api.com/json/${ip}?lang=zh-CN`
          : 'https://ip-api.com/json/?lang=zh-CN',
        parse: (data: any) => ({
          ip: data.query || '—',
          country: data.country || '—',
          city: data.city || '—',
          region: data.regionName || '—',
          isp: data.isp || '—',
          org: data.org || '—',
          timezone: data.timezone || '—',
          as: data.as || '—',
          lat: data.lat ? String(data.lat) : '—',
          lon: data.lon ? String(data.lon) : '—',
          mapUrl: data.lat && data.lon ? `https://www.google.com/maps?q=${data.lat},${data.lon}` : '',
          input: data.query,
        }),
        checkFail: (data: any) => data.status === 'fail',
        failMsg: (data: any) => data.message || '查询失败',
      },
      {
        name: 'ipinfo.io',
        url: ip ? `https://ipinfo.io/${ip}/json` : 'https://ipinfo.io/json',
        parse: (data: any) => {
          const [lat, lon] = (data.loc || ',').split(',')
          return {
            ip: data.ip || '—',
            country: data.country || '—',
            city: data.city || '—',
            region: data.region || '—',
            isp: data.org || '—',
            org: data.org || '—',
            timezone: data.timezone || '—',
            as: data.org || '—',
            lat: lat || '—',
            lon: lon || '—',
            mapUrl: lat && lon ? `https://www.google.com/maps?q=${lat},${lon}` : '',
            input: data.ip,
          }
        },
        checkFail: (data: any) => !data.ip && !data.loc,
        failMsg: () => '查询失败',
      },
      {
        name: 'ipapi.co',
        url: ip ? `https://ipapi.co/${ip}/json/` : 'https://ipapi.co/json/',
        parse: (data: any) => ({
          ip: data.ip || '—',
          country: data.country_name || data.country || '—',
          city: data.city || '—',
          region: data.region || '—',
          isp: data.org || '—',
          org: data.org || '—',
          timezone: data.timezone || '—',
          as: data.asn || '—',
          lat: data.latitude ? String(data.latitude) : '—',
          lon: data.longitude ? String(data.longitude) : '—',
          mapUrl: data.latitude && data.longitude ? `https://www.google.com/maps?q=${data.latitude},${data.longitude}` : '',
          input: data.ip,
        }),
        checkFail: (data: any) => data.error,
        failMsg: (data: any) => data.reason || '查询失败',
      },
    ]

    for (const api of apis) {
      try {
        const response = await fetch(api.url, { signal: AbortSignal.timeout(10000) })
        if (!response.ok) throw new Error(`请求失败: ${response.status}`)
        const data = await response.json()
        if (api.checkFail(data)) throw new Error(api.failMsg(data))

        const parsed = api.parse(data)
        result.value = parsed
        if (!input.value.trim() && parsed.input) {
          input.value = parsed.input
        }
        return // success
      } catch {
        // try next API
        continue
      }
    }

    throw new Error('所有查询服务均不可用')
  } catch (e: any) {
    error.value = `查询失败: ${e.message || '未知错误'}`
  } finally {
    loading.value = false
  }
}

async function queryCurrentIp() {
  input.value = ''
  await query()
}

function doCopy(text: string) {
  copyText(text, toast)
}

function clear() {
  input.value = ''
  error.value = ''
  result.value = null
}
</script>



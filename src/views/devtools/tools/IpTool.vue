<template>
  <div class="max-w-[700px]">
    <h3 class="text-lg font-bold text-base-content mb-5">🌍 IP 地址查询</h3>

    <div class="mb-5">
      <label class="text-xs font-medium text-base-content/60 mb-1 block">IP 地址</label>
      <div class="ip-input-row">
        <input
          v-model="input"
          class="input input-bordered w-full font-mono text-xs"
          placeholder="输入 IP 地址或留空查询本机 IP..."
          @keyup.enter="query"
        />
        <button class="btn btn-ghost btn-sm" @click="queryCurrentIp" style="margin-left: 8px">
          <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><circle cx="12" cy="12" r="2"/><path d="M16.24 7.76a6 6 0 0 1 0 8.49m-8.48-.01a6 6 0 0 1 0-8.49m11.31-2.82a10 10 0 0 1 0 14.14m-14.14 0a10 10 0 0 1 0-14.14"/></svg> 查询本机 IP
        </button>
      </div>

      <div class="flex gap-2.5 mb-3 flex-wrap items-center mt-3">
        <button class="btn btn-primary btn-sm" @click="query">查询</button>
        <button class="btn btn-ghost btn-sm" @click="clear">清空</button>
      </div>

      <div v-if="loading" class="loading-box">
        <div class="spinner"></div>
        <span>正在查询...</span>
      </div>

      <div v-if="error" class="error-box">{{ error }}</div>

      <div v-if="result" class="result-grid">
        <div class="result-card">
          <div class="result-label"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><circle cx="12" cy="12" r="10"/><line x1="2" y1="12" x2="22" y2="12"/><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/></svg> IP 地址</div>
          <div class="result-value">{{ result.ip }}</div>
          <button class="mini-copy-btn" @click="doCopy(result.ip)"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/></svg></button>
        </div>
        <div class="result-card">
          <div class="result-label"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><path d="M4 15s1-1 4-1 5 2 8 2 4-1 4-1V3s-1 1-4 1-5-2-8-2-4 1-4 1z"/><line x1="4" y1="22" x2="4" y2="15"/></svg> 国家</div>
          <div class="result-value">{{ result.country }}</div>
        </div>
        <div class="result-card">
          <div class="result-label"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><rect x="4" y="2" width="16" height="20" rx="2"/><line x1="9" y1="6" x2="9" y2="10"/><line x1="15" y1="6" x2="15" y2="10"/><line x1="9" y1="14" x2="9" y2="18"/><line x1="15" y1="14" x2="15" y2="18"/></svg> 城市</div>
          <div class="result-value">{{ result.city }}</div>
        </div>
        <div class="result-card">
          <div class="result-label"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><path d="M21 10c0 7-9 13-9 13s-9-6-9-13a9 9 0 0 1 18 0z"/><circle cx="12" cy="10" r="3"/></svg> 省份/地区</div>
          <div class="result-value">{{ result.region }}</div>
        </div>
        <div class="result-card">
          <div class="result-label"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><circle cx="12" cy="12" r="2"/><path d="M16.24 7.76a6 6 0 0 1 0 8.49m-8.48-.01a6 6 0 0 1 0-8.49m11.31-2.82a10 10 0 0 1 0 14.14m-14.14 0a10 10 0 0 1 0-14.14"/></svg> ISP / 运营商</div>
          <div class="result-value">{{ result.isp }}</div>
        </div>
        <div class="result-card">
          <div class="result-label">🏢 组织</div>
          <div class="result-value">{{ result.org }}</div>
        </div>
        <div class="result-card">
          <div class="result-label"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg> 时区</div>
          <div class="result-value">{{ result.timezone }}</div>
        </div>
        <div class="result-card">
          <div class="result-label"><svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><circle cx="12" cy="12" r="10"/><line x1="2" y1="12" x2="22" y2="12"/><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/></svg> ASN</div>
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
          <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><circle cx="12" cy="12" r="10"/><line x1="2" y1="12" x2="22" y2="12"/><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/></svg> 在地图上查看
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
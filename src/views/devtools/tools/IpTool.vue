<template>
  <ToolPage
    icon="mapPin"
    name="IP 地址查询"
    description="查询 IP 归属地、运营商、时区、ASN 与坐标，可查本机 IP"
    :offline="false"
    @back="$emit('back')"
  >
    <div class="bg-base-100 border border-base-content/10 rounded-xl p-4">
      <div class="flex gap-2 items-center">
        <input
          v-model="input"
          class="input input-bordered input-sm w-full font-mono bg-base-200/60"
          placeholder="输入 IP 地址或留空查询本机 IP..."
          @keyup.enter="query"
        />
        <button class="btn btn-outline btn-sm shrink-0" @click="queryCurrentIp">
          <SvgIcon name="wifi" size="13" /> 本机 IP
        </button>
        <button class="btn btn-primary btn-sm shrink-0" @click="query">查询</button>
      </div>
      <div class="flex gap-2 mt-3">
        <button class="btn btn-ghost btn-xs" @click="clear" :disabled="!input && !result">清空</button>
      </div>

      <div v-if="loading" class="mt-4 p-4 flex items-center gap-3 text-sm text-base-content/60">
        <span class="loading loading-spinner loading-sm text-primary"></span>
        <span>正在查询...</span>
      </div>

      <div v-if="error" class="mt-4 p-3 bg-error/10 border border-error/25 rounded-lg text-error text-sm">{{ error }}</div>

      <div v-if="result" class="mt-4">
        <div class="grid grid-cols-2 sm:grid-cols-3 gap-2.5">
          <div v-for="cell in resultCards" :key="cell.label" class="flex flex-col p-3 bg-base-200/60 border border-base-content/10 rounded-xl">
            <span class="text-[11px] font-medium text-base-content/50 mb-1 flex items-center gap-1"><SvgIcon :name="cell.icon" size="11" /> {{ cell.label }}</span>
            <span class="text-sm text-base-content font-mono break-all">{{ cell.value }}</span>
          </div>
        </div>
        <div v-if="result.mapUrl" class="mt-3">
          <a :href="result.mapUrl" target="_blank" rel="noopener" class="btn btn-outline btn-sm">
            <SvgIcon name="globe" size="12" /> 在地图上查看
          </a>
        </div>
      </div>
    </div>
  </ToolPage>
</template>

<script setup lang="ts">
import SvgIcon from '@/components/ui/SvgIcon.vue'
import ToolPage from '../components/ToolPage.vue'
import { ref, computed } from 'vue'
import { copyText } from '../toolUtils'
import { useToast } from '@/composables/useToast'

defineEmits<{ back: [] }>()

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

const resultCards = computed(() => {
  const r = result.value
  if (!r) { return [] }
  return [
    { label: 'IP 地址', icon: 'globe', value: r.ip },
    { label: '国家', icon: 'globe', value: r.country },
    { label: '城市', icon: 'monitor', value: r.city },
    { label: '省份/地区', icon: 'mapPin', value: r.region },
    { label: 'ISP / 运营商', icon: 'wifi', value: r.isp },
    { label: '组织', icon: 'layers', value: r.org },
    { label: '时区', icon: 'clock', value: r.timezone },
    { label: 'ASN', icon: 'globe', value: r.as },
    { label: '纬度', icon: 'mapPin', value: r.lat },
    { label: '经度', icon: 'mapPin', value: r.lon },
  ]
})

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
        if (!response.ok) {throw new Error(`请求失败: ${response.status}`)}
        const data = await response.json()
        if (api.checkFail(data)) {throw new Error(api.failMsg(data))}

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

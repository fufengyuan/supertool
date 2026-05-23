<template>
  <div class="bg-base-100 rounded-xl border border-base-content/10 p-4 lg:p-5 w-full overflow-hidden">
    <!-- 头部 -->
    <div class="flex items-center justify-between mb-4 flex-wrap gap-2">
      <div class="flex items-center gap-2">
        <SvgIcon name="calendar" size="16" class="text-primary" />
        <span class="text-sm font-medium text-base-content/70">万年历</span>
      </div>
      <div class="flex items-center gap-1">
        <button class="btn btn-ghost btn-xs btn-square" @click="prevYear" title="上一年">
          <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m11 17-5-5 5-5"/><path d="m18 17-5-5 5-5"/></svg>
        </button>
        <button class="btn btn-ghost btn-xs px-3 min-w-[90px] text-sm font-medium" @click="showYearPicker = !showYearPicker">
          {{ displayYear }} 年
        </button>
        <button class="btn btn-ghost btn-xs btn-square" @click="nextYear" title="下一年">
          <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m13 17 5-5-5-5"/><path d="m6 17 5-5-5-5"/></svg>
        </button>
        <button class="btn btn-ghost btn-xs ml-2" @click="goToday">今天</button>
      </div>
    </div>

    <!-- 年份选择器 -->
    <div v-if="showYearPicker" class="mb-3 p-2 bg-base-200 rounded-lg grid grid-cols-6 sm:grid-cols-8 gap-1 max-h-40 overflow-y-auto">
      <button
        v-for="y in yearRange"
        :key="y"
        class="btn btn-xs"
        :class="y === year ? 'btn-primary' : 'btn-ghost'"
        @click="year = y; showYearPicker = false"
      >{{ y }}</button>
    </div>

    <!-- 全年 12 个月网格 -->
    <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
      <div v-for="m in 12" :key="m" class="min-w-0">
        <!-- 月份标题 -->
        <div class="text-xs font-medium text-base-content/60 mb-2 text-center">
          {{ m }} 月
        </div>
        <!-- 星期头 -->
        <div class="grid grid-cols-7 mb-0.5 text-center text-[10px] text-base-content/40">
          <div v-for="w in weekDays" :key="w" class="py-0.5" :class="w === '六' || w === '日' ? 'text-error/50' : ''">{{ w }}</div>
        </div>
        <!-- 日期网格 -->
        <div class="grid grid-cols-7 gap-[1px]">
          <template v-for="(day, idx) in monthGrid(m)" :key="idx">
            <div
              v-if="day"
              class="aspect-square p-0.5 cursor-pointer rounded-sm transition-colors relative flex flex-col items-center justify-center text-center"
              :class="{
                'bg-primary/10 ring-1 ring-primary/30': day.isToday,
                'hover:bg-primary/5': !day.isToday,
                'bg-error/5': day.isHoliday && !day.isToday,
              }"
              @click="selectDay(day)"
            >
              <span
                class="text-[11px] leading-tight"
                :class="{
                  'text-error': day.isSunday || day.isSaturday,
                  'text-base-content': !day.isSunday && !day.isSaturday,
                  'text-primary font-bold': day.isToday,
                  'text-base-content/70': day.festivals.length > 0 && !day.isToday,
                }"
              >{{ day.day }}</span>
              <span
                v-if="day.lunarDay === '初一'"
                class="text-[8px] text-primary/60 leading-tight mt-[1px]"
              >{{ day.lunarMonth }}</span>
              <span
                v-else-if="day.festivals.length > 0"
                class="text-[7px] text-error leading-tight mt-[1px] truncate max-w-full px-0.5"
              >{{ day.festivals[0] }}</span>
              <span
                v-else-if="day.jieQi"
                class="text-[7px] text-info leading-tight mt-[1px]"
              >{{ day.jieQi }}</span>
              <span
                v-else
                class="text-[7px] text-base-content/30 leading-tight mt-[1px]"
              >{{ day.lunarDay }}</span>
            </div>
            <div v-else class="aspect-square"></div>
          </template>
        </div>
      </div>
    </div>

    <!-- 黄历详情弹窗 -->
    <Modal v-model="showAlmanac" :title="almanacTitle" size="lg" class="max-w-2xl">
      <div class="space-y-4 text-sm" v-if="selectedDay">
        <!-- 基本信息 -->
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
          <div class="bg-base-200/50 rounded-lg p-3">
            <div class="text-base-content/50 text-xs mb-1">公历</div>
            <div class="font-medium">{{ selectedDay.solarStr }}</div>
          </div>
          <div class="bg-base-200/50 rounded-lg p-3">
            <div class="text-base-content/50 text-xs mb-1">农历</div>
            <div class="font-medium">{{ selectedDay.lunarStr }}</div>
          </div>
        </div>

        <!-- 干支/生肖/星座 -->
        <div class="flex flex-wrap gap-1.5" v-if="selectedDay.almanac">
          <span class="badge badge-sm badge-ghost">{{ selectedDay.almanac.yearGanZhi }}年</span>
          <span class="badge badge-sm badge-ghost">{{ selectedDay.almanac.monthGanZhi }}月</span>
          <span class="badge badge-sm badge-ghost">{{ selectedDay.almanac.dayGanZhi }}日</span>
          <span class="badge badge-sm badge-ghost">{{ selectedDay.almanac.zodiac }}年</span>
          <span class="badge badge-sm badge-ghost">{{ selectedDay.almanac.constellation }}</span>
          <span class="badge badge-sm badge-ghost">纳音 {{ selectedDay.almanac.naYin }}</span>
        </div>

        <!-- 节气 + 节日 -->
        <div class="flex flex-wrap gap-2">
          <div v-if="selectedDay.jieQi" class="bg-info/10 text-info rounded-lg px-3 py-2 text-sm flex-1 min-w-[120px]">
            📌 节气：{{ selectedDay.jieQi }}
          </div>
          <div v-if="selectedDay.festivals.length > 0" class="bg-error/10 text-error rounded-lg px-3 py-2 text-sm flex-1 min-w-[120px]">
            🎉 节日：{{ selectedDay.festivals.join('、') }}
          </div>
        </div>

        <!-- 宜忌 -->
        <div v-if="selectedDay.almanac" class="grid grid-cols-1 sm:grid-cols-2 gap-3">
          <div>
            <div class="text-xs text-success font-medium mb-1.5">✅ 宜</div>
            <div class="flex flex-wrap gap-1">
              <span
                v-for="item in selectedDay.almanac.yi.slice(0, 15)"
                :key="item"
                class="text-xs bg-success/10 text-success rounded px-1.5 py-0.5"
              >{{ item }}</span>
              <span
                v-if="selectedDay.almanac.yi.length > 15"
                class="text-xs text-base-content/40"
              >+{{ selectedDay.almanac.yi.length - 15 }}项</span>
              <span v-if="selectedDay.almanac.yi.length === 0" class="text-xs text-base-content/40">诸事不宜</span>
            </div>
          </div>
          <div>
            <div class="text-xs text-error font-medium mb-1.5">❌ 忌</div>
            <div class="flex flex-wrap gap-1">
              <span
                v-for="item in selectedDay.almanac.ji"
                :key="item"
                class="text-xs bg-error/10 text-error rounded px-1.5 py-0.5"
              >{{ item }}</span>
              <span v-if="selectedDay.almanac.ji.length === 0" class="text-xs text-base-content/40">无忌</span>
            </div>
          </div>
        </div>

        <!-- 彭祖百忌 / 冲煞 / 吉神 -->
        <div v-if="selectedDay.almanac" class="bg-base-200/50 rounded-lg p-3 text-xs space-y-1.5 leading-relaxed">
          <div><span class="text-base-content/50">彭祖百忌：</span>{{ selectedDay.almanac.pengzu }}</div>
          <div><span class="text-base-content/50">冲煞：</span>{{ selectedDay.almanac.chongSha }}</div>
          <div><span class="text-base-content/50">吉神方位：</span>喜神{{ selectedDay.almanac.gods.xi }} / 福神{{ selectedDay.almanac.gods.fu }} / 财神{{ selectedDay.almanac.gods.cai }}</div>
          <div><span class="text-base-content/50">二十八宿：</span>{{ selectedDay.almanac.xiu }}（{{ selectedDay.almanac.xiuLuck }}）</div>
        </div>

        <!-- 时辰吉凶 -->
        <div v-if="selectedDay.almanac && selectedDay.almanac.timeYi.length > 0">
          <div class="text-xs font-medium text-base-content/70 mb-1.5">⏰ 时辰吉凶</div>
          <div class="grid grid-cols-2 sm:grid-cols-4 gap-1">
            <div
              v-for="t in selectedDay.almanac.timeYi"
              :key="t.type"
              class="text-xs bg-base-200/50 rounded px-2 py-1 flex items-center justify-between"
            >
              <span>{{ t.type }}</span>
              <span :class="t.value === '吉' ? 'text-success font-medium' : t.value === '凶' ? 'text-error' : 'text-warning'">{{ t.value }}</span>
            </div>
          </div>
        </div>
      </div>
      <template #footer>
        <button class="btn btn-primary btn-sm" @click="showAlmanac = false">关闭</button>
      </template>
    </Modal>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import SvgIcon from '../../../components/ui/SvgIcon.vue';
import Modal from '../../../components/ui/Modal.vue';
import LunarJS from 'lunar-javascript';

defineOptions({ name: 'PerpetualCalendar' });

// ===== 状态 =====
const now = new Date();
const year = ref(now.getFullYear());
const showYearPicker = ref(false);
const showAlmanac = ref(false);
const selectedDay = ref<DayInfo | null>(null);

const weekDays = ['日', '一', '二', '三', '四', '五', '六'];
const displayYear = computed(() => `${year.value}年`);

const yearRange = computed(() => {
  const y = year.value;
  return Array.from({ length: 21 }, (_, i) => y - 10 + i);
});

// ===== 日期数据类型 =====
interface AlmanacInfo {
  yi: string[];
  ji: string[];
  yearGanZhi: string;
  monthGanZhi: string;
  dayGanZhi: string;
  zodiac: string;
  constellation: string;
  naYin: string;
  pengzu: string;
  chongSha: string;
  gods: { xi: string; fu: string; cai: string };
  xiu: string;
  xiuLuck: string;
  timeYi: { type: string; value: string }[];
}

interface DayInfo {
  date: Date;
  year: number;
  month: number;
  day: number;
  isToday: boolean;
  isSunday: boolean;
  isSaturday: boolean;
  isHoliday: boolean;
  lunarDay: string;
  lunarMonth: string;
  lunarStr: string;
  solarStr: string;
  festivals: string[];
  jieQi: string | null;
  almanac: AlmanacInfo | null;
}

// 构建单月数据（缓存提升性能）
const monthCache = new Map<string, (DayInfo | null)[]>();

function buildMonthData(y: number, m: number): (DayInfo | null)[] {
  const key = `${y}-${m}`;
  const cached = monthCache.get(key);
  if (cached) return cached;

  const days: (DayInfo | null)[] = [];
  const firstDay = new Date(y, m - 1, 1);
  const totalDays = new Date(y, m, 0).getDate();
  const startWeekday = firstDay.getDay();
  const todayStr = new Date().toDateString();

  // 月度显示 weekStartsOn=0 (Sunday)
  for (let i = 0; i < startWeekday; i++) {
    days.push(null);
  }

  for (let d = 1; d <= totalDays; d++) {
    const solar = LunarJS.Solar.fromYmd(y, m, d);
    const lunar = solar.getLunar();

    const lunarDay = lunar.getDayInChinese();
    const lunarMonthStr = lunar.getMonthInChinese() + '月';

    // 节日
    const festivals: string[] = [];
    festivals.push(
      ...(solar.getFestivals() as string[]),
      ...(solar.getOtherFestivals() as string[]),
      ...(lunar.getFestivals() as string[]),
      ...(lunar.getOtherFestivals() as string[]),
    );

    // 节气
    const jieQi = (lunar.getJieQi() as string) || null;

    // 黄历 — 用正确的 API 方法名
    let almanac: AlmanacInfo | null = null;
    try {
      // 获取时辰吉凶
      const times = lunar.getTimes() as any[];
      const timeYi: { type: string; value: string }[] = [];
      for (const t of times) {
        timeYi.push({
          type: t.getZhiInChinese() as string,
          value: t.getTianShenLuck() as string,
        });
      }

      const pengzuGan = lunar.getPengZuGan() as string || '';
      const pengzuZhi = lunar.getPengZuZhi() as string || '';
      const pengzu = [pengzuGan, pengzuZhi].filter(Boolean).join(' ');

      const chongDesc = lunar.getDayChongDesc() as string || '';
      const sha = lunar.getDaySha() as string || '';

      almanac = {
        yi: lunar.getDayYi() as string[],
        ji: lunar.getDayJi() as string[],
        yearGanZhi: lunar.getYearInGanZhi() as string,
        monthGanZhi: lunar.getMonthInGanZhi() as string,
        dayGanZhi: lunar.getDayInGanZhi() as string,
        zodiac: lunar.getYearShengXiao() as string,
        constellation: solar.getXingZuo() as string,
        naYin: lunar.getDayNaYin() as string,
        pengzu,
        chongSha: `${chongDesc} 煞${sha}`,
        gods: {
          xi: (lunar.getDayPositionXiDesc() as string) || '',
          fu: (lunar.getDayPositionFuDesc() as string) || '',
          cai: (lunar.getDayPositionCaiDesc() as string) || '',
        },
        xiu: (lunar.getXiu() as string) || '',
        xiuLuck: (lunar.getXiuLuck() as string) || '',
        timeYi,
      };
    } catch (e) {
      // 某些日期可能没有完整的黄历数据
    }

    const date = new Date(y, m - 1, d);
    const isHoliday = festivals.length > 0 || date.getDay() === 0 || date.getDay() === 6;

    days.push({
      date,
      year: y,
      month: m,
      day: d,
      isToday: date.toDateString() === todayStr,
      isSunday: date.getDay() === 0,
      isSaturday: date.getDay() === 6,
      isHoliday,
      lunarDay,
      lunarMonth: lunar.getMonth() === 1 ? '正月' : lunarMonthStr,
      solarStr: `${y}年${m}月${d}日 星期${['日', '一', '二', '三', '四', '五', '六'][date.getDay()]}`,
      lunarStr: `农历${lunar.getYearInChinese()}年 ${lunar.getMonthInChinese()}月${lunarDay}`,
      festivals,
      jieQi,
      almanac,
    });
  }

  // 补齐到整周
  const remaining = 7 - (days.length % 7 || 7);
  for (let i = 0; i < remaining; i++) {
    days.push(null);
  }

  monthCache.set(key, days);
  return days;
}

// ===== 月份网格生成器 =====
const monthGrid = (m: number) => {
  return buildMonthData(year.value, m);
};

// ===== 导航 =====
const prevYear = () => { year.value--; showYearPicker.value = false; };
const nextYear = () => { year.value++; showYearPicker.value = false; };
const goToday = () => {
  const n = new Date();
  year.value = n.getFullYear();
  showYearPicker.value = false;
};

// ===== 选日看黄历 =====
const almanacTitle = computed(() => {
  if (!selectedDay.value) return '';
  const d = selectedDay.value;
  return `${d.year}年${d.month}月${d.day}日 黄历`;
});

const selectDay = (day: DayInfo) => {
  selectedDay.value = day;
  showAlmanac.value = true;
};
</script>

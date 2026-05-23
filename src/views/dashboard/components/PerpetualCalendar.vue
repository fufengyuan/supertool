<template>
  <div class="bg-base-100 rounded-xl border border-base-content/10 p-4">
    <!-- 日历头部 -->
    <div class="flex items-center justify-between mb-3">
      <div class="flex items-center gap-2">
        <SvgIcon name="calendar" size="16" class="text-primary" />
        <span class="text-sm font-medium text-base-content/70">万年历</span>
      </div>
      <div class="flex items-center gap-1">
        <button class="btn btn-ghost btn-xs btn-square" @click="prevYear" title="上一年">
          <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m11 17-5-5 5-5"/><path d="m18 17-5-5 5-5"/></svg>
        </button>
        <button class="btn btn-ghost btn-xs btn-square" @click="prevMonth" title="上月">
          <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m15 18-6-6 6-6"/></svg>
        </button>
        <button class="btn btn-ghost btn-xs px-2 min-w-[100px] text-sm font-medium" @click="showYearPicker = !showYearPicker">
          {{ year }} 年 {{ month }} 月
        </button>
        <button class="btn btn-ghost btn-xs btn-square" @click="nextMonth" title="下月">
          <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m9 18 6-6-6-6"/></svg>
        </button>
        <button class="btn btn-ghost btn-xs btn-square" @click="nextYear" title="下一年">
          <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m13 17 5-5-5-5"/><path d="m6 17 5-5-5-5"/></svg>
        </button>
        <button class="btn btn-ghost btn-xs ml-1" @click="today">今天</button>
      </div>
    </div>

    <!-- 年份选择器 -->
    <div v-if="showYearPicker" class="mb-2 p-2 bg-base-200 rounded-lg grid grid-cols-4 gap-1 max-h-[160px] overflow-y-auto">
      <button
        v-for="y in yearRange"
        :key="y"
        class="btn btn-xs"
        :class="y === year ? 'btn-primary' : 'btn-ghost'"
        @click="year = y; showYearPicker = false"
      >{{ y }}</button>
    </div>

    <!-- 星期头 -->
    <div class="grid grid-cols-7 mb-1 text-center text-xs text-base-content/50">
      <div v-for="w in weekDays" :key="w" class="py-1" :class="w === '六' || w === '日' ? 'text-error/60' : ''">{{ w }}</div>
    </div>

    <!-- 日期网格 -->
    <div class="grid grid-cols-7 gap-px bg-base-content/5 rounded-lg overflow-hidden">
      <template v-for="(day, idx) in monthDays" :key="idx">
        <div
          v-if="day"
          class="min-h-[64px] p-1 bg-base-100 cursor-pointer hover:bg-primary/10 transition-colors relative group"
          :class="{
            'bg-primary/5': day.isToday,
            'hover:bg-primary/10': true,
          }"
          @click="selectDay(day)"
        >
          <!-- 公历日期 -->
          <div class="flex items-center justify-between">
            <span
              class="text-xs font-medium leading-none"
              :class="{
                'text-error': day.isSunday || day.isSaturday || day.isHoliday,
                'text-base-content': !day.isSunday && !day.isSaturday && !day.isHoliday,
                'text-base-content/30': !day.isCurrentMonth,
                'text-primary font-bold': day.isToday,
              }"
            >{{ day.day }}</span>
            <span
              v-if="day.lunarDay === '初一'"
              class="text-[10px] text-primary/70 leading-none"
            >{{ day.lunarMonth }}</span>
            <span
              v-else-if="day.festivals.length > 0"
              class="text-[9px] text-error leading-none truncate max-w-[50px] text-right"
            >{{ day.festivals[0] }}</span>
            <span
              v-else
              class="text-[10px] text-base-content/35 leading-none"
            >{{ day.lunarDay }}</span>
          </div>
          <!-- 农历/节日第二行 -->
          <div class="mt-1 flex flex-wrap gap-0.5">
            <span
              v-if="day.festivals.length > 0"
              class="text-[9px] bg-error/10 text-error rounded-sm px-1 leading-tight truncate max-w-full"
            >{{ day.festivals[0] }}</span>
            <span
              v-else-if="day.lunarDay === '初一'"
              class="text-[9px] text-primary/50 leading-tight"
            >{{ day.lunarMonth }}</span>
            <span
              v-else-if="day.jieQi"
              class="text-[9px] bg-info/10 text-info rounded-sm px-1 leading-tight truncate max-w-full"
            >{{ day.jieQi }}</span>
          </div>
          <!-- 黄历状态小点 -->
          <div v-if="day.almanac" class="absolute bottom-0.5 right-0.5 flex gap-0.5">
            <span v-if="day.almanac.yi.length > 0" class="w-1 h-1 rounded-full bg-success/60"></span>
            <span v-if="day.almanac.ji.length > 0" class="w-1 h-1 rounded-full bg-error/60"></span>
          </div>
        </div>
        <div v-else class="min-h-[64px] bg-base-content/[0.02]"></div>
      </template>
    </div>

    <!-- 黄历详情弹窗 -->
    <Modal v-model="showAlmanac" :title="almanacTitle">
      <div class="space-y-4 text-sm" v-if="selectedDay">
        <!-- 基本信息 -->
        <div class="grid grid-cols-2 gap-3">
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
        <div class="flex flex-wrap gap-2" v-if="selectedDay.almanac">
          <span class="badge badge-sm badge-ghost">{{ selectedDay.almanac.yearGanZhi }}年</span>
          <span class="badge badge-sm badge-ghost">{{ selectedDay.almanac.monthGanZhi }}月</span>
          <span class="badge badge-sm badge-ghost">{{ selectedDay.almanac.dayGanZhi }}日</span>
          <span class="badge badge-sm badge-ghost">{{ selectedDay.almanac.zodiac }}年</span>
          <span class="badge badge-sm badge-ghost">{{ selectedDay.almanac.constellation }}</span>
        </div>

        <!-- 二十四节气 -->
        <div v-if="selectedDay.jieQi" class="bg-info/10 text-info rounded-lg px-3 py-2 text-sm">
          📌 今日节气：{{ selectedDay.jieQi }}
        </div>

        <!-- 节日 -->
        <div v-if="selectedDay.festivals.length > 0" class="bg-error/10 text-error rounded-lg px-3 py-2 text-sm">
          🎉 节日：{{ selectedDay.festivals.join('、') }}
        </div>

        <!-- 宜忌 -->
        <div v-if="selectedDay.almanac" class="grid grid-cols-2 gap-3">
          <div>
            <div class="text-xs text-success font-medium mb-1.5">✅ 宜</div>
            <div class="flex flex-wrap gap-1">
              <span
                v-for="item in selectedDay.almanac.yi"
                :key="item"
                class="text-xs bg-success/10 text-success rounded px-1.5 py-0.5"
              >{{ item }}</span>
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

        <!-- 彭祖百忌 / 冲煞 -->
        <div v-if="selectedDay.almanac" class="bg-base-200/50 rounded-lg p-3 text-xs space-y-1">
          <div><span class="text-base-content/50">彭祖百忌：</span>{{ selectedDay.almanac.pengzu }}</div>
          <div><span class="text-base-content/50">冲煞：</span>{{ selectedDay.almanac.chongSha }}</div>
          <div><span class="text-base-content/50">吉神方位：</span>喜神{{ selectedDay.almanac.gods.xi }}/ 福神{{ selectedDay.almanac.gods.fu }}/ 财神{{ selectedDay.almanac.gods.cai }}</div>
        </div>

        <!-- 时辰吉凶 -->
        <div v-if="selectedDay.almanac && selectedDay.almanac.timeYi.length > 0">
          <div class="text-xs font-medium text-base-content/70 mb-1.5">⏰ 时辰吉凶</div>
          <div class="grid grid-cols-4 gap-1">
            <div
              v-for="t in selectedDay.almanac.timeYi"
              :key="t.type"
              class="text-xs bg-base-200/50 rounded px-2 py-1 flex items-center justify-between"
            >
              <span>{{ t.type }}</span>
              <span :class="t.value === '吉' ? 'text-success' : t.value === '凶' ? 'text-error' : 'text-warning'">{{ t.value }}</span>
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
import { ref, computed, watch } from 'vue';
import SvgIcon from '../../../components/ui/SvgIcon.vue';
import Modal from '../../../components/ui/Modal.vue';
import LunarJS from 'lunar-javascript';

defineOptions({ name: 'PerpetualCalendar' });

// ===== 状态 =====
const now = new Date();
const year = ref(now.getFullYear());
const month = ref(now.getMonth() + 1);
const showYearPicker = ref(false);
const showAlmanac = ref(false);
const selectedDay = ref<DayInfo | null>(null);

const weekDays = ['日', '一', '二', '三', '四', '五', '六'];
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
  pengzu: string;
  chongSha: string;
  gods: { xi: string; fu: string; cai: string };
  timeYi: { type: string; value: string }[];
}

interface DayInfo {
  date: Date;
  year: number;
  month: number;
  day: number;
  isCurrentMonth: boolean;
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

// ===== 生成月份日期 =====
const monthDays = computed(() => {
  const days: (DayInfo | null)[] = [];

  const firstDay = new Date(year.value, month.value - 1, 1);
  const lastDay = new Date(year.value, month.value, 0);
  const totalDays = lastDay.getDate();
  const startWeekday = firstDay.getDay(); // 0=Sun

  // 上月补齐
  const prevLastDay = new Date(year.value, month.value - 1, 0).getDate();
  for (let i = startWeekday - 1; i >= 0; i--) {
    days.push(null);
  }

  const todayStr = new Date().toDateString();

  for (let d = 1; d <= totalDays; d++) {
    const date = new Date(year.value, month.value - 1, d);
    const solar = LunarJS.Solar.fromYmd(year.value, month.value, d);
    const lunar = solar.getLunar();

    const lunarDay = lunar.getDayInChinese(); // 初一, 初二...
    const lunarMonth = lunar.getMonthInChinese() + '月';

    // 节日
    const festivals: string[] = [];
    const solarFests = solar.getFestivals() as string[];
    const otherSolarFests = solar.getOtherFestivals() as string[];
    const lunarFests = lunar.getFestivals() as string[];
    const otherLunarFests = lunar.getOtherFestivals() as string[];
    festivals.push(...solarFests, ...otherSolarFests, ...lunarFests, ...otherLunarFests);

    // 节气
    const jieQi = lunar.getJieQi() as string || null;

    // 黄历
    let almanac: AlmanacInfo | null = null;
    try {
      almanac = {
        yi: lunar.getDayYi() as string[],
        ji: lunar.getDayJi() as string[],
        yearGanZhi: lunar.getYearInGanZhi() as string,
        monthGanZhi: lunar.getMonthInGanZhi() as string,
        dayGanZhi: lunar.getDayInGanZhi() as string,
        zodiac: lunar.getYearShengXiao() as string,
        constellation: solar.getXingZuo() as string,
        pengzu: lunar.getDayPengZu() as string,
        chongSha: (lunar.getDayChong() as string) + (lunar.getDaySha() as string),
        gods: {
          xi: lunar.getDayXiShen() as string,
          fu: lunar.getDayFuShen() as string,
          cai: lunar.getDayCaiShen() as string,
        },
        timeYi: (lunar.getTimeYi() as { type: string; value: string }[]) || [],
      };
    } catch (_) {
      // 某些日期可能没有黄历数据
    }

    const isHoliday = festivals.length > 0 || date.getDay() === 0 || date.getDay() === 6;

    days.push({
      date,
      year: year.value,
      month: month.value,
      day: d,
      isCurrentMonth: true,
      isToday: date.toDateString() === todayStr,
      isSunday: date.getDay() === 0,
      isSaturday: date.getDay() === 6,
      isHoliday,
      lunarDay,
      lunarMonth: lunar.getMonth() === 1 ? '正月' : lunarMonth,
      solarStr: `${year.value}年${month.value}月${d}日 ${['日', '一', '二', '三', '四', '五', '六'][date.getDay()]}`,
      lunarStr: `农历${lunar.getYearInChinese()}年 ${lunar.getMonthInChinese()}月${lunarDay}`,
      festivals,
      jieQi,
      almanac,
    });
  }

  // 下月补齐（确保最后一行完整）
  const remaining = 7 - (days.length % 7 || 7);
  for (let i = 0; i < remaining; i++) {
    days.push(null);
  }

  return days;
});

// ===== 导航 =====
const prevMonth = () => {
  if (month.value === 1) { year.value--; month.value = 12; }
  else { month.value--; }
  showYearPicker.value = false;
};

const nextMonth = () => {
  if (month.value === 12) { year.value++; month.value = 1; }
  else { month.value++; }
  showYearPicker.value = false;
};

const prevYear = () => { year.value--; showYearPicker.value = false; };
const nextYear = () => { year.value++; showYearPicker.value = false; };

const today = () => {
  const n = new Date();
  year.value = n.getFullYear();
  month.value = n.getMonth() + 1;
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

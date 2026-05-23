declare module 'lunar-javascript' {
  export class Solar {
    static fromYmd(year: number, month: number, day: number): Solar;
    toFullString(): string;
    toString(): string;
    getYear(): number;
    getMonth(): number;
    getDay(): number;
    getWeek(): number;
    getWeekIndex(): number;
    getDaysInMonth(): number;
    getFestivals(): string[];
    getOtherFestivals(): string[];
    getXingZuo(): string;
    getLunar(): Lunar;
  }

  export class Lunar {
    getYear(): number;
    getMonth(): number;
    getDay(): number;
    getYearInChinese(): string;
    getMonthInChinese(): string;
    getDayInChinese(): string;
    getYearShengXiao(): string;
    getYearInGanZhi(): string;
    getMonthInGanZhi(): string;
    getDayInGanZhi(): string;
    getDayYi(): string[];
    getDayJi(): string[];
    getFestivals(): string[];
    getOtherFestivals(): string[];
    getJieQi(): string;
    getTimes(): any[];
    getPengZuGan(): string;
    getPengZuZhi(): string;
    getDayChong(): string;
    getDayChongDesc(): string;
    getDaySha(): string;
    getDayNaYin(): string;
    getDayPositionXi(): string;
    getDayPositionXiDesc(): string;
    getDayPositionFu(): string;
    getDayPositionFuDesc(): string;
    getDayPositionCai(): string;
    getDayPositionCaiDesc(): string;
    getXiu(): string;
    getXiuLuck(): string;
    getYearShengXiaoByLiChun(): string;
  }

  export class LunarTime {
    getZhiInChinese(): string;
    getTianShenLuck(): string;
  }

  export class HolidayUtil {
    static getHoliday(year: number, month: number, day: number): string;
    static getDay(year: number, month: number, day: number, info: string): string;
  }
}

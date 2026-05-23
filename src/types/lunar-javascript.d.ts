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
    getDayPengZu(): string;
    getDayChong(): string;
    getDaySha(): string;
    getDayXiShen(): string;
    getDayFuShen(): string;
    getDayCaiShen(): string;
    getTimeYi(): { type: string; value: string }[];
    getJieQi(): string;
    getFestivals(): string[];
    getOtherFestivals(): string[];
  }
}

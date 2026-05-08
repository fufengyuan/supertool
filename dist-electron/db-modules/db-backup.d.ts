import { type WeeklyReport, type ExportData } from './db-core';
declare function saveWeeklyReport(report: {
    startDate: string;
    endDate: string;
    data: Record<string, unknown>;
}): {
    success: boolean;
    id: number;
};
declare function getWeeklyReports(limit?: number): any[];
declare function getWeeklyReport(id: number): WeeklyReport | null;
declare function exportAllData(): ExportData;
declare function importAllData(data: ExportData, mode?: 'replace' | 'merge'): {
    imported: number;
    skipped: number;
};
declare const _default: {
    saveWeeklyReport: typeof saveWeeklyReport;
    getWeeklyReports: typeof getWeeklyReports;
    getWeeklyReport: typeof getWeeklyReport;
    exportAllData: typeof exportAllData;
    importAllData: typeof importAllData;
};
export = _default;

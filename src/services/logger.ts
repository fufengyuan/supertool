// @ts-nocheck
/**
 * logger.ts — 日志服务
 *
 * - LogLevel: DEBUG / INFO / WARN / ERROR
 * - 内存日志缓冲区 (最近 100 条)
 * - 开发环境 console 输出, 生产环境写入文件 (通过 Tauri API)
 */
import type { LogEntry } from '../types';
import { getTauriAPI } from '../utils/tauri-api'

// ========== 日志级别 ==========
export const LogLevel = {
  DEBUG: 0,
  INFO: 1,
  WARN: 2,
  ERROR: 3,
} as const;

export type LogLevelValue = typeof LogLevel[keyof typeof LogLevel];

const LEVEL_LABELS: Record<LogLevelValue, string> = {
  [LogLevel.DEBUG]: 'DEBUG',
  [LogLevel.INFO]: 'INFO',
  [LogLevel.WARN]: 'WARN',
  [LogLevel.ERROR]: 'ERROR',
};

const isDev = import.meta.env?.DEV ?? true;

// ========== 内存缓冲区 (环形, 最多 100 条) ==========
const MAX_LOG_ENTRIES = 100;
const buffer: (LogEntry | null)[] = [];
let writeIndex = 0;

function pushToBuffer(entry: LogEntry): void {
  if (buffer.length < MAX_LOG_ENTRIES) {
    buffer.push(entry);
  } else {
    buffer[writeIndex] = entry;
  }
  writeIndex = (writeIndex + 1) % MAX_LOG_ENTRIES;
}

// ========== 核心 log 函数 ==========
export function log(message: string, level: LogLevelValue = LogLevel.INFO, context: string = ''): LogEntry {
  const entry: LogEntry = {
    timestamp: new Date().toISOString(),
    level,
    levelLabel: LEVEL_LABELS[level] ?? 'INFO',
    message: String(message),
    context: context ? String(context) : '',
  };

  pushToBuffer(entry);

  // 开发环境输出到 console
  if (isDev) {
    const prefix = context ? `[${entry.levelLabel}][${context}]` : `[${entry.levelLabel}]`;
    const consoleFn =
      level === LogLevel.ERROR
        ? console.error
        : level === LogLevel.WARN
          ? console.warn
          : level === LogLevel.DEBUG
            ? console.debug
            : console.log;
    consoleFn(`${prefix} ${entry.message}`);
  }

  // 生产环境通过 Tauri API 写入文件 (仅 ERROR/WARN)
  if (!isDev && level >= LogLevel.WARN) {
    try {
      getTauriAPI().writeLogFile?.(JSON.stringify(entry));
    } catch {
      // 忽略写入失败
    }
  }

  return entry;
}

// ========== 便捷函数 ==========
export const debug = (message: string, context?: string): LogEntry => log(message, LogLevel.DEBUG, context ?? '');
export const info = (message: string, context?: string): LogEntry => log(message, LogLevel.INFO, context ?? '');
export const warn = (message: string, context?: string): LogEntry => log(message, LogLevel.WARN, context ?? '');
export const error = (message: string, context?: string): LogEntry => log(message, LogLevel.ERROR, context ?? '');

// ========== 日志查询 ==========
export function getLogs(minLevel: LogLevelValue = LogLevel.DEBUG): LogEntry[] {
  const entries = buffer.filter(Boolean) as LogEntry[];
  if (entries.length === 0) return [];

  // 按写入顺序排列
  if (entries.length < MAX_LOG_ENTRIES) {
    return entries.filter((e) => e.level >= minLevel);
  }
  // 环形缓冲已满, 需要重新排序
  const ordered = [
    ...buffer.slice(writeIndex),
    ...buffer.slice(0, writeIndex),
  ].filter(Boolean) as LogEntry[];
  return ordered.filter((e) => e.level >= minLevel);
}

export function clearLogs(): void {
  buffer.length = 0;
  writeIndex = 0;
}

export function getLogCount(minLevel: LogLevelValue = LogLevel.DEBUG): number {
  return getLogs(minLevel).length;
}

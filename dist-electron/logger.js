"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.debug = debug;
exports.info = info;
exports.warn = warn;
exports.error = error;
/**
 * Simple logger for Electron main process.
 * Only outputs in development mode.
 */
const IS_DEV = process.env.NODE_ENV === 'development' ||
    process.env.ELECTRON_IS_DEV === '1' ||
    !process.env.FORCE_PROD;
function format(level, context, message) {
    return context ? `[${level}][${context}] ${message}` : `[${level}] ${message}`;
}
function debug(message, ...args) {
    if (!IS_DEV)
        return;
    const extra = args.length > 0 ? ' ' + args.map(a => typeof a === 'string' ? a : JSON.stringify(a)).join(' ') : '';
    console.debug(`[DEBUG] ${message}${extra}`);
}
function info(message, ...args) {
    if (!IS_DEV)
        return;
    const extra = args.length > 0 ? ' ' + args.map(a => typeof a === 'string' ? a : JSON.stringify(a)).join(' ') : '';
    console.log(`[INFO] ${message}${extra}`);
}
function warn(message, ...args) {
    if (!IS_DEV)
        return;
    const extra = args.length > 0 ? ' ' + args.map(a => typeof a === 'string' ? a : JSON.stringify(a)).join(' ') : '';
    console.warn(`[WARN] ${message}${extra}`);
}
function error(message, ...args) {
    if (!IS_DEV)
        return;
    const extra = args.length > 0 ? ' ' + args.map(a => typeof a === 'string' ? a : JSON.stringify(a)).join(' ') : '';
    console.error(`[ERROR] ${message}${extra}`);
}
//# sourceMappingURL=logger.js.map
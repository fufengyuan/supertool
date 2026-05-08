"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.generateTOTP = generateTOTP;
exports.getRemainingTime = getRemainingTime;
exports.validateBase32 = validateBase32;
exports.totpUri = totpUri;
exports.parseOtpAuthUri = parseOtpAuthUri;
exports.formatCode = formatCode;
/**
 * TOTP (Time-based One-Time Password) implementation — RFC 6238
 * Pure Node.js implementation using built-in crypto module.
 * Used in Electron main process only.
 */
const crypto = __importStar(require("crypto"));
/**
 * Decode a Base32-encoded string to a Buffer.
 * Handles standard Base32 (A-Z, 2-7) with optional padding (=).
 */
function base32Decode(encoded) {
    // Normalize: uppercase, remove spaces and padding
    const clean = encoded.toUpperCase().replace(/[=\s]/g, '');
    // Base32 alphabet
    const alphabet = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ234567';
    const charMap = new Map();
    for (let i = 0; i < alphabet.length; i++) {
        charMap.set(alphabet[i], i);
    }
    // Decode to bits
    const bits = [];
    for (const char of clean) {
        const value = charMap.get(char);
        if (value === undefined) {
            throw new Error(`Invalid Base32 character: ${char}`);
        }
        // Each Base32 char = 5 bits
        bits.push((value >> 4) & 1, (value >> 3) & 1, (value >> 2) & 1, (value >> 1) & 1, value & 1);
    }
    // Pad to multiple of 8
    while (bits.length % 8 !== 0) {
        bits.push(0);
    }
    // Convert bits to bytes
    const bytes = [];
    for (let i = 0; i < bits.length; i += 8) {
        let byte = 0;
        for (let j = 0; j < 8; j++) {
            byte = (byte << 1) | bits[i + j];
        }
        bytes.push(byte);
    }
    return Buffer.from(bytes);
}
/**
 * HMAC-SHA1 (default), HMAC-SHA256, or HMAC-SHA512
 */
function hmacDigest(algorithm, key, message) {
    const algoMap = {
        sha1: 'sha1',
        sha256: 'sha256',
        sha512: 'sha512',
    };
    const algo = algoMap[algorithm.toLowerCase()] || 'sha1';
    return crypto.createHmac(algo, key).update(message).digest();
}
/**
 * Generate a TOTP code for the given secret and time.
 *
 * @param secretBase32 - Base32-encoded shared secret
 * @param digits - Number of digits (default 6)
 * @param period - Time step in seconds (default 30)
 * @param algorithm - HMAC algorithm: 'sha1' | 'sha256' | 'sha512' (default 'sha1')
 * @param timeOffset - Time offset in seconds (for testing)
 * @returns The TOTP code as a zero-padded string
 */
function generateTOTP(secretBase32, digits = 6, period = 30, algorithm = 'sha1', timeOffset = 0) {
    // Decode secret
    const key = base32Decode(secretBase32);
    // Current time step
    const epoch = Math.floor((Date.now() + timeOffset * 1000) / 1000);
    const timeStep = Math.floor(epoch / period);
    // Time as 8-byte big-endian buffer
    const timeBuffer = Buffer.alloc(8);
    let temp = timeStep;
    for (let i = 7; i >= 0; i--) {
        timeBuffer[i] = temp & 0xff;
        temp = Math.floor(temp / 256);
    }
    // HMAC
    const hmac = hmacDigest(algorithm, key, timeBuffer);
    // Dynamic truncation (RFC 4226 Section 5.4)
    const offset = hmac[hmac.length - 1] & 0x0f;
    const code = ((hmac[offset] & 0x7f) << 24) |
        ((hmac[offset + 1] & 0xff) << 16) |
        ((hmac[offset + 2] & 0xff) << 8) |
        (hmac[offset + 3] & 0xff);
    // Generate digits
    const modulus = Math.pow(10, digits);
    const otp = code % modulus;
    // Zero-pad
    return otp.toString().padStart(digits, '0');
}
/**
 * Get the remaining seconds until the current TOTP code expires.
 */
function getRemainingTime(period = 30) {
    const epoch = Math.floor(Date.now() / 1000);
    return period - (epoch % period);
}
/**
 * Validate a Base32 secret string.
 */
function validateBase32(secret) {
    const clean = secret.toUpperCase().replace(/[=\s]/g, '');
    return /^[A-Z2-7]+$/.test(clean) && clean.length >= 8;
}
/**
 * Generate a URI for QR code (otpauth://totp/...).
 */
function totpUri(secret, account = '', issuer = '', digits = 6, period = 30, algorithm = 'sha1') {
    const params = new URLSearchParams();
    params.set('secret', secret);
    if (issuer)
        params.set('issuer', issuer);
    if (digits !== 6)
        params.set('digits', String(digits));
    if (period !== 30)
        params.set('period', String(period));
    if (algorithm.toLowerCase() !== 'sha1')
        params.set('algorithm', algorithm.toUpperCase());
    const label = issuer ? `${encodeURIComponent(issuer)}:${encodeURIComponent(account || '')}` : encodeURIComponent(account || 'Unknown');
    return `otpauth://totp/${label}?${params.toString()}`;
}
/**
 * Parse an otpauth:// TOTP URI and extract all parameters.
 * Supports both standard and Google Authenticator formats.
 * Returns null if the URI is invalid or not a TOTP URI.
 */
function parseOtpAuthUri(uri) {
    try {
        const url = new URL(uri);
        if (url.protocol !== 'otpauth:' || url.hostname !== 'totp')
            return null;
        // Parse label: pathname starts with "/"
        const label = decodeURIComponent(url.pathname.slice(1));
        if (!label)
            return null;
        let issuer = '';
        let account = '';
        if (label.includes(':')) {
            const [first, ...rest] = label.split(':');
            issuer = first.trim();
            account = rest.join(':').trim();
        }
        else {
            account = label.trim();
        }
        // issuer param overrides label issuer (only if non-empty)
        const urlIssuer = url.searchParams.get('issuer');
        if (urlIssuer && !issuer)
            issuer = urlIssuer;
        const secret = url.searchParams.get('secret') || '';
        const digits = parseInt(url.searchParams.get('digits') || '6', 10);
        const period = parseInt(url.searchParams.get('period') || '30', 10);
        const algorithm = (url.searchParams.get('algorithm') || 'sha1').toLowerCase();
        return {
            secret: secret.toUpperCase().replace(/[=\s]/g, ''),
            issuer,
            account,
            digits: isNaN(digits) ? 6 : digits,
            period: isNaN(period) ? 30 : period,
            algorithm: ['sha1', 'sha256', 'sha512'].includes(algorithm) ? algorithm : 'sha1',
        };
    }
    catch {
        return null;
    }
}
/**
 * Format a TOTP code with spaces for readability (Google style).
 * 6 digits → "123 456"
 * 8 digits → "1234 5678"
 */
function formatCode(code) {
    if (code.length === 6)
        return `${code.slice(0, 3)} ${code.slice(3)}`;
    if (code.length === 8)
        return `${code.slice(0, 4)} ${code.slice(4)}`;
    return code;
}
//# sourceMappingURL=totp.js.map
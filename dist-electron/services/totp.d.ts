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
export declare function generateTOTP(secretBase32: string, digits?: number, period?: number, algorithm?: string, timeOffset?: number): string;
/**
 * Get the remaining seconds until the current TOTP code expires.
 */
export declare function getRemainingTime(period?: number): number;
/**
 * Validate a Base32 secret string.
 */
export declare function validateBase32(secret: string): boolean;
/**
 * Generate a URI for QR code (otpauth://totp/...).
 */
export declare function totpUri(secret: string, account?: string, issuer?: string, digits?: number, period?: number, algorithm?: string): string;
/**
 * Parse an otpauth:// TOTP URI and extract all parameters.
 * Supports both standard and Google Authenticator formats.
 * Returns null if the URI is invalid or not a TOTP URI.
 */
export declare function parseOtpAuthUri(uri: string): {
    secret: string;
    issuer: string;
    account: string;
    digits: number;
    period: number;
    algorithm: string;
} | null;
/**
 * Format a TOTP code with spaces for readability (Google style).
 * 6 digits → "123 456"
 * 8 digits → "1234 5678"
 */
export declare function formatCode(code: string): string;

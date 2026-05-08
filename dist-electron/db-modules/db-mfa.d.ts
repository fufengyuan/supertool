import { type MfaSecretRecord } from './db-core';
export declare function getAllMfaSecrets(): MfaSecretRecord[];
export declare function addMfaSecret(data: {
    id?: string;
    name: string;
    secret: string;
    digits?: number;
    period?: number;
    algorithm?: string;
    account?: string;
    issuer?: string;
}): MfaSecretRecord;
export declare function updateMfaSecret(id: string, updates: {
    name?: string;
    account?: string;
    issuer?: string;
}): MfaSecretRecord | null;
export declare function deleteMfaSecret(id: string): boolean;

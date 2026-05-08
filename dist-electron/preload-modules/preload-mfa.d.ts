declare const _default: {
    getMfaSecrets: () => Promise<any>;
    addMfaSecret: (data: {
        name: string;
        secret: string;
        digits?: number;
        period?: number;
        algorithm?: string;
        account?: string;
        issuer?: string;
    }) => Promise<any>;
    updateMfaSecret: (id: string, updates: {
        name?: string;
        account?: string;
        issuer?: string;
    }) => Promise<any>;
    deleteMfaSecret: (id: string) => Promise<any>;
    generateMfaCode: (secret: string, digits: number, period: number, algorithm: string) => Promise<any>;
    parseOtpAuthUri: (uri: string) => Promise<any>;
};
export default _default;

import { type ApiRequest, type LogPresetRecord } from './db-core';
declare function getLogPresets(): LogPresetRecord[];
declare function addLogPreset(name: string, serverIds: string[], logPath: string, logType: string, keywords: string[], maxLines: number, presetGroup?: string): {
    id: string;
};
declare function updateLogPreset(id: string, updates: {
    name?: string;
    presetGroup?: string;
    serverIds?: string[];
    logPath?: string;
    logType?: string;
    keywords?: string[];
    maxLines?: number;
}): void;
declare function deleteLogPreset(id: string): {
    success: boolean;
    error?: string;
};
declare function getCalculatorHistory(limit?: number): Array<{
    id: string;
    expression: string;
    result: string;
    createdAt: string;
}>;
declare function addCalculatorHistory(expression: string, result: string): {
    id: string;
    expression: string;
    result: string;
    createdAt: string;
};
declare function clearCalculatorHistory(): void;
declare function rowToApiRequest(row: any): ApiRequest;
declare function addApiRequest(request: ApiRequest): ApiRequest;
declare function getApiRequests(): ApiRequest[];
declare function getApiRequestById(id: string): ApiRequest | null;
declare function updateApiRequest(id: string, updates: Partial<ApiRequest>): ApiRequest | null;
declare function deleteApiRequest(id: string): {
    success: boolean;
};
declare const _default: {
    getLogPresets: typeof getLogPresets;
    addLogPreset: typeof addLogPreset;
    updateLogPreset: typeof updateLogPreset;
    deleteLogPreset: typeof deleteLogPreset;
    getCalculatorHistory: typeof getCalculatorHistory;
    addCalculatorHistory: typeof addCalculatorHistory;
    clearCalculatorHistory: typeof clearCalculatorHistory;
    rowToApiRequest: typeof rowToApiRequest;
    addApiRequest: typeof addApiRequest;
    getApiRequests: typeof getApiRequests;
    getApiRequestById: typeof getApiRequestById;
    updateApiRequest: typeof updateApiRequest;
    deleteApiRequest: typeof deleteApiRequest;
};
export = _default;

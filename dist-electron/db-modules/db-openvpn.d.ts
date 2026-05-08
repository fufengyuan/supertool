import { type OpenVPNConfigRecord } from './db-core';
declare function getOpenVPNConfigs(): OpenVPNConfigRecord[];
declare function addOpenVPNConfig(name: string, filePath: string, content: string): {
    id: string;
};
declare function updateOpenVPNConfig(id: string, updates: {
    name?: string;
    filePath?: string;
    content?: string;
}): void;
declare function deleteOpenVPNConfig(id: string): {
    success: boolean;
    error?: string;
};
declare const _default: {
    getOpenVPNConfigs: typeof getOpenVPNConfigs;
    addOpenVPNConfig: typeof addOpenVPNConfig;
    updateOpenVPNConfig: typeof updateOpenVPNConfig;
    deleteOpenVPNConfig: typeof deleteOpenVPNConfig;
};
export = _default;

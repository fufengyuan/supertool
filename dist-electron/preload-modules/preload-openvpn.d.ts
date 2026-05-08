declare const _default: {
    openvpnGetAll: () => Promise<any>;
    openvpnAdd: (data: {
        name: string;
        filePath: string;
        content: string;
    }) => Promise<any>;
    openvpnDelete: (id: string) => Promise<any>;
    openvpnConnect: (configId: string, configName: string, content: string, sudoPassword?: string) => Promise<any>;
    openvpnRetryWithPassword: (password: string) => Promise<any>;
    openvpnDisconnect: () => Promise<any>;
    openvpnGetStatus: () => Promise<any>;
    openvpnGetLogs: () => Promise<any>;
    openvpnCheckAvailable: () => Promise<any>;
    openvpnValidateConfig: (content: string) => Promise<any>;
    openvpnGetTrafficStats: () => Promise<any>;
    importOvpnFile: () => Promise<any>;
    readFileContent: (filePath: string) => Promise<any>;
};
export default _default;

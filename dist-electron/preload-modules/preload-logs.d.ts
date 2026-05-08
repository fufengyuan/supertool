declare const _default: {
    logPresetsGetAll: () => Promise<any>;
    logPresetsAdd: (data: {
        name: string;
        serverIds: string[];
        logPath: string;
        logType: string;
        keywords: string[];
        maxLines: number;
    }) => Promise<any>;
    logPresetsUpdate: (id: string, updates: any) => Promise<any>;
    logPresetsDelete: (id: string) => Promise<any>;
    logsStartStream: (streamId: string, params: {
        serverIds: string[];
        command: string;
    }) => Promise<any>;
    logsStopStream: (streamId: string) => Promise<any>;
    logsSearch: (params: {
        serverIds: string[];
        logType: string;
        logPath: string;
        keyword: string;
        contextLines: number;
    }) => Promise<any>;
    onLogsLine: (callback: (data: {
        streamId: string;
        serverId: string;
        serverName: string;
        line: string;
    }) => void) => () => Electron.IpcRenderer;
    onLogsServerEnd: (callback: (data: {
        streamId: string;
        serverId: string;
    }) => void) => () => Electron.IpcRenderer;
    onLogsError: (callback: (data: {
        streamId: string;
        serverId: string;
        error: string;
    }) => void) => () => Electron.IpcRenderer;
    onLogsStreamStopped: (callback: (data: {
        streamId: string;
    }) => void) => () => Electron.IpcRenderer;
};
export default _default;

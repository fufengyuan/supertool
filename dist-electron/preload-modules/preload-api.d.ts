declare const _default: {
    apiHttpRequest: (config: {
        method: string;
        url: string;
        headers: Record<string, string>;
        body?: string;
        contentType?: string;
        timeout?: number;
    }) => Promise<any>;
    apiRequestsGetAll: () => Promise<any>;
    apiRequestsAdd: (request: {
        name?: string;
        method: string;
        url: string;
        headers?: string;
        body?: string;
        contentType?: string;
    }) => Promise<any>;
    apiRequestsUpdate: (id: string, updates: Record<string, unknown>) => Promise<any>;
    apiRequestsDelete: (id: string) => Promise<any>;
};
export default _default;

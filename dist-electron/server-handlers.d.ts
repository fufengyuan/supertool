export declare function startServerHeartbeat(): void;
export declare function stopServerHeartbeat(): void;
export declare function getServerService(): any;
export declare function setServerService(s: any): void;
export declare function registerServerHandlers(db: any, requireService: (name: string) => any, notifyDataChange: (type: string, data?: unknown) => void): void;

export declare function getDeployAbortControllers(): Map<string, AbortController>;
export declare function cicdDeploy(db: any, configId: string, streamCallback?: (event: {
    type: string;
    data: any;
}) => void): Promise<{
    success: boolean;
    deployLogId?: string;
    error?: string;
}>;
export declare function cicdCancelDeploy(deployLogId: string): Promise<{
    success: boolean;
    message?: string;
    error?: string;
}>;
export declare function cicdRollback(db: any, configId: string, deployHistoryId: string): Promise<{
    success: boolean;
    error?: string;
}>;
export declare function registerCicdHandlers(db: any, notifyDataChange: (type: string, data?: unknown) => void): void;

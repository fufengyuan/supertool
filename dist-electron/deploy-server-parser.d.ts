interface DeployServerConfig {
    host: string;
    port: number;
    username: string;
    password?: Buffer | string;
    privateKey?: Buffer;
    deployDir: string;
    libDir?: string | null;
}
export declare function parseDeployServers(serversJson: string | undefined | null, globalDeployPath: string, libSeparate?: boolean): DeployServerConfig[] | {
    error: string;
};
export {};

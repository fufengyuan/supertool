export interface OpenVPNStatus {
    connected: boolean;
    configId: string | null;
    configName: string | null;
    state: 'disconnected' | 'connecting' | 'connected' | 'error' | 'disconnecting' | 'password_required';
    log: string[];
    connectedSince?: string;
    remote?: string;
    /** Bytes sent */
    bytesSent?: number;
    /** Bytes received */
    bytesReceived?: number;
}
export interface TrafficStats {
    bytesSent: number;
    bytesReceived: number;
    bytesSentHuman: string;
    bytesReceivedHuman: string;
}
declare class OpenVPNManager {
    private process;
    private status;
    private tempConfigPath;
    private logBuffer;
    private readonly MAX_LOG_LINES;
    private pendingConnect;
    private mgmtSocket;
    private mgmtSocketPath;
    private cachedSudoPassword;
    /**
     * Get the path to the bundled OpenVPN binary
     */
    private getBundledPath;
    checkAvailable(): Promise<{
        available: boolean;
        error?: string;
    }>;
    /**
     * Validate .ovpn config content for common issues
     */
    validateConfig(content: string): {
        valid: boolean;
        error?: string;
    };
    /**
     * Check if sudo requires a password for the current user.
     */
    checkSudoNeedsPassword(): Promise<{
        needsPassword: boolean;
    }>;
    /**
     * Cache sudo password for this session auto-reconnect.
     * Cleared on disconnect for security.
     */
    cacheSudoPassword(password: string): void;
    getStatus(): OpenVPNStatus;
    getTrafficStats(): TrafficStats | null;
    private humanBytes;
    private addLog;
    /**
     * Connect to OpenVPN management socket for real-time stats.
     */
    private connectManagement;
    /**
     * Close management socket
     */
    private closeManagement;
    connect(configId: string, configName: string, content: string, sudoPassword?: string): Promise<{
        success: boolean;
        error?: string;
        needsPassword?: boolean;
    }>;
    private spawnOpenVPN;
    /**
     * Retry connection with a password
     */
    retryWithPassword(password: string): Promise<{
        success: boolean;
        error?: string;
    }>;
    disconnect(): Promise<{
        success: boolean;
        error?: string;
    }>;
    private cleanup;
}
export declare const openVPNManager: OpenVPNManager;
export {};

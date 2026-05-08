export declare function performAutoBackup(): Promise<void>;
export declare function scheduleAutoBackup(): void;
export declare function stopAutoBackup(): void;
export declare function setBackupTimer(t: ReturnType<typeof setTimeout> | undefined): void;
export declare function getBackupTimer(): ReturnType<typeof setTimeout> | undefined;

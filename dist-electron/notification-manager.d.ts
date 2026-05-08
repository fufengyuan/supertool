export declare function playNotificationSound(): void;
export declare function dismissNotification(todoId?: string): void;
export declare function isNotified(todoId: string): boolean;
export declare function markNotified(todoId: string): void;
export declare function scheduleNotifiedIdsCleanup(): void;
export declare function checkTaskNotifications(db: any): void;
export declare function startNotificationCheck(db: any): void;
export declare function stopNotificationCheck(): void;
export declare function testNotification(): boolean;

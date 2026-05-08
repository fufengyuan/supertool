export declare function registerRedisStreamHandlers(): void;
export declare function getStreamListeners(): Map<string, {
    id: string;
    key: string;
    interval: any;
}>;
export declare function getNextStreamListenerId(): number;

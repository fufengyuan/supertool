export declare function getActiveLogStreams(): Map<string, Array<{
    stop: () => void;
}>>;
export declare function registerLogHandlers(db: any, requireService: (name: string) => any): void;

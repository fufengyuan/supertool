interface TrayOptions {
    isDev: boolean;
    getAppIcon: () => Electron.NativeImage;
    createMenuTemplate: () => (Electron.MenuItemConstructorOptions | Electron.MenuItem)[];
    appDirname: string;
    createWindow: (opts: Omit<TrayOptions, 'createWindow'>) => void;
    setupWindowLifecycle: () => void;
}
export declare function createTray(opts: TrayOptions): void;
export {};

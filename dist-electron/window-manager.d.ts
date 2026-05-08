import { BrowserWindow, MenuItem, MenuItemConstructorOptions } from 'electron';
interface WindowOptions {
    isDev: boolean;
    getAppIcon: () => Electron.NativeImage;
    createMenuTemplate: () => (MenuItemConstructorOptions | MenuItem)[];
    appDirname: string;
}
export declare function getMainWindow(): BrowserWindow | undefined;
export declare function setMainWindow(window: BrowserWindow | undefined): void;
export declare function createWindow(opts: WindowOptions): void;
export declare function setupWindowLifecycle(): void;
export {};

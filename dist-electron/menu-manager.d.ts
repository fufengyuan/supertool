import { MenuItem, MenuItemConstructorOptions } from 'electron';
export declare function trackMenuUsage(actionId: string): void;
export declare function refreshApplicationMenu(): void;
export declare function getMenuIcon(name: string): Electron.NativeImage | undefined;
export declare function loadAndRegisterShortcuts(db: any): Promise<void>;
export declare function updateShortcuts(shortcuts: Record<string, string>, db: any): Promise<void>;
export declare function buildFavoritesMenu(): MenuItemConstructorOptions;
export declare function createMenuTemplate(): Array<MenuItemConstructorOptions | MenuItem>;

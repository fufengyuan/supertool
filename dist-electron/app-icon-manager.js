"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.getAppIcon = getAppIcon;
const logger_1 = require("./logger");
const electron_1 = require("electron");
const path = __importStar(require("path"));
const fs = __importStar(require("fs"));
function getAppIcon() {
    if (process.platform === 'darwin') {
        const icnsPath = path.join(__dirname, '../build/icon.icns');
        if (fs.existsSync(icnsPath)) {
            const img = electron_1.nativeImage.createFromPath(icnsPath);
            (0, logger_1.info)(`[AppIcon] Loaded icns from ${icnsPath}, isEmpty=${img.isEmpty()}, size=${img.getSize().width}x${img.getSize().height}`);
            return img;
        }
        // 开发模式: 从项目根目录加载
        const devPath = path.join(process.cwd(), 'build', 'icon.icns');
        if (fs.existsSync(devPath)) {
            const img = electron_1.nativeImage.createFromPath(devPath);
            (0, logger_1.info)(`[AppIcon] Loaded icns from ${devPath}, isEmpty=${img.isEmpty()}`);
            return img;
        }
        // 生产模式: 从 resources 目录加载
        const prodPath = path.join(process.resourcesPath, 'build', 'icon.icns');
        if (fs.existsSync(prodPath)) {
            const img = electron_1.nativeImage.createFromPath(prodPath);
            (0, logger_1.info)(`[AppIcon] Loaded icns from ${prodPath}, isEmpty=${img.isEmpty()}`);
            return img;
        }
        console.warn(`[AppIcon] icns not found! __dirname=${__dirname}, cwd=${process.cwd()}, resourcesPath=${process.resourcesPath}`);
    }
    // Linux/Windows: 使用 PNG
    const pngPath = path.join(__dirname, '../build/icons/256x256.png');
    if (fs.existsSync(pngPath))
        return electron_1.nativeImage.createFromPath(pngPath);
    const devPng = path.join(process.cwd(), 'build', 'icons', '256x256.png');
    if (fs.existsSync(devPng))
        return electron_1.nativeImage.createFromPath(devPng);
    return electron_1.nativeImage.createEmpty();
}
//# sourceMappingURL=app-icon-manager.js.map
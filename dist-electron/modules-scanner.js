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
exports.scanProjectModules = scanProjectModules;
const path = __importStar(require("path"));
const fs = __importStar(require("fs"));
/**
 * 扫描项目目录，自动识别 Maven 和 Node.js 模块
 * 支持最多 3 级子模块
 */
function scanProjectModules(projectPath) {
    if (!projectPath || !fs.existsSync(projectPath))
        return [];
    const results = [];
    // 检查根目录是否有 pom.xml
    const rootPomPath = path.join(projectPath, 'pom.xml');
    if (fs.existsSync(rootPomPath)) {
        const rootInfo = extractMavenInfo(rootPomPath);
        const subModules = parseMavenModules(rootPomPath, projectPath, 0, projectPath);
        if (subModules.length > 0) {
            results.push({
                name: rootInfo.artifactId || path.basename(projectPath),
                path: '.',
                type: 'maven',
                hasPomXml: true,
                artifactId: rootInfo.artifactId,
                version: rootInfo.version,
                children: subModules,
            });
        }
        else {
            // Root pom exists but no modules listed — add it as a single module
            results.push({
                name: rootInfo.artifactId || path.basename(projectPath),
                path: '.',
                type: 'maven',
                hasPomXml: true,
                artifactId: rootInfo.artifactId,
                version: rootInfo.version,
            });
        }
    }
    // 检查根目录是否有 package.json（可能是单模块前端项目）
    const rootPackagePath = path.join(projectPath, 'package.json');
    if (fs.existsSync(rootPackagePath) && results.length === 0) {
        try {
            const pkg = JSON.parse(fs.readFileSync(rootPackagePath, 'utf-8'));
            if (pkg.name || pkg.scripts) {
                results.push({
                    name: pkg.name || path.basename(projectPath),
                    path: '.',
                    type: 'npm',
                    hasPackageJson: true,
                    scripts: pkg.scripts ? Object.keys(pkg.scripts) : [],
                });
            }
        }
        catch { }
    }
    // 如果根目录没有 pom.xml 且没有 package.json，扫描一级子目录
    if (results.length === 0) {
        scanSubdirectoriesTree(projectPath, projectPath, 0, results);
    }
    return results;
}
/**
 * 递归扫描子目录并构建树形结构
 */
function scanSubdirectoriesTree(rootPath, currentPath, depth, results) {
    if (depth > 4)
        return; // 最多扫描 5 级（0-4）
    let entries;
    try {
        entries = fs.readdirSync(currentPath);
    }
    catch {
        return;
    }
    for (const entry of entries) {
        if (entry.startsWith('.') || entry === 'node_modules' || entry === 'target' || entry === 'dist') {
            continue;
        }
        const fullPath = path.join(currentPath, entry);
        if (!fs.statSync(fullPath).isDirectory())
            continue;
        const relativePath = path.relative(rootPath, fullPath);
        // Check for pom.xml — if found, parse as Maven module with children
        const pomPath = path.join(fullPath, 'pom.xml');
        if (fs.existsSync(pomPath)) {
            const subModules = parseMavenModules(pomPath, fullPath, depth + 1, rootPath);
            if (subModules.length > 0) {
                const modInfo = extractMavenInfo(pomPath);
                results.push({
                    name: modInfo.artifactId || entry,
                    path: relativePath,
                    type: 'maven',
                    hasPomXml: true,
                    artifactId: modInfo.artifactId,
                    version: modInfo.version,
                    children: subModules,
                });
            }
            else {
                const modInfo = extractMavenInfo(pomPath);
                results.push({
                    name: modInfo.artifactId || entry,
                    path: relativePath,
                    type: 'maven',
                    hasPomXml: true,
                    artifactId: modInfo.artifactId,
                    version: modInfo.version,
                });
            }
            continue;
        }
        // Check for package.json
        const pkgPath = path.join(fullPath, 'package.json');
        if (fs.existsSync(pkgPath)) {
            try {
                const pkg = JSON.parse(fs.readFileSync(pkgPath, 'utf-8'));
                if (pkg.workspaces && (Array.isArray(pkg.workspaces) || pkg.workspaces.packages)) {
                    // Workspace root: build tree with children
                    const workspaceChildren = [];
                    scanSubdirectoriesTree(rootPath, fullPath, depth + 1, workspaceChildren);
                    if (workspaceChildren.length > 0) {
                        results.push({
                            name: pkg.name || entry,
                            path: relativePath,
                            type: 'npm',
                            hasPackageJson: true,
                            scripts: pkg.scripts ? Object.keys(pkg.scripts) : [],
                            children: workspaceChildren,
                        });
                    }
                    else {
                        results.push({
                            name: pkg.name || entry,
                            path: relativePath,
                            type: 'npm',
                            hasPackageJson: true,
                            scripts: pkg.scripts ? Object.keys(pkg.scripts) : [],
                        });
                    }
                }
                else if (pkg.name || pkg.scripts) {
                    results.push({
                        name: pkg.name || entry,
                        path: relativePath,
                        type: 'npm',
                        hasPackageJson: true,
                        scripts: pkg.scripts ? Object.keys(pkg.scripts) : [],
                    });
                }
            }
            catch { }
            continue;
        }
        // No pom.xml or package.json found — check if this directory has any interesting subdirectories
        if (depth < 4) {
            const subChildren = [];
            scanSubdirectoriesTree(rootPath, fullPath, depth + 1, subChildren);
            if (subChildren.length > 0) {
                // This directory contains modules — make it a parent node
                results.push({
                    name: entry,
                    path: relativePath,
                    type: 'unknown',
                    children: subChildren,
                });
            }
        }
    }
}
/**
 * 解析 Maven pom.xml 中的 modules
 */
function parseMavenModules(pomPath, baseDir, depth, rootPath) {
    const results = [];
    const modules = extractMavenModuleNames(pomPath);
    for (const modName of modules) {
        const modDir = path.join(baseDir, modName);
        const modPomPath = path.join(modDir, 'pom.xml');
        const relativePath = path.relative(rootPath, modDir) || '.';
        if (fs.existsSync(modPomPath) && depth < 4) {
            // 递归解析子模块
            const subModules = parseMavenModules(modPomPath, modDir, depth + 1, rootPath);
            if (subModules.length > 0) {
                const modInfo = extractMavenInfo(modPomPath);
                results.push({
                    name: modInfo.artifactId || modName,
                    path: relativePath,
                    type: 'maven',
                    hasPomXml: true,
                    artifactId: modInfo.artifactId,
                    version: modInfo.version,
                    children: subModules,
                });
            }
            else {
                const modInfo = extractMavenInfo(modPomPath);
                results.push({
                    name: modInfo.artifactId || modName,
                    path: relativePath,
                    type: 'maven',
                    hasPomXml: true,
                    artifactId: modInfo.artifactId,
                    version: modInfo.version,
                });
            }
        }
        else {
            results.push({
                name: modName,
                path: relativePath,
                type: 'maven',
                hasPomXml: fs.existsSync(modPomPath),
            });
        }
    }
    return results;
}
/**
 * 从 pom.xml 提取 <modules> 列表
 */
function extractMavenModuleNames(pomPath) {
    try {
        const content = fs.readFileSync(pomPath, 'utf-8');
        const modulesMatch = content.match(/<modules>([\s\S]*?)<\/modules>/);
        if (!modulesMatch)
            return [];
        const moduleNames = [];
        const moduleRegex = /<module>\s*([^<]+?)\s*<\/module>/g;
        let match;
        while ((match = moduleRegex.exec(modulesMatch[1])) !== null) {
            moduleNames.push(match[1].trim());
        }
        return moduleNames;
    }
    catch {
        return [];
    }
}
/**
 * 从 pom.xml 提取 artifactId 和 version
 */
function extractMavenInfo(pomPath) {
    try {
        const content = fs.readFileSync(pomPath, 'utf-8');
        const artifactIdMatch = content.match(/<artifactId>\s*([^<]+?)\s*<\/artifactId>/);
        const versionMatch = content.match(/<version>\s*([^<]+?)\s*<\/version>/);
        return {
            artifactId: artifactIdMatch?.[1]?.trim(),
            version: versionMatch?.[1]?.trim(),
        };
    }
    catch {
        return {};
    }
}
//# sourceMappingURL=modules-scanner.js.map
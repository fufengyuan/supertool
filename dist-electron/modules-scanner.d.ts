export interface ModuleNode {
    name: string;
    path: string;
    type: 'maven' | 'npm' | 'unknown';
    children?: ModuleNode[];
    hasPomXml?: boolean;
    hasPackageJson?: boolean;
    artifactId?: string;
    version?: string;
    scripts?: string[];
}
/**
 * 扫描项目目录，自动识别 Maven 和 Node.js 模块
 * 支持最多 3 级子模块
 */
export declare function scanProjectModules(projectPath: string): ModuleNode[];

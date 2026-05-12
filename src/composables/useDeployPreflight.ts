// @ts-nocheck
import { getTauriAPI } from '../utils/tauri-api'
/**
 * useDeployPreflight — 部署前预检 composable
 *
 * 检查项:
 * 1. SSH 连接可达性
 * 2. 部署路径存在性
 * 3. Maven 可用性
 * 4. 配置完整性
 */
import { useToast } from './useToast';
import type { DeployPreflightResult, DeployPreflightReport } from '../types';

interface CicdConfig {
  buildTool?: string;
  javaHome?: string;
  mavenHome?: string;
  nodeHome?: string;
  npmHome?: string;
  servers?: string;
  deployPath?: string;
  [key: string]: unknown;
}

export function useDeployPreflight() {
  console.log("[useDeployPreflight.ts] useDeployPreflight() init")
  const toast = useToast();

  /**
   * 运行全部预检
   */
  async function runAll(config: CicdConfig): Promise<DeployPreflightReport> {
    const results: DeployPreflightResult[] = [];

    // 1. 配置完整性检查
    results.push(await checkConfigCompleteness(config));

    // 2. SSH 连接检查
    results.push(await checkSshConnection(config));

    // 3. 构建工具可用性（根据构建类型选择性检查）
    const buildTool = (config.buildTool || '').toLowerCase();
    if (buildTool === 'maven') {
      results.push(await checkMavenAvailable(config.mavenHome));
      // Maven 需要 JDK
      results.push(await checkJdkAvailable(config.javaHome));
    } else if (['npm', 'pnpm', 'yarn'].includes(buildTool)) {
      // npm 系列需要 Node.js
      results.push(await checkNodeAvailable(config.nodeHome));
    }

    const passed = results.every((r) => r.passed);

    if (!passed) {
      const failures = results.filter((r) => !r.passed);
      toast.error(
        `部署预检未通过 (${failures.length} 项失败): ${failures.map((f) => f.message).join('; ')}`,
        6000
      );
    } else {
      toast.success('部署预检全部通过 ✅');
    }

    return { passed, results };
  }

  /**
   * 检查配置完整性
   */
  async function checkConfigCompleteness(config: CicdConfig): Promise<DeployPreflightResult> {
    const required = [
      { key: 'deployPath', label: '部署路径' },
    ];

    const missing = required.filter((f) => !config?.[f.key]);
    if (missing.length > 0) {
      return {
        name: '配置完整性',
        passed: false,
        message: `缺少必要配置: ${missing.map((m) => m.label).join(', ')}`,
      };
    }

    // Check servers
    if (!config.servers) {
      return {
        name: '配置完整性',
        passed: false,
        message: '未配置服务器',
      };
    }

    try {
      const parsed = JSON.parse(config.servers as string);
      if (!Array.isArray(parsed) || parsed.length === 0 || !parsed.some((s: { serverId?: string }) => s.serverId)) {
        return {
          name: '配置完整性',
          passed: false,
          message: '未选择有效服务器',
        };
      }
    } catch {
      return {
        name: '配置完整性',
        passed: false,
        message: '服务器配置格式错误',
      };
    }

    return { name: '配置完整性', passed: true, message: '配置完整' };
  }

  /**
   * 检查 SSH 连接
   */
  async function checkSshConnection(config: CicdConfig): Promise<DeployPreflightResult> {
    try {
      // 解析 servers JSON（格式: [{serverId, deployDir}]）
      let serverRefs: Array<{ serverId?: string }> = [];
      if (config.servers && typeof config.servers === 'string') {
        try { serverRefs = JSON.parse(config.servers); } catch {}
      }

      if (!serverRefs.length || !serverRefs.some(s => s.serverId)) {
        return { name: 'SSH 连接', passed: false, message: '未配置有效服务器' };
      }

      const firstRef = serverRefs[0];
      const server = await getTauriAPI().getServerById(firstRef.serverId!);
      if (!server) {
        return { name: 'SSH 连接', passed: false, message: `服务器 ${firstRef.serverId} 不存在` };
      }

      // 传标准 server 字段：host/port/username/password/sshKeyPath/serverId
      const serverConfig = {
        host: server.host,
        port: server.port,
        username: server.username,
        password: server.password || undefined,
        sshKeyPath: server.sshKeyPath || server.privateKey || undefined,
        serverId: server.id,
      };
      const result = await getTauriAPI().testSsh(serverConfig);
      if (result.success) {
        return { name: 'SSH 连接', passed: true, message: `SSH 连接成功 (${server.host}:${server.port})` };
      }
      return { name: 'SSH 连接', passed: false, message: `连接失败: ${result.error}` };
    } catch (error) {
      return { name: 'SSH 连接', passed: false, message: `SSH 检查异常: ${(error as Error).message}` };
    }
  }

  /**
   * 检查 Maven 是否可用
   */
  async function checkMavenAvailable(mavenHome?: string): Promise<DeployPreflightResult> {
    try {
      const result = await getTauriAPI().checkMavenAvailable(mavenHome);
      if (result.available) {
        return { name: 'Maven 可用性', passed: true, message: `Maven 可用 (${result.version || '未知版本'})` };
      }
      return { name: 'Maven 可用性', passed: false, message: result.error || 'Maven 不可用' };
    } catch (error) {
      return { name: 'Maven 可用性', passed: false, message: `Maven 检查异常: ${(error as Error).message}` };
    }
  }

  /**
   * 检查 JDK 是否可用（Maven 构建需要）
   */
  async function checkJdkAvailable(javaHome?: string): Promise<DeployPreflightResult> {
    try {
      const result = await getTauriAPI().checkJavaAvailable(javaHome);
      const label = javaHome ? `JDK 路径 (${javaHome})` : 'JDK 可用性';
      if (result.available) {
        return { name: label, passed: true, message: `Java 可用 (v${result.version || '未知版本'})` };
      }
      return { name: label, passed: false, message: result.error || 'Java 不可用' };
    } catch (error) {
      return { name: 'JDK 可用性', passed: false, message: `Java 检查异常: ${(error as Error).message}` };
    }
  }

  /**
   * 检查 Node.js 是否可用（npm 构建需要）
   */
  async function checkNodeAvailable(nodeHome?: string): Promise<DeployPreflightResult> {
    try {
      const result = await getTauriAPI().checkNodeAvailable(nodeHome);
      const label = nodeHome ? `Node.js 路径 (${nodeHome})` : 'Node.js 可用性';
      if (result.available) {
        return { name: label, passed: true, message: `Node.js 可用 (v${result.version || '未知版本'})` };
      }
      return { name: label, passed: false, message: result.error || 'Node.js 不可用' };
    } catch (error) {
      return { name: 'Node.js 可用性', passed: false, message: `Node 检查异常: ${(error as Error).message}` };
    }
  }

  return { runAll, checkConfigCompleteness, checkSshConnection, checkMavenAvailable, checkJdkAvailable, checkNodeAvailable };
}

/**
 * macOS 局域网权限请求工具
 *
 * macOS 14+ (Sonoma/Sequoia) 引入了 Local Network Privacy (TCC)，
 * 要求 app 在访问局域网前必须获得用户授权。
 *
 * 问题：adhoc 签名（无开发者证书）的 app 不会触发 TCC 弹窗，
 * 而是静默阻止所有局域网出站连接。
 *
 * 解决方案：通过创建本地 TCP 回环服务器并连接自身来触发权限请求，
 * 同时通过 socket 连接测试来检测权限是否被授予。
 */
/**
 * 检查当前进程是否具备局域网访问权限
 *
 * 原理：尝试创建一个本地 TCP 服务器并连接它。
 * 如果 macOS 防火墙/TCC 阻止了网络操作，连接会失败。
 *
 * 注意：这个方法主要检测的是网络栈是否可用，
 * 对于 TCC 阻止的情况，连接仍然可能成功（因为回环地址不受 TCC 限制），
 * 所以还需要配合实际的局域网连接测试。
 */
export declare function checkLocalNetworkPermission(): Promise<{
    granted: boolean;
    details?: string;
}>;
/**
 * 主动请求 macOS 局域网访问权限
 *
 * 这会尝试连接一个本地广播地址来触发 macOS 的局域网权限弹窗。
 * 对于 adhoc 签名的 app，这可能仍然不会弹窗（macOS 14+ 的限制），
 * 但至少可以检测到权限是否被授予。
 *
 * @param timeout 超时时间（毫秒），默认 5000
 * @returns 权限是否被授予
 */
export declare function requestLocalNetworkAccess(timeout?: number): Promise<{
    granted: boolean;
    details?: string;
}>;
/**
 * Get all non-loopback IPv4 addresses (for broadcast targets)
 */
export declare function getLocalIpAddresses(): string[];

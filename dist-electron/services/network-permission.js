"use strict";
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
exports.checkLocalNetworkPermission = checkLocalNetworkPermission;
exports.requestLocalNetworkAccess = requestLocalNetworkAccess;
exports.getLocalIpAddresses = getLocalIpAddresses;
const net = __importStar(require("net"));
const os = __importStar(require("os"));
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
async function checkLocalNetworkPermission() {
    return new Promise((resolve) => {
        const server = net.createServer();
        let resolved = false;
        const cleanup = () => {
            try {
                server.close();
            }
            catch { }
        };
        server.on('error', (err) => {
            if (!resolved) {
                resolved = true;
                cleanup();
                resolve({ granted: false, details: `Local socket test failed: ${err.code} - ${err.message}` });
            }
        });
        server.listen(0, '127.0.0.1', () => {
            const addr = server.address();
            if (!addr || typeof addr === 'string') {
                if (!resolved) {
                    resolved = true;
                    cleanup();
                    resolve({ granted: false, details: 'Failed to get local address' });
                }
                return;
            }
            const port = addr.port;
            const client = new net.Socket();
            client.on('connect', () => {
                if (!resolved) {
                    resolved = true;
                    client.destroy();
                    cleanup();
                    resolve({ granted: true });
                }
            });
            client.on('error', (err) => {
                if (!resolved) {
                    resolved = true;
                    client.destroy();
                    cleanup();
                    resolve({ granted: false, details: `Connection test failed: ${err.message}` });
                }
            });
            client.setTimeout(3000);
            client.on('timeout', () => {
                if (!resolved) {
                    resolved = true;
                    client.destroy();
                    cleanup();
                    resolve({ granted: false, details: 'Connection test timed out' });
                }
            });
            client.connect(port, '127.0.0.1');
        });
    });
}
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
async function requestLocalNetworkAccess(timeout = 5000) {
    // Step 1: Basic local socket test
    const localTest = await checkLocalNetworkPermission();
    if (!localTest.granted) {
        return { granted: false, details: `Local network access blocked: ${localTest.details}` };
    }
    // Step 2: Try connecting to a non-routable address to trigger TCC
    // This is the key test - macOS will check TCC for non-loopback connections
    return new Promise((resolve) => {
        const client = new net.Socket();
        let resolved = false;
        const cleanup = () => {
            try {
                client.destroy();
            }
            catch { }
        };
        // Try connecting to a local IP on a closed port
        // This WILL fail with ECONNREFUSED if allowed, 
        // or EHOSTUNREACH/ETIMEDOUT if blocked by TCC
        const localIp = getFirstNonLoopbackIp();
        if (!localIp) {
            resolve({ granted: true, details: 'No non-loopback interface found, assuming granted' });
            return;
        }
        const testPort = 65534; // Very likely to be closed
        client.on('connect', () => {
            if (!resolved) {
                resolved = true;
                cleanup();
                resolve({ granted: true, details: 'Unexpectedly connected (port may be open)' });
            }
        });
        client.on('error', (err) => {
            if (!resolved) {
                resolved = true;
                cleanup();
                // ECONNREFUSED = network stack works, TCC granted, port just closed ✅
                // EHOSTUNREACH = TCC blocked (macOS lies and says host unreachable) ❌
                // ETIMEDOUT = likely TCC blocked or firewall ❌
                // ENETUNREACH = no route, probably TCC ❌
                if (err.code === 'ECONNREFUSED') {
                    resolve({ granted: true, details: `Network access granted (got ECONNREFUSED on ${localIp}:${testPort})` });
                }
                else if (err.code === 'EHOSTUNREACH') {
                    resolve({ granted: false, details: `Local Network Privacy blocking access to ${localIp}. macOS returns deceptive EHOSTUNREACH. Please check System Settings → Privacy & Security → Local Network, or disable firewall for testing.` });
                }
                else if (err.code === 'ETIMEDOUT' || err.code === 'ENETUNREACH') {
                    resolve({ granted: false, details: `Network access likely blocked (${err.code}). Check macOS Local Network permissions.` });
                }
                else {
                    resolve({ granted: false, details: `Network test failed: ${err.code} - ${err.message}` });
                }
            }
        });
        client.setTimeout(timeout);
        client.on('timeout', () => {
            if (!resolved) {
                resolved = true;
                cleanup();
                resolve({ granted: false, details: `Network test timed out after ${timeout}ms — likely blocked by macOS Local Network Privacy` });
            }
        });
        client.connect(testPort, localIp);
    });
}
/**
 * Get the first non-loopback IPv4 address
 */
function getFirstNonLoopbackIp() {
    const interfaces = os.networkInterfaces();
    for (const ifaces of Object.values(interfaces)) {
        if (!ifaces)
            continue;
        for (const iface of ifaces) {
            if (iface.family === 'IPv4' && !iface.internal) {
                return iface.address;
            }
        }
    }
    return null;
}
/**
 * Get all non-loopback IPv4 addresses (for broadcast targets)
 */
function getLocalIpAddresses() {
    const addresses = [];
    const interfaces = os.networkInterfaces();
    for (const ifaces of Object.values(interfaces)) {
        if (!ifaces)
            continue;
        for (const iface of ifaces) {
            if (iface.family === 'IPv4' && !iface.internal) {
                addresses.push(iface.address);
            }
        }
    }
    return addresses;
}
//# sourceMappingURL=network-permission.js.map
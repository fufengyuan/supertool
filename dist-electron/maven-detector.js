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
exports.detectMavenHome = detectMavenHome;
const async_exec_1 = require("./async-exec");
const path = __importStar(require("path"));
const fs = __importStar(require("fs"));
async function detectMavenHome() {
    // 1. Environment variable
    if (process.env.MAVEN_HOME && fs.existsSync(process.env.MAVEN_HOME))
        return process.env.MAVEN_HOME;
    // 2. Homebrew opt symlinks (preferred — always point to current version)
    const homebrewOptPaths = [
        '/opt/homebrew/opt/maven',
        '/usr/local/opt/maven',
    ];
    for (const p of homebrewOptPaths) {
        if (fs.existsSync(p + '/bin/mvn'))
            return p;
    }
    // 3. Versioned Cellar directories — find latest version
    const cellarPaths = ['/opt/homebrew/Cellar/maven', '/usr/local/Cellar/maven'];
    for (const base of cellarPaths) {
        if (fs.existsSync(base)) {
            try {
                const versions = fs.readdirSync(base).filter(d => /^\d/.test(d)).sort().reverse();
                if (versions.length > 0) {
                    const mavenHome = path.join(base, versions[0]);
                    if (fs.existsSync(mavenHome + '/bin/mvn'))
                        return mavenHome;
                }
            }
            catch { }
        }
    }
    // 4. Linux: common package manager and manual install paths
    const fallbackPaths = [
        '/usr/share/maven', // apt/dnf default (Ubuntu, Fedora, CentOS)
        '/usr/lib/maven', // some older distros
        '/usr/local/maven', // manual tar.gz extraction
        '/opt/maven', // manual installation
        '/snap/maven/current', // Ubuntu Snap
    ];
    for (const p of fallbackPaths) {
        if (fs.existsSync(p + '/bin/mvn'))
            return p;
    }
    // 5. Use `which mvn` and resolve symlink to find Maven home
    try {
        const mvnPathResult = await (0, async_exec_1.tryCommand)('which mvn', { timeout: 2000 });
        const mvnPath = mvnPathResult?.stdout.trim();
        if (mvnPath && fs.existsSync(mvnPath)) {
            const realPath = fs.realpathSync(mvnPath);
            // /opt/homebrew/Cellar/maven/X.Y.Z/bin/mvn → /opt/homebrew/Cellar/maven/X.Y.Z
            const binDir = path.dirname(realPath);
            if (path.basename(binDir) === 'bin') {
                const mavenHome = path.dirname(binDir);
                if (fs.existsSync(mavenHome))
                    return mavenHome;
            }
            // Fallback: just use the directory containing mvn
            return binDir;
        }
    }
    catch { }
    return undefined;
}
//# sourceMappingURL=maven-detector.js.map
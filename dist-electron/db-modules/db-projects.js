"use strict";
const db_core_1 = require("./db-core");
const child_process_1 = require("child_process");
function rowToProject(row) {
    return {
        id: row.id,
        name: row.name,
        description: row.description,
        color: row.color,
        repoPath: row.repoPath,
        branch: row.branch,
        repoPath2: row.repoPath2,
        branch2: row.branch2,
        gitUrl1: row.gitUrl1,
        gitUrl2: row.gitUrl2,
        category: row.category,
        createdAt: row.createdAt,
        updatedAt: row.updatedAt,
        archived: row.archived === 1
    };
}
function getAllProjects(onlyActive = true) {
    let query = 'SELECT * FROM projects';
    if (onlyActive) {
        query += ' WHERE archived = 0';
    }
    query += ' ORDER BY createdAt DESC';
    const stmt = (0, db_core_1.getDatabase)().prepare(query);
    const rows = stmt.all();
    return rows.map(rowToProject);
}
function addProject(project) {
    const stmt = (0, db_core_1.getDatabase)().prepare(`
    INSERT INTO projects (id, name, description, color, repoPath, branch, repoPath2, branch2, gitUrl1, gitUrl2, category, createdAt, updatedAt, archived)
    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
  `);
    stmt.run(project.id, project.name, project.description || '', project.color || '#6366f1', project.repoPath || null, project.branch || null, project.repoPath2 || null, project.branch2 || null, project.gitUrl1 || null, project.gitUrl2 || null, project.category || null, project.createdAt, project.updatedAt, project.archived ? 1 : 0);
    return project;
}
function updateProject(project) {
    const stmt = (0, db_core_1.getDatabase)().prepare(`
    UPDATE projects
    SET name = ?, description = ?, color = ?, repoPath = ?, branch = ?, repoPath2 = ?, branch2 = ?, gitUrl1 = ?, gitUrl2 = ?, category = ?, updatedAt = ?, archived = ?
    WHERE id = ?
  `);
    stmt.run(project.name, project.description || '', project.color || '#6366f1', project.repoPath || null, project.branch || null, project.repoPath2 || null, project.branch2 || null, project.gitUrl1 || null, project.gitUrl2 || null, project.category || null, project.updatedAt, project.archived ? 1 : 0, project.id);
    return project;
}
function deleteProject(id) {
    const stmt = (0, db_core_1.getDatabase)().prepare('DELETE FROM projects WHERE id = ?');
    stmt.run(id);
    return id;
}
function getProjectStats(projectId) {
    const db = (0, db_core_1.getDatabase)();
    const statsStmt = db.prepare(`
    SELECT
      COUNT(*) as total,
      SUM(CASE WHEN completed = 1 THEN 1 ELSE 0 END) as completed
    FROM todos WHERE projectId = ?
  `);
    const stats = statsStmt.get(projectId);
    return {
        total: stats ? stats.total || 0 : 0,
        completed: stats ? stats.completed || 0 : 0,
        progress: stats && stats.total > 0 ? Math.round((stats.completed / stats.total) * 100) : 0
    };
}
function getTodosByProject(projectId) {
    const stmt = (0, db_core_1.getDatabase)().prepare('SELECT * FROM todos WHERE projectId = ? ORDER BY orderNum ASC, createdAt DESC');
    const rows = stmt.all(projectId);
    return rows.map(db_core_1.rowToTodo);
}
// 获取Git提交记录
async function getGitCommits(repoPath, branch = null, sinceDate = null) {
    return new Promise((resolve) => {
        if (!repoPath || !require('fs').existsSync(repoPath)) {
            resolve([]);
            return;
        }
        // SECURITY: Use execFile with argument array to prevent command injection
        const args = ['log', '--pretty=format:%h|%ad|%an|%s', '--date=iso'];
        if (branch)
            args.splice(1, 0, branch);
        if (sinceDate)
            args.push(`--since=${sinceDate}`);
        (0, child_process_1.execFile)('git', args, { cwd: repoPath, maxBuffer: 1024 * 1024 * 10 }, (logError, stdout) => {
            if (logError) {
                console.error('Git log failed:', logError);
                resolve([]);
                return;
            }
            const commits = [];
            const lines = stdout.trim().split('\n').filter(line => line.trim() !== '');
            lines.forEach(line => {
                const parts = line.split('|');
                if (parts.length >= 4) {
                    commits.push({
                        hash: parts[0],
                        date: parts[1],
                        author: parts[2],
                        message: parts.slice(3).join('|')
                    });
                }
            });
            resolve(commits);
        });
    });
}
// 扫描本地 git 仓库
async function scanLocalGitRepos() {
    const os = require('os');
    const path = require('path');
    const fs = require('fs');
    const homeDir = os.homedir();
    const codeDirs = [
        // JetBrains IDEs 默认工作目录
        path.join(homeDir, 'WebstormProjects'),
        path.join(homeDir, 'IdeaProjects'),
        path.join(homeDir, 'PycharmProjects'),
        path.join(homeDir, 'GoLandProjects'),
        path.join(homeDir, 'PhpStormProjects'),
        path.join(homeDir, 'CLionProjects'),
        path.join(homeDir, 'RubyMineProjects'),
        path.join(homeDir, 'RiderProjects'),
        path.join(homeDir, 'AndroidStudioProjects'),
        path.join(homeDir, 'DataGripProjects'),
        path.join(homeDir, 'StudioProjects'),
        // VSCode 及常见路径
        path.join(homeDir, 'Code'),
        path.join(homeDir, 'VSCodeProjects'),
        path.join(homeDir, 'vscode'),
        path.join(homeDir, 'projects'),
        path.join(homeDir, 'work'),
        path.join(homeDir, 'code'),
        path.join(homeDir, 'dev'),
        path.join(homeDir, 'src'),
        path.join(homeDir, 'workspace'),
        path.join(homeDir, 'git'),
        path.join(homeDir, 'repos'),
        // 大小写变体
        path.join(homeDir, 'Projects'),
        path.join(homeDir, 'Work'),
        path.join(homeDir, 'Workspace'),
        // 文档/桌面
        path.join(homeDir, 'Documents', 'code'),
        path.join(homeDir, 'Documents', 'GitHub'),
        path.join(homeDir, 'Desktop'),
        // 系统级
        path.join('/', 'data'),
        path.join('/', 'home', require('os').userInfo().username, 'projects'),
        path.join('/', 'home', require('os').userInfo().username, 'workspace'),
        path.join('/', 'home', require('os').userInfo().username, 'code'),
    ].filter((dir) => {
        try {
            return fs.existsSync(dir) && fs.statSync(dir).isDirectory();
        }
        catch {
            return false;
        }
    });
    const repos = [];
    const scanDir = (dir, depth) => {
        if (depth > 2)
            return;
        try {
            const entries = fs.readdirSync(dir);
            if (entries.includes('.git') && fs.existsSync(path.join(dir, '.git', 'config'))) {
                const gitConfig = fs.readFileSync(path.join(dir, '.git', 'config'), 'utf-8');
                let url = '';
                const urlMatch = gitConfig.match(/url\s*=\s*(.+)/);
                if (urlMatch)
                    url = urlMatch[1].trim();
                repos.push({
                    path: dir,
                    name: path.basename(dir),
                    relativePath: path.relative(homeDir, dir),
                    url,
                });
            }
            else {
                for (const entry of entries) {
                    const fullPath = path.join(dir, entry);
                    try {
                        if (fs.statSync(fullPath).isDirectory() && !entry.startsWith('.') && entry !== 'node_modules' && entry !== 'dist' && entry !== 'build') {
                            scanDir(fullPath, depth + 1);
                        }
                    }
                    catch { }
                }
            }
        }
        catch { }
    };
    for (const baseDir of codeDirs) {
        try {
            scanDir(baseDir, 0);
        }
        catch { }
    }
    return repos;
}
// 验证指定路径是否为有效的 Git 仓库
async function validateGitRepoPath(repoPath) {
    const path = require('path');
    const fs = require('fs');
    if (!repoPath || !repoPath.trim()) {
        return { valid: false, error: '路径不能为空' };
    }
    const fullPath = path.resolve(repoPath.trim());
    if (!fs.existsSync(fullPath)) {
        return { valid: false, error: '路径不存在' };
    }
    const stat = fs.statSync(fullPath);
    if (!stat.isDirectory()) {
        return { valid: false, error: '路径不是目录' };
    }
    const gitDir = path.join(fullPath, '.git');
    if (!fs.existsSync(gitDir)) {
        return { valid: false, error: '该目录不是 Git 仓库（缺少 .git 目录）' };
    }
    const gitConfig = path.join(gitDir, 'config');
    if (!fs.existsSync(gitConfig)) {
        return { valid: false, error: 'Git 仓库配置无效' };
    }
    return { valid: true, name: path.basename(fullPath) };
}
// 获取指定仓库的分支列表
async function getGitBranches(repoPath) {
    return new Promise((resolve) => {
        if (!repoPath || !require('fs').existsSync(repoPath)) {
            resolve([]);
            return;
        }
        // SECURITY: Use execFile to prevent command injection
        (0, child_process_1.execFile)('git', ['branch', '-r', '--format=%(refname:short)'], { cwd: repoPath, maxBuffer: 1024 * 1024 }, (error, stdout) => {
            if (error) {
                resolve([]);
                return;
            }
            const branches = stdout.trim().split('\n').filter((b) => b.trim() && !b.includes('HEAD'));
            resolve(branches);
        });
    });
}
module.exports = {
    rowToProject,
    getAllProjects,
    addProject,
    updateProject,
    deleteProject,
    getProjectStats,
    getTodosByProject,
};
//# sourceMappingURL=db-projects.js.map
#!/usr/bin/env node
/**
 * version-sync.js — 统一更新项目版本号
 * 
 * 用法:
 *   node scripts/version-sync.js              # 显示当前版本
 *   node scripts/version-sync.js 4.2.0        # 更新为 4.2.0
 *   node scripts/version-sync.js minor        # 自动递增次版本号 (4.1.0 → 4.2.0)
 *   node scripts/version-sync.js patch        # 自动递增修订号 (4.1.0 → 4.1.1)
 *   node scripts/version-sync.js major        # 自动递增主版本号 (4.1.0 → 5.0.0)
 *
 * 同步的文件:
 *   - package.json (version)
 *   - tauri/Cargo.toml (version)
 *   - tauri/tauri.conf.json (version)
 */

const fs = require('fs');
const path = require('path');

const ROOT = path.resolve(__dirname, '..');
const FILES = [
  { path: path.join(ROOT, 'package.json'), pattern: /"version":\s*"[^"]+"/, format: (v) => `"version": "${v}"` },
  { path: path.join(ROOT, 'tauri', 'Cargo.toml'), pattern: /^version\s*=\s*"[^"]+"/m, format: (v) => `version = "${v}"` },
  { path: path.join(ROOT, 'tauri', 'tauri.conf.json'), pattern: /"version":\s*"[^"]+"/, format: (v) => `"version": "${v}"` },
];

function getCurrentVersion() {
  const pkg = JSON.parse(fs.readFileSync(FILES[0].path, 'utf8'));
  return pkg.version;
}

function bumpVersion(version, type) {
  const [major, minor, patch] = version.split('.').map(Number);
  switch (type) {
    case 'major': return `${major + 1}.0.0`;
    case 'minor': return `${major}.${minor + 1}.0`;
    case 'patch': return `${major}.${minor}.${patch + 1}`;
    default: return version;
  }
}

function validateVersion(v) {
  return /^\d+\.\d+\.\d+$/.test(v);
}

// ---- Main ----
const args = process.argv.slice(2);
const currentVersion = getCurrentVersion();

if (args.length === 0) {
  console.log(`当前版本: ${currentVersion}`);
  console.log(`用法: node scripts/version-sync.js <version|major|minor|patch>`);
  process.exit(0);
}

let newVersion = args[0];
if (['major', 'minor', 'patch'].includes(newVersion)) {
  newVersion = bumpVersion(currentVersion, newVersion);
}

if (!validateVersion(newVersion)) {
  console.error(`错误: 无效版本号 "${newVersion}"，格式应为 x.y.z`);
  process.exit(1);
}

if (newVersion === currentVersion) {
  console.log(`版本未变更: ${currentVersion}`);
  process.exit(0);
}

console.log(`更新版本: ${currentVersion} → ${newVersion}`);

for (const file of FILES) {
  let content = fs.readFileSync(file.path, 'utf8');
  const oldMatch = content.match(file.pattern);
  if (!oldMatch) {
    console.warn(`⚠ 未找到版本字段: ${file.path}`);
    continue;
  }
  const newContent = content.replace(file.pattern, file.format(newVersion));
  fs.writeFileSync(file.path, newContent, 'utf8');
  console.log(`✓ ${path.relative(ROOT, file.path)}`);
}

console.log(`\n版本已更新为 ${newVersion}`);

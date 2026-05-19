#!/bin/bash
# 初始化 Git hooks - 克隆仓库后运行一次
# 用法: ./scripts/init-hooks.sh

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_DIR"

# 设置 hooks 路径
git config core.hooksPath scripts/hooks

echo "✅ Git hooks 已配置: scripts/hooks"
echo ""
echo "自动版本号规则:"
echo "  feat:     → minor (+0.1.0)"
echo "  fix:      → patch (+0.0.1)"
echo "  feat!:    → major (+1.0.0)"
echo "  chore:    → 不更新版本号"
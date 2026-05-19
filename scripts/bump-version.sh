#!/bin/bash
# 自动版本号更新脚本 - 统一更新所有4处版本号
# 用法: ./scripts/bump-version.sh [level]
# level: patch (默认, +0.0.1), minor (+0.1.0), major (+1.0.0)
#
# 更新位置:
#   - package.json
#   - cli/Cargo.toml
#   - core/Cargo.toml
#   - tauri/Cargo.toml

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

LEVEL="${1:-patch}"

if [[ ! "$LEVEL" =~ ^(patch|minor|major)$ ]]; then
    echo "❌ 无效级别: $LEVEL (可用: patch, minor, major)"
    exit 1
fi

# 所有版本号文件
VERSION_FILES=(
    "$PROJECT_DIR/package.json"
    "$PROJECT_DIR/cli/Cargo.toml"
    "$PROJECT_DIR/core/Cargo.toml"
    "$PROJECT_DIR/tauri/Cargo.toml"
)

# 从 package.json 读取当前版本
CURRENT_VERSION=$(grep '"version"' "$PROJECT_DIR/package.json" | sed -E 's/.*"version": *"([^"]+)".*/\1/')

if [[ ! "$CURRENT_VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "❌ 无效版本格式: $CURRENT_VERSION"
    exit 1
fi

# 解析版本号
MAJOR=$(echo "$CURRENT_VERSION" | cut -d. -f1)
MINOR=$(echo "$CURRENT_VERSION" | cut -d. -f2)
PATCH=$(echo "$CURRENT_VERSION" | cut -d. -f3)

# 根据级别更新
case "$LEVEL" in
    major)
        MAJOR=$((MAJOR + 1))
        MINOR=0
        PATCH=0
        ;;
    minor)
        MINOR=$((MINOR + 1))
        PATCH=0
        ;;
    patch)
        PATCH=$((PATCH + 1))
        ;;
esac

NEW_VERSION="$MAJOR.$MINOR.$PATCH"

echo "📦 版本号更新: $CURRENT_VERSION → $NEW_VERSION ($LEVEL)"
echo ""

# 更新所有文件
cd "$PROJECT_DIR"

for file in "${VERSION_FILES[@]}"; do
    if [[ "$file" == *"package.json" ]]; then
        # JSON 格式
        sed -i -E 's/"version": *"[^"]+"/"version": "'"$NEW_VERSION"'"/' "$file"
    else
        # TOML 格式
        sed -i -E 's/^version = "[^"]+"/version = "'"$NEW_VERSION"'"/' "$file"
    fi
    git add "$file"
    fname=$(basename "$file")
    echo "  ✅ $fname"
done

echo ""
echo "📋 所有版本号文件已添加到 git 暂存区"
echo ""
echo "建议提交命令:"
echo "  git commit -m \"chore: bump version to $NEW_VERSION\""
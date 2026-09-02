#!/bin/bash
# 自动版本号更新脚本 - 统一更新所有6处版本号
# 用法: ./scripts/bump-version.sh [level] [--no-add]
# level: patch (默认, +0.0.1), minor (+0.1.0), major (+1.0.0)
# --no-add: 不自动 git add（钩子调用时使用）
#
# 更新位置:
#   - package.json
#   - Cargo.toml（根 workspace.package.version）
#   - cli/Cargo.toml
#   - core/Cargo.toml
#   - tauri/Cargo.toml
#   - tauri/tauri.conf.json
#   - Cargo.lock（通过 cargo generate-lockfile 同步）

set -e

# ─── 确保 cargo 在 PATH 中（git hooks 环境 PATH 不一定包含）───
if ! command -v cargo &>/dev/null; then
    if [[ -f "$HOME/.cargo/env" ]]; then
        source "$HOME/.cargo/env"
    elif [[ -x "$HOME/.cargo/bin/cargo" ]]; then
        export PATH="$HOME/.cargo/bin:$PATH"
    elif [[ -n "$CARGO_HOME" && -x "$CARGO_HOME/bin/cargo" ]]; then
        export PATH="$CARGO_HOME/bin:$PATH"
    fi
fi

CARGO_AVAILABLE=false
if command -v cargo &>/dev/null; then
    CARGO_AVAILABLE=true
fi

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
# 用 git 获取 repo 根路径（大小写匹配 git 内部表示，避免 macOS 大小写不敏感路径导致 git add 报错）
PROJECT_DIR=$(cd "$SCRIPT_DIR" && git rev-parse --show-toplevel 2>/dev/null || echo "$(dirname "$SCRIPT_DIR")")

LEVEL="${1:-patch}"
NO_ADD=false

if [[ "$LEVEL" == "--no-add" ]]; then
    LEVEL="patch"
    NO_ADD=true
fi

if [[ "$2" == "--no-add" ]]; then
    NO_ADD=true
fi

if [[ ! "$LEVEL" =~ ^(patch|minor|major)$ ]]; then
    echo "❌ 无效级别: $LEVEL (可用: patch, minor, major)"
    exit 1
fi

# 所有版本号文件
VERSION_FILES=(
    "$PROJECT_DIR/package.json"
    "$PROJECT_DIR/Cargo.toml"
    "$PROJECT_DIR/cli/Cargo.toml"
    "$PROJECT_DIR/core/Cargo.toml"
    "$PROJECT_DIR/tauri/Cargo.toml"
    "$PROJECT_DIR/tauri/tauri.conf.json"
)

CARGO_LOCK="$PROJECT_DIR/Cargo.lock"

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

# 版本替换统一用 perl（macOS/Linux 均内置）。
# 之前用 macOS 自带 BSD sed 的 `-i ''` 在 zsh 下会把正则误当文件报
# "No such file or directory"，且 `+` 依赖 -E 扩展正则才生效，坑很多。
# perl 的 -i 原地替换 + -pe 正则里的 + 直接用，跨平台一致、无坑。
cd "$PROJECT_DIR"

for file in "${VERSION_FILES[@]}"; do
    # JSON 文件用 "version": "x.y.z" 格式
    # Cargo.toml 用 version = "x.y.z" 格式
    if [[ "$file" == *"package.json"* ]] || [[ "$file" == *"tauri.conf.json"* ]]; then
        perl -i -pe 's/"version": *"[^"]+"/"version": "'"$NEW_VERSION"'"/' "$file"
    else
        perl -i -pe 's/^version = "[^"]+"/version = "'"$NEW_VERSION"'"/' "$file"
    fi

    if [[ "$NO_ADD" == "false" ]]; then
        git add "$file"
    fi

    fname=$(basename "$file")
    echo "  ✅ $fname"
done

# ─── 更新 Cargo.lock ───
if [[ "$CARGO_AVAILABLE" == "true" ]]; then
    echo "  🔄 同步 Cargo.lock..."
    cd "$PROJECT_DIR"
    cargo generate-lockfile > /dev/null 2>&1
    if [[ "$NO_ADD" == "false" ]]; then
        git add "$CARGO_LOCK"
    fi
    echo "  ✅ Cargo.lock"
else
    echo "  ⚠️ 未找到 cargo，跳过 Cargo.lock 更新"
fi

if [[ "$NO_ADD" == "false" ]]; then
    echo ""
    echo "📋 所有版本号已添加到 git 暂存区"
    echo ""
    echo "建议提交信息:"
    echo "  git commit -m \"chore: bump version to $NEW_VERSION\""
fi

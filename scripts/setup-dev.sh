#!/bin/bash
# SuperTool 开发环境初始化脚本
set -e

echo "🔧 SuperTool Development Setup"
echo ""

# ── sccache ──
if command -v sccache &>/dev/null; then
    echo "✅ sccache 已安装 ($(sccache --version | head -1))"
else
    echo "📦 正在安装 sccache（Rust 编译缓存，显著加速重复构建）..."
    if [[ "$(uname)" == "Darwin" ]]; then
        brew install sccache
    elif command -v apt-get &>/dev/null; then
        apt-get install -y sccache 2>/dev/null || cargo install sccache
    else
        cargo install sccache
    fi
    echo "✅ sccache 安装完成"
fi

# ── 验证 Cargo 配置 ──
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
CONFIG="$PROJECT_DIR/.cargo/config.toml"
if [ -f "$CONFIG" ]; then
    echo "✅ Cargo 配置已就绪 (.cargo/config.toml)"
else
    echo "⚠️  .cargo/config.toml 不存在，请手动创建"
fi

echo ""
echo "🚀 开发准备完成！"
echo "   - cargo check --workspace   # 编译检查(dev)"
echo "   - cargo build --release     # 发布构建"

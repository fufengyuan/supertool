#!/usr/bin/env bash
# GPUI 开发热重启 — 保存文件后自动 rebuild + restart
# 依赖: cargo-watch (cargo install cargo-watch)
set -e
cd "$(dirname "$0")/.."
echo "🚀 启动 GPUI 开发模式（debug）..."
echo "   保存 .rs 文件后自动重建 + 重启"
echo "   按 Ctrl+C 停止"
cargo watch -q -c -x '+nightly run -p supertool-gpui'

#!/bin/bash
set -e

MODE="${1:-pre-build}"
ARCH="${2:-native}"

build_cli() {
    local arch="$1"
    echo "🔨 Building CLI (stool) for: $arch"
    cd cli

    case "$arch" in
      arm64|aarch64)
        cargo build --release --target aarch64-apple-darwin
        mkdir -p ../target/release
        cp target/aarch64-apple-darwin/release/stool ../target/release/stool
        ;;
      x64|x86_64)
        cargo build --release --target x86_64-apple-darwin
        mkdir -p ../target/release
        cp target/x86_64-apple-darwin/release/stool ../target/release/stool
        ;;
      native)
        cargo build --release
        # Copy arch-specific to generic location
        for dir in target/*/release; do
            if [ -f "$dir/stool" ]; then
                mkdir -p ../target/release
                cp "$dir/stool" ../target/release/stool
                break
            fi
        done
        ;;
      all)
        cargo build --release --target aarch64-apple-darwin
        cargo build --release --target x86_64-apple-darwin
        mkdir -p ../target/release
        lipo -create -output ../target/release/stool \
          target/aarch64-apple-darwin/release/stool \
          target/x86_64-apple-darwin/release/stool
        ;;
    esac

    cd ..
    echo "✅ CLI built → cli/target/release/stool"
}

case "$MODE" in
  pre-build)
    build_cli "$ARCH"
    ;;
  full)
    build_cli "$ARCH"
    echo "📦 Building Tauri app..."
    pnpm tauri build
    echo "✅ Done!"
    ;;
  *)
    echo "Usage: $0 {pre-build|full} [native|arm64|x64|all]"
    exit 1
    ;;
esac

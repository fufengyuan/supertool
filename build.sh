#!/bin/bash
set -e

MODE="${1:-pre-build}"
ARCH="${2:-native}"

# ─── 版本 ───
VERSION="4.1.0"
PKG_OUTPUT="target/release"

# ═══════════════════════════════════════════
# CLI 编译（跨平台）
# ═══════════════════════════════════════════
build_cli() {
    local arch="$1"
    local target
    local os
    os="$(uname)"

    echo "🔨 Building CLI (stool)..."
    cd cli

    case "$arch" in
      arm64|aarch64)
        case "$os" in
          Darwin)  target="aarch64-apple-darwin" ;;
          Linux)   target="aarch64-unknown-linux-gnu" ;;
          *)       target="" ;;
        esac
        ;;
      x64|x86_64)
        case "$os" in
          Darwin)  target="x86_64-apple-darwin" ;;
          Linux)   target="x86_64-unknown-linux-gnu" ;;
          MINGW*|MSYS*|CYGWIN*|Windows_NT) target="x86_64-pc-windows-msvc" ;;
        esac
        ;;
      native)
        target=""
        ;;
      all)
        # macOS universal
        if [[ "$os" == "Darwin" ]]; then
            cargo build --release --target aarch64-apple-darwin
            cargo build --release --target x86_64-apple-darwin
            mkdir -p ../target/release
            lipo -create -output ../target/release/stool \
              target/aarch64-apple-darwin/release/stool \
              target/x86_64-apple-darwin/release/stool
            cd ..
            echo "✅ CLI universal built → target/release/stool"
            return
        fi
        target=""
        ;;
    esac

    if [ -n "$target" ]; then
        cargo build --release --target "$target"
    else
        cargo build --release
    fi

    # Copy to workspace target/release
    mkdir -p ../target/release
    for dir in target/release target/*/release; do
        if [ -f "$dir/stool" ] || [ -f "$dir/stool.exe" ]; then
            cp "$dir/stool"* ../target/release/ 2>/dev/null || true
            break
        fi
    done

    cd ..
    echo "✅ CLI built → target/release/"
}

# ─── 通用：Tauri 构建 ───
tauri_build() {
    echo "🔨 Building Tauri app..."
    local os
    os="$(uname)"
    case "$os" in
        Darwin)
            # macOS: dmg + app (pkg 由后续步骤处理)
            pnpm tauri build --bundles dmg,app || pnpm tauri build
            ;;
        Linux)
            # Linux: deb + rpm
            pnpm tauri build --bundles deb,rpm || pnpm tauri build
            ;;
        *)
            pnpm tauri build
            ;;
    esac
    echo "✅ Tauri build complete"
}

# ═══════════════════════════════════════════
# macOS: 生成 .dmg (原生) + .pkg (增强)
# ═══════════════════════════════════════════
build_macos_all() {
    local arch="$1"
    build_cli "$arch"
    tauri_build

    mkdir -p "$PKG_OUTPUT"

    # 清理上次打包残留的临时 DMG 文件（rw.*.dmg 是 create-dmg 的临时文件）
    rm -f target/release/bundle/macos/rw.*.dmg

    # ── 1. 复制原生 .dmg ──
    local DMG_SRC=""
    for d in target/release/bundle/dmg/*.dmg target/release/bundle/macos/*.dmg; do
        if [ -f "$d" ]; then DMG_SRC="$d"; break; fi
    done
    if [ -n "$DMG_SRC" ]; then
        cp -f "$DMG_SRC" "$PKG_OUTPUT/"
        echo "✅ dmg → $PKG_OUTPUT/$(basename "$DMG_SRC")"
    fi

    # ── 2. 生成增强 .pkg（CLI + Skills 通过 postinstall 从 app bundle 分发）──
    local APP_PATH=""
    for d in target/release/bundle/macos/*.app target/release/bundle/osx/*.app; do
        if [ -d "$d" ]; then APP_PATH="$d"; break; fi
    done
    if [ -z "$APP_PATH" ]; then
        echo "❌ .app bundle not found"; return
    fi
    echo "📱 Found app: $APP_PATH"

    local PKG_DIR="pkg-build"
    rm -rf "$PKG_DIR"
    mkdir -p "$PKG_DIR/scripts"

    # postinstall：从 /Applications/SuperTool.app 内部取出 stool CLI + skills
    cat > "$PKG_DIR/scripts/postinstall" << 'POSTINSTALL'
#!/bin/bash
set -e
INSTALL_DIR="$2"
APP_DIR="${INSTALL_DIR}/SuperTool.app"
echo "🔧 SuperTool postinstall..."

CLI_SRC="${APP_DIR}/Contents/Resources/_up_/target/release/stool"
if [ -f "$CLI_SRC" ]; then
    mkdir -p /usr/local/bin
    cp -f "$CLI_SRC" /usr/local/bin/stool
    chmod 755 /usr/local/bin/stool
    echo "✅ stool → /usr/local/bin/stool"
fi

SKILLS_SRC="${APP_DIR}/Contents/Resources/_up_/skills"
if [ -d "$SKILLS_SRC" ]; then
    mkdir -p /usr/local/share/supertool/skills
    cp -R "$SKILLS_SRC"/* /usr/local/share/supertool/skills/ 2>/dev/null || true
    echo "✅ skills → /usr/local/share/supertool/skills"
fi

LOGGED_IN_USER=$(stat -f '%Su' /dev/console 2>/dev/null || echo "")
if [ -n "$LOGGED_IN_USER" ] && [ "$LOGGED_IN_USER" != "root" ]; then
    USER_HOME=$(dscl . -read "/Users/${LOGGED_IN_USER}" NFSHomeDirectory 2>/dev/null | cut -d' ' -f2 || echo "/Users/${LOGGED_IN_USER}")
    if [ -n "$USER_HOME" ] && [ -d "$USER_HOME" ]; then
        DATA_DIR="${USER_HOME}/.supertool"
        mkdir -p "${DATA_DIR}"
        chown "${LOGGED_IN_USER}" "${DATA_DIR}"
        USER_SKILLS="${DATA_DIR}/skills"
        if [ ! -e "$USER_SKILLS" ]; then
            ln -sf /usr/local/share/supertool/skills "$USER_SKILLS"
            chown -h "${LOGGED_IN_USER}" "$USER_SKILLS"
            echo "✅ Symlink: ${USER_SKILLS} -> /usr/local/share/supertool/skills"
        fi
    fi
fi
echo "✅ SuperTool installation complete!"
exit 0
POSTINSTALL
    chmod 755 "$PKG_DIR/scripts/postinstall"

    # 使用 pkgbuild --component 标准方式打包 .app（比 --root 更可靠）
    # 自动安装到 /Applications，无 bundle relocate 副作用
    pkgbuild --component "$APP_PATH" \
        --identifier "com.fufengyuan.supertool" \
        --version "$VERSION" \
        --install-location "/Applications" \
        --scripts "$PKG_DIR/scripts" \
        "$PKG_OUTPUT/SuperTool-${VERSION}.pkg"

    rm -rf "$PKG_DIR"
    echo "✅ pkg → $PKG_OUTPUT/SuperTool-${VERSION}.pkg"
}

# ═══════════════════════════════════════════
# Linux: 生成 .AppImage (原生) + .deb (增强)
# ═══════════════════════════════════════════
build_linux_all() {
    local arch="$1"
    build_cli "$arch"
    tauri_build

    mkdir -p "$PKG_OUTPUT"

    # ── 生成增强 .deb ──
    local TAURI_DEB=""
    for d in target/release/bundle/deb/*.deb target/release/bundle/app/*.deb; do
        if [ -f "$d" ]; then TAURI_DEB="$d"; break; fi
    done

    if [ -n "$TAURI_DEB" ]; then
        local DEB_DIR="deb-build"
        rm -rf "$DEB_DIR"
        mkdir -p "$DEB_DIR"
        dpkg-deb -R "$TAURI_DEB" "$DEB_DIR/root"

        mkdir -p "$DEB_DIR/root/usr/local/bin"
        cp -f target/release/stool "$DEB_DIR/root/usr/local/bin/"
        chmod 755 "$DEB_DIR/root/usr/local/bin/stool"

        mkdir -p "$DEB_DIR/root/usr/local/share/supertool/skills"
        if [ -d "skills" ]; then
            cp -R skills/* "$DEB_DIR/root/usr/local/share/supertool/skills/" 2>/dev/null || true
        fi

        cat > "$DEB_DIR/root/DEBIAN/postinst" << 'POSTINST'
#!/bin/bash
set -e
chmod 755 /usr/local/bin/stool 2>/dev/null || true
LOGGED_IN_USER=$(logname 2>/dev/null || echo "${SUDO_USER:-}")
if [ -n "$LOGGED_IN_USER" ]; then
    USER_HOME=$(eval echo "~${LOGGED_IN_USER}")
    if [ -d "$USER_HOME" ]; then
        DATA_DIR="${USER_HOME}/.supertool"
        mkdir -p "${DATA_DIR}"
        chown "${LOGGED_IN_USER}" "${DATA_DIR}"
        USER_SKILLS="${DATA_DIR}/skills"
        if [ ! -e "$USER_SKILLS" ]; then
            ln -sf /usr/local/share/supertool/skills "$USER_SKILLS"
            chown -h "${LOGGED_IN_USER}" "$USER_SKILLS" 2>/dev/null || true
        fi
    fi
fi
echo "✅ SuperTool installed successfully"
exit 0
POSTINST
        chmod 755 "$DEB_DIR/root/DEBIAN/postinst"

        dpkg-deb -b "$DEB_DIR/root" "$PKG_OUTPUT/SuperTool-${VERSION}-linux-amd64.deb"
        rm -rf "$DEB_DIR"
        echo "✅ deb → $PKG_OUTPUT/SuperTool-${VERSION}-linux-amd64.deb"
    fi
}

# ═══════════════════════════════════════════
# Windows: 生成 .exe (NSIS) + .msi (WiX)
# ═══════════════════════════════════════════
build_windows_all() {
    local arch="$1"
    build_cli "$arch"
    tauri_build

    mkdir -p "$PKG_OUTPUT"

    # ── 1. 复制原生 .exe (NSIS) ──
    local EXE_FILE=""
    for d in target/release/bundle/nsis/*.exe target/release/bundle/*.exe; do
        if [ -f "$d" ]; then EXE_FILE="$d"; break; fi
    done
    if [ -n "$EXE_FILE" ]; then
        cp -f "$EXE_FILE" "$PKG_OUTPUT/"
        echo "✅ exe → $PKG_OUTPUT/$(basename "$EXE_FILE")"
    fi

    # ── 2. 复制原生 .msi (WiX) ──
    local MSI_FILE=""
    for d in target/release/bundle/msi/*.msi; do
        if [ -f "$d" ]; then MSI_FILE="$d"; break; fi
    done
    if [ -n "$MSI_FILE" ]; then
        cp -f "$MSI_FILE" "$PKG_OUTPUT/"
        echo "✅ msi → $PKG_OUTPUT/$(basename "$MSI_FILE")"
    fi

    echo "💡 Windows: CLI 已通过 tauri.conf.json resources 打包进应用"
}

# ═══════════════════════════════════════════
# 分发到各平台
# ═══════════════════════════════════════════
dispatch_build() {
    local arch="$1"
    local os
    os="$(uname)"

    case "$os" in
        Darwin)
            build_macos_all "$arch"
            ;;
        Linux)
            build_linux_all "$arch"
            ;;
        MINGW*|MSYS*|CYGWIN*|Windows_NT)
            build_windows_all "$arch"
            ;;
        *)
            echo "❌ 不支持的操作系统: $os"
            exit 1
            ;;
    esac
}

case "$MODE" in
  pre-build)
    build_cli "$ARCH"
    ;;
  dmg)
    # 只生成 dmg，不生成 pkg
    build_cli "$ARCH"
    pnpm tauri build --bundles dmg,app
    mkdir -p "$PKG_OUTPUT"
    rm -f target/release/bundle/macos/rw.*.dmg
    DMG_SRC=""
    for d in target/release/bundle/dmg/*.dmg target/release/bundle/macos/*.dmg; do
        if [ -f "$d" ]; then DMG_SRC="$d"; break; fi
    done
    if [ -n "$DMG_SRC" ]; then
        cp -f "$DMG_SRC" "$PKG_OUTPUT/"
        echo "✅ dmg → $PKG_OUTPUT/$(basename "$DMG_SRC")"
    fi
    ;;
  pkg)
    # 只生成 pkg installer（含 CLI + Skills），跳过 dmg
    build_cli "$ARCH"
    pnpm tauri build --bundles app
    
    mkdir -p "$PKG_OUTPUT"
    
    APP_PATH=""
    for d in target/release/bundle/macos/*.app target/release/bundle/osx/*.app; do
        if [ -d "$d" ]; then APP_PATH="$d"; break; fi
    done
    if [ -z "$APP_PATH" ]; then
        echo "❌ .app bundle not found"; exit 1
    fi
    echo "📱 Found app: $APP_PATH"
    
    PKG_DIR="pkg-build"
    rm -rf "$PKG_DIR"
    mkdir -p "$PKG_DIR/scripts"
    
    # postinstall：从 /Applications/SuperTool.app 内部取出 stool CLI + skills
    cat > "$PKG_DIR/scripts/postinstall" << 'POSTINSTALL'
#!/bin/bash
set -e
INSTALL_DIR="$2"
APP_DIR="${INSTALL_DIR}/SuperTool.app"
echo "🔧 SuperTool postinstall..."

CLI_SRC="${APP_DIR}/Contents/Resources/_up_/target/release/stool"
if [ -f "$CLI_SRC" ]; then
    mkdir -p /usr/local/bin
    cp -f "$CLI_SRC" /usr/local/bin/stool
    chmod 755 /usr/local/bin/stool
    echo "✅ stool → /usr/local/bin/stool"
fi

SKILLS_SRC="${APP_DIR}/Contents/Resources/_up_/skills"
if [ -d "$SKILLS_SRC" ]; then
    mkdir -p /usr/local/share/supertool/skills
    cp -R "$SKILLS_SRC"/* /usr/local/share/supertool/skills/ 2>/dev/null || true
    echo "✅ skills → /usr/local/share/supertool/skills"
fi

LOGGED_IN_USER=$(stat -f '%Su' /dev/console 2>/dev/null || echo "")
if [ -n "$LOGGED_IN_USER" ] && [ "$LOGGED_IN_USER" != "root" ]; then
    USER_HOME=$(dscl . -read "/Users/${LOGGED_IN_USER}" NFSHomeDirectory 2>/dev/null | cut -d' ' -f2 || echo "/Users/${LOGGED_IN_USER}")
    if [ -n "$USER_HOME" ] && [ -d "$USER_HOME" ]; then
        DATA_DIR="${USER_HOME}/.supertool"
        mkdir -p "${DATA_DIR}"
        chown "${LOGGED_IN_USER}" "${DATA_DIR}"
        USER_SKILLS="${DATA_DIR}/skills"
        if [ ! -e "$USER_SKILLS" ]; then
            ln -sf /usr/local/share/supertool/skills "$USER_SKILLS"
            chown -h "${LOGGED_IN_USER}" "$USER_SKILLS"
            echo "✅ Symlink: ${USER_SKILLS} -> /usr/local/share/supertool/skills"
        fi
    fi
fi
echo "✅ SuperTool installation complete!"
exit 0
POSTINSTALL
    chmod 755 "$PKG_DIR/scripts/postinstall"
    
    # 使用 pkgbuild --component 标准方式打包 .app
    pkgbuild --component "$APP_PATH" \
        --identifier "com.fufengyuan.supertool" \
        --version "$VERSION" \
        --install-location "/Applications" \
        --scripts "$PKG_DIR/scripts" \
        "$PKG_OUTPUT/SuperTool-${VERSION}.pkg"
    
    rm -rf "$PKG_DIR"
    echo "✅ pkg → $PKG_OUTPUT/SuperTool-${VERSION}.pkg"
    ;;
  full)
    # 同时生成 dmg + pkg
    dispatch_build "$ARCH"
    ;;
  *)
    echo "用法: $0 {pre-build|dmg|pkg|full} [native|arm64|x64|all]"
    exit 1
    ;;
esac

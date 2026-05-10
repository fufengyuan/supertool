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

# ═══════════════════════════════════════════
# macOS → .pkg (pkgbuild + productbuild)
# ═══════════════════════════════════════════
build_macos_pkg() {
    local arch="$1"
    echo "📦 Building Tauri app + macOS pkg installer..."

    build_cli "$arch"
    echo "🔨 Building Tauri app..."
    pnpm tauri build

    local VERSION="4.1.0"
    local PKG_DIR="pkg-build"
    local PKG_OUTPUT="target/release"

    local APP_PATH=""
    for d in target/release/bundle/macos/*.app target/release/bundle/osx/*.app; do
        if [ -d "$d" ]; then APP_PATH="$d"; break; fi
    done
    if [ -z "$APP_PATH" ]; then
        echo "❌ .app bundle not found"; exit 1
    fi
    echo "📱 Found app: $APP_PATH"

    rm -rf "$PKG_DIR"
    mkdir -p "$PKG_DIR/app" "$PKG_DIR/scripts" "$PKG_DIR/resources" "$PKG_OUTPUT"
    cp -R "$APP_PATH" "$PKG_DIR/app/"

    # CLI component
    local CLI_DIR="$PKG_DIR/cli/usr/local/bin"
    mkdir -p "$CLI_DIR"
    cp -f cli/target/release/stool "$CLI_DIR/"
    chmod 755 "$CLI_DIR/stool"

    # Skills component
    local SKILLS_DIR="$PKG_DIR/skills/usr/local/share/supertool/skills"
    mkdir -p "$SKILLS_DIR"
    if [ -d "skills" ]; then
        cp -R skills/* "$SKILLS_DIR/" 2>/dev/null || true
    fi

    # postinstall
    cat > "$PKG_DIR/scripts/postinstall" << 'POSTINSTALL'
#!/bin/bash
set -e
TARGET_DIR="$3"
echo "🔧 SuperTool postinstall..."
if [ -f "${TARGET_DIR}/usr/local/bin/stool" ]; then
    chmod 755 "${TARGET_DIR}/usr/local/bin/stool"
    echo "✅ stool → /usr/local/bin/stool"
fi
if [ -d "${TARGET_DIR}/usr/local/share/supertool/skills" ]; then
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

    # Build component packages
    pkgbuild --root "$PKG_DIR/app" \
        --identifier "com.fufengyuan.supertool.tauri" \
        --version "$VERSION" \
        --install-location "/Applications" \
        "$PKG_DIR/SuperTool-App.pkg"

    pkgbuild --root "$PKG_DIR/cli" \
        --identifier "com.fufengyuan.supertool.cli" \
        --version "$VERSION" \
        --install-location "/" \
        "$PKG_DIR/SuperTool-CLI.pkg"

    pkgbuild --root "$PKG_DIR/skills" \
        --identifier "com.fufengyuan.supertool.skills" \
        --version "$VERSION" \
        --install-location "/" \
        "$PKG_DIR/SuperTool-Skills.pkg"

    # Distribution XML
    cat > "$PKG_DIR/distribution.xml" << EOF
<?xml version="1.0" encoding="utf-8"?>
<installer-gui-script minSpecVersion="2">
    <title>SuperTool ${VERSION}</title>
    <organization>com.fufengyuan.supertool</organization>
    <domains enable_anySystem="false" enable_anyUser="true" enable_currentUserOnly="false" enable_localSystem="true"/>
    <options require-scripts="false" customize="allow" rootVolumeOnly="true"/>
    <welcome file="welcome.html" mime-type="text/html"/>
    <conclusion file="conclusion.html" mime-type="text/html"/>
    <choices-outline>
        <line choice="choice_app"/>
        <line choice="choice_cli"/>
        <line choice="choice_skills"/>
    </choices-outline>
    <choice id="choice_app" title="SuperTool 应用" description="安装到 /Applications" enabled="false" selected="true" start_selected="true" visible="true">
        <pkg-ref id="com.fufengyuan.supertool.tauri"/>
    </choice>
    <choice id="choice_cli" title="CLI 命令行工具" description="安装 stool 到 /usr/local/bin" start_selected="true">
        <pkg-ref id="com.fufengyuan.supertool.cli"/>
    </choice>
    <choice id="choice_skills" title="AI 技能库" description="安装到 /usr/local/share/supertool/skills" start_selected="true">
        <pkg-ref id="com.fufengyuan.supertool.skills"/>
    </choice>
    <pkg-ref id="com.fufengyuan.supertool.tauri" version="${VERSION}" onConclusion="none">SuperTool-App.pkg</pkg-ref>
    <pkg-ref id="com.fufengyuan.supertool.cli" version="${VERSION}" onConclusion="none" auth="root">SuperTool-CLI.pkg</pkg-ref>
    <pkg-ref id="com.fufengyuan.supertool.skills" version="${VERSION}" onConclusion="none" auth="root">SuperTool-Skills.pkg</pkg-ref>
</installer-gui-script>
EOF

    cat > "$PKG_DIR/resources/welcome.html" << 'EOF'
<!DOCTYPE html><html><head><meta charset="utf-8"></head>
<body style="font-family:-apple-system,sans-serif;font-size:13px">
<h2>欢迎安装 SuperTool</h2>
<p>本安装包将安装：</p>
<ul><li>SuperTool.app → /Applications/</li><li>stool CLI → /usr/local/bin/</li><li>AI Skills → /usr/local/share/supertool/skills/</li></ul>
</body></html>
EOF

    cat > "$PKG_DIR/resources/conclusion.html" << 'EOF'
<!DOCTYPE html><html><head><meta charset="utf-8"></head>
<body style="font-family:-apple-system,sans-serif;font-size:13px">
<h2>安装完成</h2>
<ul><li>从 /Applications/SuperTool-Tauri.app 启动</li><li>CLI: stool --help</li><li>数据目录: ~/.supertool/</li></ul>
</body></html>
EOF

    echo "🎁 Building distribution package..."
    productbuild --distribution "$PKG_DIR/distribution.xml" \
        --resources "$PKG_DIR/resources" \
        --package-path "$PKG_DIR" \
        "$PKG_OUTPUT/SuperTool-${VERSION}.pkg"

    rm -rf "$PKG_DIR"
    echo "✅ pkg → $PKG_OUTPUT/SuperTool-${VERSION}.pkg"
}

# ═══════════════════════════════════════════
# Linux → .deb (dpkg-deb with postinst)
# ═══════════════════════════════════════════
build_linux_deb() {
    local arch="$1"
    echo "📦 Building Tauri app + Linux deb installer..."

    # Build CLI for Linux
    echo "🔨 Building CLI for Linux..."
    cd cli
    if [[ "$arch" == "arm64" || "$arch" == "aarch64" ]]; then
        cargo build --release --target aarch64-unknown-linux-gnu
    elif [[ "$arch" == "x64" || "$arch" == "x86_64" ]]; then
        cargo build --release --target x86_64-unknown-linux-gnu
    else
        cargo build --release
    fi
    cd ..
    mkdir -p target/release
    for dir in cli/target/*/release cli/target/release; do
        if [ -f "$dir/stool" ]; then cp -f "$dir/stool" target/release/stool; break; fi
    done
    echo "✅ CLI built"

    echo "🔨 Building Tauri app..."
    pnpm tauri build

    local VERSION="4.1.0"
    local DEB_DIR="deb-build"
    local PKG_OUTPUT="target/release"

    # Find .deb from Tauri build
    local TAURI_DEB=""
    for d in target/release/bundle/deb/*.deb target/release/bundle/app/*.deb; do
        if [ -f "$d" ]; then TAURI_DEB="$d"; break; fi
    done

    # Find .appimage as fallback
    local APPIMAGE=""
    for d in target/release/bundle/appimage/*.AppImage; do
        if [ -f "$d" ]; then APPIMAGE="$d"; break; fi
    done

    if [ -z "$TAURI_DEB" ] && [ -z "$APPIMAGE" ]; then
        echo "❌ No .deb or .AppImage found from Tauri build"; exit 1
    fi

    if [ -n "$TAURI_DEB" ]; then
        echo "📱 Found Tauri deb: $TAURI_DEB"
        # Repack with CLI + skills
        rm -rf "$DEB_DIR"
        mkdir -p "$DEB_DIR"

        # Extract original deb
        dpkg-deb -R "$TAURI_DEB" "$DEB_DIR/root"

        # Add CLI
        mkdir -p "$DEB_DIR/root/usr/local/bin"
        cp -f target/release/stool "$DEB_DIR/root/usr/local/bin/"
        chmod 755 "$DEB_DIR/root/usr/local/bin/stool"

        # Add skills
        mkdir -p "$DEB_DIR/root/usr/local/share/supertool/skills"
        if [ -d "skills" ]; then
            cp -R skills/* "$DEB_DIR/root/usr/local/share/supertool/skills/" 2>/dev/null || true
        fi

        # Add/merge postinst
        local POSTINST="$DEB_DIR/root/DEBIAN/postinst"
        cat > "$POSTINST" << 'POSTINST'
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
        chmod 755 "$POSTINST"

        # Rebuild deb
        mkdir -p "$PKG_OUTPUT"
        dpkg-deb -b "$DEB_DIR/root" "$PKG_OUTPUT/SuperTool-${VERSION}-linux-amd64.deb"
        rm -rf "$DEB_DIR"
        echo "✅ deb → $PKG_OUTPUT/SuperTool-${VERSION}-linux-amd64.deb"
    else
        echo "⚠️  No .deb from Tauri, building custom deb from AppImage..."
        rm -rf "$DEB_DIR"
        mkdir -p "$DEB_DIR/root/opt/supertool"
        cp "$APPIMAGE" "$DEB_DIR/root/opt/supertool/SuperTool.AppImage"
        chmod 755 "$DEB_DIR/root/opt/supertool/SuperTool.AppImage"

        mkdir -p "$DEB_DIR/root/usr/local/bin"
        cp -f target/release/stool "$DEB_DIR/root/usr/local/bin/"
        chmod 755 "$DEB_DIR/root/usr/local/bin/stool"

        mkdir -p "$DEB_DIR/root/usr/local/share/supertool/skills"
        if [ -d "skills" ]; then
            cp -R skills/* "$DEB_DIR/root/usr/local/share/supertool/skills/" 2>/dev/null || true
        fi

        mkdir -p "$DEB_DIR/root/usr/share/applications"
        cat > "$DEB_DIR/root/usr/share/applications/supertool.desktop" << 'DESKTOP'
[Desktop Entry]
Name=SuperTool
Exec=/opt/supertool/SuperTool.AppImage
Icon=supertool
Type=Application
Categories=Utility;
DESKTOP

        mkdir -p "$DEB_DIR/root/DEBIAN"
        cat > "$DEB_DIR/root/DEBIAN/control" << EOF
Package: supertool
Version: ${VERSION}
Section: utility
Priority: optional
Architecture: amd64
Maintainer: fufengyuan
Description: SuperTool - Cross-platform desktop operations management tool
EOF

        cat > "$DEB_DIR/root/DEBIAN/postinst" << 'POSTINST'
#!/bin/bash
set -e
chmod 755 /usr/local/bin/stool 2>/dev/null || true
echo "✅ SuperTool installed"
exit 0
POSTINST
        chmod 755 "$DEB_DIR/root/DEBIAN/postinst"

        mkdir -p "$PKG_OUTPUT"
        dpkg-deb -b "$DEB_DIR/root" "$PKG_OUTPUT/SuperTool-${VERSION}-linux-amd64.deb"
        rm -rf "$DEB_DIR"
        echo "✅ deb → $PKG_OUTPUT/SuperTool-${VERSION}-linux-amd64.deb"
    fi
}

# ═══════════════════════════════════════════
# Windows → .msi (WiX via Tauri + CLI injection)
# ═══════════════════════════════════════════
build_windows_msi() {
    local arch="$1"
    echo "📦 Building Tauri app + Windows MSI installer..."

    # Build CLI for Windows
    echo "🔨 Building CLI for Windows..."
    cd cli
    cargo build --release --target x86_64-pc-windows-msvc
    cd ..
    mkdir -p target/release
    cp -f cli/target/x86_64-pc-windows-msvc/release/stool.exe target/release/stool.exe 2>/dev/null || \
    cp -f cli/target/release/stool.exe target/release/stool.exe 2>/dev/null || true
    echo "✅ CLI built"

    echo "🔨 Building Tauri app (MSI via WiX)..."
    pnpm tauri build

    local VERSION="4.1.0"
    local PKG_OUTPUT="target/release"

    # Find MSI from Tauri build
    local MSI_FILE=""
    for d in target/release/bundle/msi/*.msi; do
        if [ -f "$d" ]; then MSI_FILE="$d"; break; fi
    done

    if [ -n "$MSI_FILE" ]; then
        cp -f "$MSI_FILE" "$PKG_OUTPUT/SuperTool-${VERSION}-windows-x64.msi"
        echo "✅ msi → $PKG_OUTPUT/SuperTool-${VERSION}-windows-x64.msi"
        echo "⚠️  Note: CLI (stool.exe) bundled in app resources. Manual PATH setup may be needed."
    else
        echo "❌ No .msi found from Tauri build"
        exit 1
    fi
}

# ═══════════════════════════════════════════
# Dispatch by OS
# ═══════════════════════════════════════════
dispatch_build() {
    local arch="$1"
    local os
    os="$(uname)"

    case "$os" in
        Darwin)
            build_macos_pkg "$arch"
            ;;
        Linux)
            build_linux_deb "$arch"
            ;;
        MINGW*|MSYS*|CYGWIN*|Windows_NT)
            build_windows_msi "$arch"
            ;;
        *)
            echo "❌ Unsupported OS: $os"
            exit 1
            ;;
    esac
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
  pkg)
    dispatch_build "$ARCH"
    ;;
  *)
    echo "Usage: $0 {pre-build|full|pkg} [native|arm64|x64|all]"
    exit 1
    ;;
esac

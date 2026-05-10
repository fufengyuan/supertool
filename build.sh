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

build_pkg() {
    local arch="$1"
    echo "📦 Building Tauri app + macOS pkg installer..."

    # Step 1: Build CLI
    build_cli "$arch"

    # Step 2: Build Tauri (.app + .dmg)
    echo "🔨 Building Tauri app..."
    pnpm tauri build

    # Step 3: Create .pkg installer (macOS only)
    if [[ "$(uname)" != "Darwin" ]]; then
        echo "⚠️  .pkg packaging requires macOS. Skipping."
        return
    fi

    local APP_NAME="SuperTool-Tauri"
    local APP_BUNDLE="${APP_NAME}.app"
    local VERSION="4.0.0"
    local PKG_DIR="pkg-build"
    local PKG_OUTPUT="target/release"

    # Find the .app bundle
    local APP_PATH=""
    for d in target/release/bundle/macos/*.app target/release/bundle/osx/*.app; do
        if [ -d "$d" ]; then APP_PATH="$d"; break; fi
    done
    if [ -z "$APP_PATH" ]; then
        echo "❌ .app bundle not found. Tauri build may have failed."
        exit 1
    fi

    echo "📱 Found app: $APP_PATH"

    # Clean previous pkg build
    rm -rf "$PKG_DIR"
    mkdir -p "$PKG_DIR/app" "$PKG_DIR/scripts" "$PKG_DIR/resources" "$PKG_OUTPUT"

    # ─── Component: App Bundle ───
    cp -R "$APP_PATH" "$PKG_DIR/app/"

    # ─── Component: CLI Binary ───
    local CLI_DIR="$PKG_DIR/cli/usr/local/bin"
    mkdir -p "$CLI_DIR"
    cp -f cli/target/release/stool "$CLI_DIR/"
    chmod 755 "$CLI_DIR/stool"

    # ─── Component: Skills ───
    local SKILLS_DIR="$PKG_DIR/skills/usr/local/share/supertool/skills"
    mkdir -p "$SKILLS_DIR"
    if [ -d "skills" ]; then
        cp -R skills/* "$SKILLS_DIR/" 2>/dev/null || true
    fi

    # ─── Postinstall Script ───
    cat > "$PKG_DIR/scripts/postinstall" << 'POSTINSTALL'
#!/bin/bash
set -e

APP_NAME="SuperTool-Tauri"
TARGET_DIR="$3"  # Installation destination (from pkg installer)
echo "🔧 SuperTool postinstall..."

# Install CLI (already placed by pkg, just ensure permissions)
if [ -f "${TARGET_DIR}/usr/local/bin/stool" ]; then
    chmod 755 "${TARGET_DIR}/usr/local/bin/stool"
    echo "✅ stool → /usr/local/bin/stool"
fi

# Install skills (already placed by pkg)
if [ -d "${TARGET_DIR}/usr/local/share/supertool/skills" ]; then
    echo "✅ skills → /usr/local/share/supertool/skills"
fi

# Ensure ~/.supertool/ data directory exists for current user
LOGGED_IN_USER=$(stat -f '%Su' /dev/console 2>/dev/null || echo "")
if [ -n "$LOGGED_IN_USER" ] && [ "$LOGGED_IN_USER" != "root" ]; then
    USER_HOME=$(dscl . -read "/Users/${LOGGED_IN_USER}" NFSHomeDirectory 2>/dev/null | cut -d' ' -f2 || echo "/Users/${LOGGED_IN_USER}")
    if [ -n "$USER_HOME" ] && [ -d "$USER_HOME" ]; then
        DATA_DIR="${USER_HOME}/.supertool"
        mkdir -p "${DATA_DIR}"
        chown "${LOGGED_IN_USER}" "${DATA_DIR}"

        # Symlink skills to user data dir
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

    # ─── Build Component Packages ───
    echo "📦 Building component packages..."

    # App component
    pkgbuild --root "$PKG_DIR/app" \
        --identifier "com.fufengyuan.supertool.tauri" \
        --version "$VERSION" \
        --install-location "/Applications" \
        "$PKG_DIR/SuperTool-App.pkg" \
        2>/dev/null || pkgbuild --root "$PKG_DIR/app" \
        --identifier "com.fufengyuan.supertool.tauri" \
        --version "$VERSION" \
        --install-location "/Applications" \
        "$PKG_DIR/SuperTool-App.pkg"

    # CLI component
    pkgbuild --root "$PKG_DIR/cli" \
        --identifier "com.fufengyuan.supertool.cli" \
        --version "$VERSION" \
        --install-location "/" \
        "$PKG_DIR/SuperTool-CLI.pkg"

    # Skills component
    pkgbuild --root "$PKG_DIR/skills" \
        --identifier "com.fufengyuan.supertool.skills" \
        --version "$VERSION" \
        --install-location "/" \
        "$PKG_DIR/SuperTool-Skills.pkg"

    # ─── Create Distribution XML ───
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
    <choice id="choice_app" title="SuperTool Application" description="Install SuperTool to /Applications" enabled="false" selected="true" start_selected="true" visible="true">
        <pkg-ref id="com.fufengyuan.supertool.tauri"/>
    </choice>
    <choice id="choice_cli" title="CLI (stool)" description="Install stool CLI to /usr/local/bin" start_selected="true">
        <pkg-ref id="com.fufengyuan.supertool.cli"/>
    </choice>
    <choice id="choice_skills" title="Skills" description="Install AI skills to /usr/local/share/supertool/skills" start_selected="true">
        <pkg-ref id="com.fufengyuan.supertool.skills"/>
    </choice>
    <pkg-ref id="com.fufengyuan.supertool.tauri" version="${VERSION}" onConclusion="none">SuperTool-App.pkg</pkg-ref>
    <pkg-ref id="com.fufengyuan.supertool.cli" version="${VERSION}" onConclusion="none" auth="root">SuperTool-CLI.pkg</pkg-ref>
    <pkg-ref id="com.fufengyuan.supertool.skills" version="${VERSION}" onConclusion="none" auth="root">SuperTool-Skills.pkg</pkg-ref>
</installer-gui-script>
EOF

    # Create welcome/conclusion HTML
    cat > "$PKG_DIR/resources/welcome.html" << 'EOF'
<!DOCTYPE html>
<html><head><meta charset="utf-8"></head>
<body style="font-family: -apple-system, sans-serif; font-size: 13px;">
<h2>Welcome to SuperTool</h2>
<p>This installer will set up SuperTool on your Mac.</p>
<ul>
<li><strong>SuperTool.app</strong> — Desktop application in /Applications</li>
<li><strong>stool</strong> — CLI tool in /usr/local/bin</li>
<li><strong>Skills</strong> — AI skills in /usr/local/share/supertool/skills</li>
</ul>
</body></html>
EOF

    cat > "$PKG_DIR/resources/conclusion.html" << 'EOF'
<!DOCTYPE html>
<html><head><meta charset="utf-8"></head>
<body style="font-family: -apple-system, sans-serif; font-size: 13px;">
<h2>Installation Complete</h2>
<p>SuperTool has been installed successfully.</p>
<ul>
<li>Launch from /Applications/SuperTool-Tauri.app</li>
<li>CLI: <code>stool --help</code></li>
<li>Data directory: ~/.supertool/</li>
</ul>
</body></html>
EOF

    # ─── Build Final Distribution Package ───
    echo "🎁 Building distribution package..."
    productbuild --distribution "$PKG_DIR/distribution.xml" \
        --resources "$PKG_DIR/resources" \
        --package-path "$PKG_DIR" \
        "$PKG_OUTPUT/SuperTool-${VERSION}.pkg"

    # Clean up
    rm -rf "$PKG_DIR"

    echo ""
    echo "✅ pkg installer → $PKG_OUTPUT/SuperTool-${VERSION}.pkg"
    echo "📋 Contents:"
    echo "   • /Applications/SuperTool-Tauri.app"
    echo "   • /usr/local/bin/stool"
    echo "   • /usr/local/share/supertool/skills/"
    echo "   • ~/.supertool/ (data directory + skills symlink)"
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
    build_pkg "$ARCH"
    ;;
  *)
    echo "Usage: $0 {pre-build|full|pkg} [native|arm64|x64|all]"
    exit 1
    ;;
esac

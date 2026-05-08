# OpenVPN 内置二进制

应用自带的 OpenVPN 客户端二进制文件，避免用户手动安装。

## 当前状态

| 平台 | 状态 | 大小 | 来源 |
|------|------|------|------|
| Linux x64 | ✅ 已就绪 | ~931 KB | 系统 openvpn 包 |
| Linux ARM64 | ✅ 已就绪 | ~963 KB | Debian sid arm64 |
| macOS ARM64 | ✅ 已修复 | ~1.5 MB | Homebrew + install_name_tool + codesign |
| macOS x64 | ⚠️ 需手动修复 | ~1.5 MB | 需 Intel Mac 重新构建 |
| Windows x64 | ⚠️ 需手动放入 | ~3 MB | OpenVPN 官网 |

## 修复 macOS 二进制

macOS 下 OpenVPN 依赖动态库，需要正确设置 `@loader_path` 并签名：

```bash
# ARM64 (Apple Silicon)
cd resources/openvpn/macos-arm64
# 1. 从 Homebrew 复制干净二进制
cp $(brew --prefix openvpn)/sbin/openvpn ./openvpn
# 2. 修复 dylib 路径
for lib in liblzo2.2.dylib liblz4.1.dylib libpkcs11-helper.1.dylib libssl.3.dylib libcrypto.3.dylib; do
  brew_path=$(otool -L ./openvpn | grep "$lib" | awk '{print $1}' | head -1)
  [ -n "$brew_path" ] && install_name_tool -change "$brew_path" "@loader_path/$lib" ./openvpn
  [ -f "$brew_path" ] && cp "$brew_path" ./$lib
done
# 3. 签名（关键！不签名会被 macOS 杀掉）
codesign -s - --force ./openvpn ./*.dylib
```

## Fallback 机制

应用会自动按以下顺序查找 OpenVPN：
1. 内置二进制 `resources/openvpn/macos-arm64/openvpn`
2. Homebrew 安装路径 `/opt/homebrew/opt/openvpn/sbin/openvpn`（ARM64）或 `/usr/local/opt/openvpn/sbin/openvpn`（x64）
3. 系统 PATH 中的 `openvpn` 命令

## 放入二进制文件

### macOS (ARM64 / Apple Silicon)
```bash
brew install openvpn
cp $(brew --prefix openvpn)/sbin/openvpn resources/openvpn/macos-arm64/openvpn
chmod 755 resources/openvpn/macos-arm64/openvpn
```

### macOS (x64 / Intel)
```bash
brew install openvpn
cp $(brew --prefix openvpn)/sbin/openvpn resources/openvpn/macos-x64/openvpn
chmod 755 resources/openvpn/macos-x64/openvpn
```

### Windows x64
1. 从 [openvpn.net](https://openvpn.net/community-downloads/) 下载 Windows 安装程序
2. 运行安装程序（默认安装到 `C:\Program Files\OpenVPN`）
3. 复制以下文件到 `resources/openvpn/win-x64/`：
   - `bin\openvpn.exe`
   - `bin\libssl-3-x64.dll`
   - `bin\libcrypto-3-x64.dll`

## 目录结构

```
resources/openvpn/
├── linux-x64/openvpn      ← Linux x86_64
├── linux-arm64/openvpn    ← Linux ARM64
├── macos-arm64/openvpn    ← macOS Apple Silicon
├── macos-x64/openvpn      ← macOS Intel
└── win-x64/               ← Windows x64
    ├── openvpn.exe
    ├── libssl-3-x64.dll
    └── libcrypto-3-x64.dll
```

# 打包分发：CLI 架构串包问题

日期：2026-09-01
涉及提交：`747d41cb`（fix 打包）

## 现象

打包出的 x64 `.pkg`，安装后 `/usr/local/bin/stool` 是 **arm64** 架构。

## 根因（两层叠加，比表面更深）

### 1. build_cli 的复制循环从未生效过

`build.sh` 的 `build_cli()` 开头会 `cd cli`，然后：

```bash
cd cli
cargo build --release --target "$target"
# ...
for dir in target/release target/*/release; do    # ← 这是 cli/target/...
    if [ -f "$dir/stool" ] || [ -f "$dir/stool.exe" ]; then
        cp "$dir/stool"* ../target/release/ 2>/dev/null || true
        break
    fi
done
```

**关键**：项目是 **cargo workspace**，产物统一输出到 **workspace 根的 `target/`**，
不是 `cli/target/`。所以循环里那两个路径（`cli/target/release`、`cli/target/*/release`）
**都不存在**，循环体一次都没进。

再叠加 `2>/dev/null || true` 把错误吞掉 —— 这个复制步骤**从来没成功过**，也没人发现。

### 2. 根 target/release/stool 是陈旧残留

某次手工 `cargo build --release`（Apple Silicon 本机 = arm64）留下的二进制，
因为第 1 点的复制从未生效，一直躺在 `target/release/stool` 没被更新。

而 `tauri.conf.json` 的 resources 正是：

```json
"resources": [
  "../target/release/stool",
  "../skills"
]
```

tauri 直接把它打进 app bundle，postinstall 再从
`Contents/Resources/_up_/target/release/stool` 取出装到 `/usr/local/bin/stool`。

**结论**：不只是 x64 装错 —— 在此之前 **x64 和 arm64 包装的都是同一个陈旧 arm64 CLI**，
x64 只是先暴露出症状（arm64 包装了 arm64，看起来"正常"）。

## 修复（build.sh）

### 1. 按目标精确取产物（不再遍历猜测）

```bash
# cargo workspace 的产物统一在 workspace 根 target/，不在 cli/target/
if [ -n "$target" ]; then
    cargo build --release --target "$target"
    SRC_DIR="../target/${target}/release"
else
    cargo build --release
    SRC_DIR="../target/release"
fi

DEST_DIR="../target/release"
mkdir -p "$DEST_DIR"

# native 构建时源就是目标，无需复制（否则 cp 到自己）
if [ "$SRC_DIR" != "$DEST_DIR" ]; then
    rm -f "$DEST_DIR/stool" "$DEST_DIR/stool.exe"
    if [ -f "$SRC_DIR/stool" ]; then
        cp -f "$SRC_DIR/stool" "$DEST_DIR/stool"
    elif [ -f "$SRC_DIR/stool.exe" ]; then
        cp -f "$SRC_DIR/stool.exe" "$DEST_DIR/stool.exe"
    else
        echo "❌ 未找到 CLI 产物: $SRC_DIR/stool（构建目标 ${target:-native}）"
        exit 1          # ← 不再静默吞错
    fi
fi
```

### 2. 两道架构校验关卡

**关卡一（构建后）**：校验 `target/release/stool`

**关卡二（打包前，最终交付物）**：校验 app bundle 内嵌的 CLI
`${APP_PATH}/Contents/Resources/_up_/target/release/stool`

架构不符立即 `exit 1`，绝不把错误产物打进 pkg。两处打包函数
（`build_macos_all` / `build_pkg_one`）都加了关卡二。

### 3. 校验用 bash 原生 case，不用 grep

```bash
local cli_out cli_arch="" want_arch=""
cli_out=$(file -b "$embedded_cli" 2>/dev/null || echo "")
case "$cli_out" in
    *x86_64*) cli_arch="x86_64" ;;
    *arm64*)  cli_arch="arm64" ;;
esac
```

**原因**：某些精简版 grep（如 WorkBuddy 沙箱内的）不支持 BRE alternation `a\|b`，
`grep -o 'x86_64\|arm64'` 会**静默失配**（不报错、返回空），校验形同虚设。
改用 bash 原生 `case` 模式匹配完全不依赖 grep 正则。

## 验证

| 场景 | 结果 |
|---|---|
| `./build.sh pre-build x64` | `✅ CLI 架构校验通过: [x86_64]`，`file` 确认 x86_64 |
| `./build.sh pre-build arm64` | `✅ CLI 架构校验通过: [arm64]`，`file` 确认 arm64 |
| 故意把 arm64 污染到 `target/release/stool` 后跑 x64 | 脚本从 `target/x86_64-apple-darwin/release/stool` 精确取产物覆盖污染，校验通过 `[x86_64]` |
| 修复过程中路径写错时 | `❌ 未找到 CLI 产物` 报错退出（证明不再静默吞错） |

即：**即使存在历史污染，修复后的脚本也会自动纠正**，不再依赖手工清理。

## 环境限制（非代码问题）

完整 `tauri build` 在 WorkBuddy 沙箱内跑不通：`tauri build` 会自动调 `pnpm install`，
而 pnpm 的 symlink 操作被 fs 拦截层拒绝（`EEXIST: file already exists, symlink ...`，
发生在 `node_modules/.pnpm` 的 `symlinkAllModules` 阶段）。

这是沙箱限制，**真实终端不受影响**。核心链路（CLI 构建 + 架构校验）已完整验证；
打包脚本的最终校验关卡会在真实打包时守住。

## 使用建议

打 x64 包前先重建 CLI（确保 `target/release/stool` 是 x86_64）：

```bash
./build.sh pre-build x64    # 重建 x64 CLI + 架构校验
./build.sh pkg x64          # 打包，脚本会再校验 app 内嵌 CLI
```

打包成功时会看到：

```
✅ CLI 架构校验通过: [x86_64]
✅ app 内置 CLI 架构校验通过: [x86_64]
✅ pkg → target/release/SuperTool-<version>-x64.pkg
```

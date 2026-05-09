#!/usr/bin/env python3
"""Fix SVG that was incorrectly placed inside Vue template expressions."""
import os, re

root = os.path.expanduser('~/workspace/supertool')

# Replacements: find SVG-inside-expression and revert to original emoji
# Pattern: SVG inside {{ ... }} or :attr="..."
FIXES = {
    'src/views/backup/DataBackup.vue': [
        # Line 18: {{ isExporting ? '导出中...' : '<svg...>' }}
        (r"'<svg[^>]*>.*?</svg>\s*完整备份",
         "'💾 完整备份"),
        # Line 46: {{ isImporting ? '导入中...' : '<svg...>' }}
        (r"'<svg[^>]*>.*?</svg>\s*导入备份",
         "'📂 导入备份"),
        # Line 162: {{ dataDir.saving ? '保存中...' : '<svg...>' }}
        (r"'<svg[^>]*>.*?</svg>\s*保存",
         "'💾 保存"),
    ],
    'src/views/db/DataSync.vue': [
        # Line 286: {{ comparing ? "对比中..." : `<svg...>  对比 ${...}` }}
        (r'`<svg[^>]*>.*?</svg>  对比',
         '`🔍 对比'),
    ],
    'src/views/db/StructureSync.vue': [
        # Line 110: {{ comparing ? '对比中...' : `<svg...>  对比 ${...}` }}
        (r'`<svg[^>]*>.*?</svg>  对比',
         '`🔍 对比'),
        # Line 253: {{ execResult.success ? '<svg...>  同步成功' : '<svg...>  同步失败' }}
        (r"'<svg[^>]*>.*?</svg>\s*同步成功",
         "'✅ 同步成功"),
        (r"'<svg[^>]*>.*?</svg>\s*同步失败",
         "'❌ 同步失败"),
    ],
    'src/views/db/DBBackup.vue': [
        # Line 30: {{ creating ? '备份中...' : '<svg...>  新建备份' }}
        (r"'<svg[^>]*>.*?</svg>\s*新建备份",
         "'💾 新建备份"),
    ],
    'src/views/nginx/NginxManager.vue': [
        # Line 74: {{ loading ? '<svg...>  加载中...' : '<svg...>  获取配置' }}
        (r"'<svg[^>]*>.*?</svg>\s*加载中\.\.\.",
         "'⏳ 加载中..."),
        (r"'<svg[^>]*>.*?</svg>\s*获取配置",
         "'📥 获取配置"),
        # Line 108: {{ testResult.passed ? '<svg...>  配置检测通过' : '<svg...>  配置检测失败' }}
        (r"'<svg[^>]*>.*?</svg>\s*配置检测通过",
         "'✅ 配置检测通过"),
        (r"'<svg[^>]*>.*?</svg>\s*配置检测失败",
         "'❌ 配置检测失败"),
    ],
    'src/views/cicd/DeployPanel.vue': [
        # Line 102: {{ r.passed ? '<svg...>' : '<svg...>' }}
        (r"'<svg[^>]*>.*?</svg> ",
         "''"),
    ],
}

for rel_path, fixes in FIXES.items():
    full = os.path.join(root, rel_path)
    if not os.path.exists(full):
        print(f"SKIP {rel_path} (not found)")
        continue
    with open(full, 'r') as f:
        content = f.read()
    original = content
    for pattern, replacement in fixes:
        content = re.sub(pattern, replacement, content)
    if content != original:
        with open(full, 'w') as f:
            f.write(content)
        print(f"FIXED {rel_path}")
    else:
        print(f"OK   {rel_path} (no changes needed)")

# Now also scan for remaining SVGs inside template expressions
print("\n=== Checking for remaining issues ===")
for root_dir, dirs, files in os.walk(os.path.join(root, 'src/views')):
    for f in files:
        if not f.endswith('.vue'):
            continue
        path = os.path.join(root_dir, f)
        rel = os.path.relpath(path, root)
        with open(path, 'r') as fh:
            content = fh.read()
        # Find any remaining SVG inside {{ }}
        bad = re.findall(r'\{\{[^}]*<svg[^>]*>', content)
        if bad:
            print(f"  BAD {rel}: {len(bad)} SVGs in {{}}")
            for i, b in enumerate(bad[:3]):
                print(f"    [{i}]: ...{b[:80]}...")
        # Find SVG inside :attr="..."
        bad2 = re.findall(r':\w+="[^"]*<svg', content)
        if bad2:
            print(f"  BAD {rel}: {len(bad2)} SVGs in :attr=")
            for b in bad2[:3]:
                print(f"    ...{b[:80]}...")

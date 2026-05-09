#!/usr/bin/env python3
"""Audit emoji usage in Vue template files - find emoji used as icons."""
import re, os, sys

emoji_pat = re.compile(r'[\U0001F300-\U0001F9FF\U00002600-\U000027BF\U00002700-\U000027BF]')

target_files = [
    'src/views/db/DBManager.vue',
    'src/views/accounting/AccountingBook.vue',
    'src/views/vpn/VPNManager.vue',
    'src/views/projects/ProjectDetail.vue',
    'src/views/backup/DataBackup.vue',
    'src/views/todo/TodoStats.vue',
    'src/views/db/DBBackup.vue',
    'src/views/projects/ProjectItem.vue',
    'src/views/db/DataSync.vue',
    'src/views/db/StructureSync.vue',
    'src/views/db/DataGrid.vue',
    'src/views/db/RedisQueueManager.vue',
    'src/views/db/ConnectionTree.vue',
    'src/views/cicd/DeployPanel.vue',
    'src/views/projects/ProjectView.vue',
    'src/views/devtools/DevTools.vue',
]

root = os.path.expanduser('~/workspace/supertool')
for rel_path in target_files:
    full_path = os.path.join(root, rel_path)
    if not os.path.exists(full_path):
        continue
    with open(full_path, 'r') as f:
        content = f.read()
    matches = emoji_pat.findall(content)
    unique = sorted(set(matches))
    print(f"{rel_path.split('/')[-1]:30s} {len(matches):3d} total, {len(unique):2d} unique: {''.join(unique)}")

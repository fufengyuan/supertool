#!/usr/bin/env python3
import re, os, sys

root = os.path.expanduser('~/workspace/supertool')
files = [
    'src/views/db/DBManager.vue',
    'src/views/backup/DataBackup.vue', 
    'src/views/accounting/AccountingBook.vue',
    'src/views/vpn/VPNManager.vue',
    'src/views/projects/ProjectDetail.vue',
    'src/views/cicd/DeployPanel.vue',
]

emoji_pat = re.compile(r'[\U0001F300-\U0001F9FF\U00002600-\U000027BF\U00002700-\U000027BF]')

for rel in files:
    path = os.path.join(root, rel)
    if not os.path.exists(path):
        continue
    with open(path, 'r') as f:
        content = f.read()
    emojis = emoji_pat.findall(content)
    unique = sorted(set(emojis))
    
    # Check SVG count
    svg_count = content.count('<svg')
    
    print(f"{rel.split('/')[-1]:30s} SVGs: {svg_count:3d}  Emoji remaining: {len(emojis):3d} ({''.join(unique) if unique else 'none'})")
    
    # Check a sample line with SVG
    for i, line in enumerate(content.split('\n')):
        if '<svg' in line and 'stroke-width=' in line:
            print(f"  Sample L{i+1}: {line.strip()[:120]}...")
            break

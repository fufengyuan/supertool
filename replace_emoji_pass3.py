#!/usr/bin/env python3
"""
Third pass: Handle remaining emoji with FE0F variation selector and other unhandled emoji.
"""

import os
import re

# Additional emoji that have FE0F variation selector
# These were either not in the original mapping or had FE0F after them
EXTRA_EMOJI_MAP = {
    '⌨️': '<svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><rect x="2" y="4" width="20" height="16" rx="2"/><path d="M6 8h.01M10 8h.01M14 8h.01M18 8h.01M8 12h.01M12 12h.01M16 12h.01M6 16h12"/></svg>',
    '⏱️': '<svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg>',
    '🅰️': '<svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><circle cx="12" cy="12" r="10"/><text x="12" y="16" text-anchor="middle" fill="currentColor" font-size="12" font-weight="bold">A</text></svg>',
    '⬇️': '<svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><line x1="12" y1="5" x2="12" y2="19"/><polyline points="19 12 12 19 5 12"/></svg>',
    '🏳️': '<svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><path d="M4 15s1-1 4-1 5 2 8 2 4-1 4-1V3s-1 1-4 1-5-2-8-2-4 1-4 1z"/><line x1="4" y1="22" x2="4" y2="15"/></svg>',
    '🏙️': '<svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><rect x="4" y="2" width="16" height="20" rx="2"/><line x1="9" y1="6" x2="9" y2="10"/><line x1="15" y1="6" x2="15" y2="10"/><line x1="9" y1="14" x2="9" y2="18"/><line x1="15" y1="14" x2="15" y2="18"/></svg>',
    '🗺️': '<svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><circle cx="12" cy="12" r="10"/><line x1="2" y1="12" x2="22" y2="12"/><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/></svg>',
    '🗃️': '<svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><rect x="2" y="3" width="20" height="18" rx="2" ry="2"/><line x1="2" y1="9" x2="22" y2="9"/></svg>',
    # Also handle versions without FE0F
    '⌨': '<svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><rect x="2" y="4" width="20" height="16" rx="2"/><path d="M6 8h.01M10 8h.01M14 8h.01M18 8h.01M8 12h.01M12 12h.01M16 12h.01M6 16h12"/></svg>',
    '⏱': '<svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg>',
    '🅰': '<svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><circle cx="12" cy="12" r="10"/><text x="12" y="16" text-anchor="middle" fill="currentColor" font-size="12" font-weight="bold">A</text></svg>',
    '⬇': '<svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><line x1="12" y1="5" x2="12" y2="19"/><polyline points="19 12 12 19 5 12"/></svg>',
    '🏳': '<svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><path d="M4 15s1-1 4-1 5 2 8 2 4-1 4-1V3s-1 1-4 1-5-2-8-2-4 1-4 1z"/><line x1="4" y1="22" x2="4" y2="15"/></svg>',
    '🏙': '<svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><rect x="4" y="2" width="16" height="20" rx="2"/><line x1="9" y1="6" x2="9" y2="10"/><line x1="15" y1="6" x2="15" y2="10"/><line x1="9" y1="14" x2="9" y2="18"/><line x1="15" y1="14" x2="15" y2="18"/></svg>',
    '🗺': '<svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><circle cx="12" cy="12" r="10"/><line x1="2" y1="12" x2="22" y2="12"/><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/></svg>',
    '🗃': '<svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="inline-block align-text-bottom"><rect x="2" y="3" width="20" height="18" rx="2" ry="2"/><line x1="2" y1="9" x2="22" y2="9"/></svg>',
    # 🔶 already got handled but let's add it just in case
    '🔶': '<svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" class="inline-block align-text-bottom"><polygon points="12 2 22 8.5 22 15.5 12 22 2 15.5 2 8.5" fill="currentColor"/></svg>',
}

EXCLUDED_FILES = {
    'GroupTree.vue', 'ServerManager.vue', 'ServerView.vue',
    'ServerItem.vue', 'ServerForm.vue', 'ServerMonitor.vue',
    'GroupedServerSelector.vue', 'SftpPanel.vue', 'CiCdConfig.vue',
    'DeployPanel.vue', 'LogAggregator.vue'
}

def is_excluded(filepath):
    return os.path.basename(filepath) in EXCLUDED_FILES

def process_file(filepath):
    with open(filepath, 'r', encoding='utf-8') as f:
        content = f.read()
    
    tmpl_match = re.search(r'<template\b[^>]*>(.*?)</template>', content, re.DOTALL)
    if not tmpl_match:
        return False
    
    template_start = tmpl_match.start(1)
    template_end = tmpl_match.end(1)
    template_content = tmpl_match.group(1)
    
    old_template = template_content
    
    for emoji, svg in EXTRA_EMOJI_MAP.items():
        if emoji in template_content:
            template_content = template_content.replace(emoji, svg)
            print(f"  Replaced {repr(emoji)} in {os.path.basename(filepath)}")
    
    if old_template == template_content:
        return False
    
    new_content = content[:template_start] + template_content + content[template_end:]
    
    with open(filepath, 'w', encoding='utf-8') as f:
        f.write(new_content)
    
    return True

def main():
    base_dir = os.path.expanduser('~/workspace/supertool/src')
    
    for root_dir in ['views', 'components']:
        full_dir = os.path.join(base_dir, root_dir)
        if os.path.isdir(full_dir):
            for root, dirs, files in os.walk(full_dir):
                for f in sorted(files):
                    if f.endswith('.vue') and not is_excluded(os.path.join(root, f)):
                        process_file(os.path.join(root, f))
    
    print("Done!")

if __name__ == '__main__':
    main()

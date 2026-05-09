#!/usr/bin/env python3
"""
Second pass: handle remaining issues
1. Remove leftover ️ (U+FE0F) variation selector characters in templates
2. Replace 🔶 emoji
"""

import os
import re

# Map for remaining emoji
ADDITIONAL_MAP = {
    '🔶': '<svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" class="inline-block align-text-bottom"><polygon points="12 2 22 8.5 22 15.5 12 22 2 15.5 2 8.5" fill="currentColor"/></svg>',
}

EXCLUDED_FILES = {
    'GroupTree.vue', 'ServerManager.vue', 'ServerView.vue',
    'ServerItem.vue', 'ServerForm.vue', 'ServerMonitor.vue',
    'GroupedServerSelector.vue', 'SftpPanel.vue', 'CiCdConfig.vue',
    'DeployPanel.vue', 'LogAggregator.vue'
}

def is_excluded(filepath):
    basename = os.path.basename(filepath)
    return basename in EXCLUDED_FILES

def process_file_for_emoji(filepath):
    """Replace additional emoji in templates."""
    with open(filepath, 'r', encoding='utf-8') as f:
        content = f.read()
    
    # Find template section
    tmpl_match = re.search(r'<template\b[^>]*>(.*?)</template>', content, re.DOTALL)
    if not tmpl_match:
        return False
    
    template_start = tmpl_match.start(1)
    template_end = tmpl_match.end(1)
    template_content = tmpl_match.group(1)
    
    old_template = template_content
    
    # Replace additional emoji
    for emoji, svg in ADDITIONAL_MAP.items():
        template_content = template_content.replace(emoji, svg)
    
    # Remove leftover FE0F variation selector characters
    template_content = template_content.replace('\ufe0f', '')
    # Also try the part of the variation selector
    template_content = template_content.replace('️', '')
    
    if old_template == template_content:
        return False
    
    new_content = content[:template_start] + template_content + content[template_end:]
    
    with open(filepath, 'w', encoding='utf-8') as f:
        f.write(new_content)
    
    print(f"  Fixed {filepath}")
    return True

def clean_fe0f_in_templates(filepath):
    """Just remove FE0F variation selectors from templates."""
    with open(filepath, 'r', encoding='utf-8') as f:
        content = f.read()
    
    # Find template section
    tmpl_match = re.search(r'<template\b[^>]*>(.*?)</template>', content, re.DOTALL)
    if not tmpl_match:
        return False
    
    template_start = tmpl_match.start(1)
    template_end = tmpl_match.end(1)
    template_content = tmpl_match.group(1)
    
    if '\ufe0f' not in template_content and '️' not in template_content:
        return False
    
    old_template = template_content
    template_content = template_content.replace('\ufe0f', '')
    template_content = template_content.replace('️', '')
    
    if old_template == template_content:
        return False
    
    new_content = content[:template_start] + template_content + content[template_end:]
    
    with open(filepath, 'w', encoding='utf-8') as f:
        f.write(new_content)
    
    print(f"  Cleaned FE0F in {filepath}")
    return True

def main():
    base_dir = os.path.expanduser('~/workspace/supertool/src')
    
    # First pass: Process additional emoji replacements
    print("=== Pass 1: Replace additional emoji ===")
    for root_dir in ['views', 'components']:
        full_dir = os.path.join(base_dir, root_dir)
        if os.path.isdir(full_dir):
            for root, dirs, files in os.walk(full_dir):
                for f in sorted(files):
                    if f.endswith('.vue') and not is_excluded(os.path.join(root, f)):
                        process_file_for_emoji(os.path.join(root, f))
    
    # Second pass: Clean FE0F from all files (including previously excluded ones)
    print("\n=== Pass 2: Clean leftover FE0F variation selectors ===")
    for root_dir in ['views', 'components', 'layouts']:
        full_dir = os.path.join(base_dir, root_dir)
        if os.path.isdir(full_dir):
            for root, dirs, files in os.walk(full_dir):
                for f in sorted(files):
                    if f.endswith('.vue'):
                        clean_fe0f_in_templates(os.path.join(root, f))
    
    print("\nDone!")

if __name__ == '__main__':
    main()

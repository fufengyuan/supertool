#!/usr/bin/env node
// Batch-replace inline SVGs with SvgIcon component
// Usage: node replace_svg.mjs <file_path>

import fs from 'fs';
import path from 'path';

const filePath = process.argv[2];
if (!filePath) {
  console.log('Usage: node replace_svg.mjs <file_path>');
  process.exit(1);
}

let content = fs.readFileSync(filePath, 'utf-8');
const original = content;

// SVG path -> SvgIcon name mappings (for common patterns)
const patterns = [
  // === Status icons ===
  { match: /<svg[^>]*><polyline points="20 6 9 17 4 12"\/><\/svg>/g, replace: '<SvgIcon name="check" size="14" />' },
  { match: /<svg[^>]*><path d="M18 6 6 18"\/><path d="m6 6 12 12"\/><\/svg>/g, replace: '<SvgIcon name="x" size="14" />' },
  { match: /<svg[^>]*><circle cx="12" cy="12" r="10"\/><polyline points="12 6 12 12 16 14"\/><\/svg>/g, replace: '<SvgIcon name="clock" size="14" />' },

  // === Search ===
  { match: /<svg[^>]*><circle cx="11" cy="11" r="8"\/><line x1="21" y1="21" x2="16\.65" y2="16\.65"\/><\/svg>/g, replace: '<SvgIcon name="search" size="14" />' },
  { match: /<svg[^>]*><circle cx="11" cy="11" r="8" \/><path d="m21 21-4\.35-4\.35" \/><\/svg>/g, replace: '<SvgIcon name="search" size="14" />' },

  // === Navigation ===
  { match: /<svg[^>]*><polyline points="6 9 12 15 18 9"\/><\/svg>/g, replace: '<SvgIcon name="chevronDown" size="14" />' },
  { match: /<svg[^>]*><polyline points="9 18 15 12 9 6"\/><\/svg>/g, replace: '<SvgIcon name="chevronLeft" size="14" />' },
  { match: /<svg[^>]*><polyline points="15 18 9 12 15 6"\/><\/svg>/g, replace: '<SvgIcon name="chevronRight" size="14" />' },
  { match: /<svg[^>]*><polyline points="18 15 12 9 6 15"\/><\/svg>/g, replace: '<SvgIcon name="chevronUp" size="14" />' },

  // === Actions ===
  { match: /<svg[^>]*><line x1="12" y1="5" x2="12" y2="19" \/><line x1="5" y1="12" x2="19" y2="12" \/><\/svg>/g, replace: '<SvgIcon name="plus" size="14" />' },
  { match: /<svg[^>]*><line x1="5" y1="12" x2="19" y2="12"\/><\/svg>/g, replace: '<SvgIcon name="minus" size="14" />' },
  { match: /<svg[^>]*><polyline points="3 6 5 6 21 6"\/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"\/><\/svg>/g, replace: '<SvgIcon name="trash" size="14" />' },
  { match: /<svg[^>]*><polyline points="1 4 1 10 7 10"\/><path d="M3\.51 15a9 9 0 1 0 2\.13-9\.36L1 10"\/><\/svg>/g, replace: '<SvgIcon name="undo" size="14" />' },
  { match: /<svg[^>]*><polyline points="17 21 17 13 7 13 7 21"\/><polyline points="7 3 7 8 15 8"\/><path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"\/><\/svg>/g, replace: '<SvgIcon name="save" size="14" />' },
  { match: /<svg[^>]*><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"\/><polyline points="7 10 12 15 17 10"\/><line x1="12" y1="15" x2="12" y2="3"\/><\/svg>/g, replace: '<SvgIcon name="download" size="14" />' },
  { match: /<svg[^>]*><polygon points="12 2 15\.09 8\.26 22 9\.27 17 14\.14 18\.18 21\.02 12 17\.77 5\.82 21\.02 7 14\.14 2 9\.27 8\.91 8\.26 12 2"\/><\/svg>/g, replace: '<SvgIcon name="star" size="14" />' },
  { match: /<svg[^>]*><polygon points="13 2 16 9 23 10 18 14\.7 19\.2 22 13 18\.8 6\.8 22 8 14\.7 3 10 10 9 13 2"\/><\/svg>/g, replace: '<SvgIcon name="sparkles" size="14" />' },
  { match: /<svg[^>]*><circle cx="12" cy="12" r="6" fill="currentColor"\/><\/svg>/g, replace: '<SvgIcon name="dot" size="14" />' },
  { match: /<svg[^>]*><circle cx="12" cy="12" r="10"\/><line x1="4\.93" y1="4\.93" x2="19\.07" y2="19\.07"\/><\/svg>/g, replace: '<SvgIcon name="ban" size="14" />' },

  // === Arrow down (for "back to bottom" etc) ===
  { match: /<svg[^>]*><line x1="12" y1="5" x2="12" y2="19"\/><polyline points="19 12 12 19 5 12"\/><\/svg>/g, replace: '<SvgIcon name="arrowDown" size="14" />' },
  { match: /<svg[^>]*><line x1="12" y1="19" x2="12" y2="5"\/><polyline points="5 12 12 5 19 12"\/><\/svg>/g, replace: '<SvgIcon name="arrowUp" size="14" />' },

  // === Objects ===
  { match: /<svg[^>]*><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"\/><\/svg>/g, replace: '<SvgIcon name="folder" size="14" />' },
  { match: /<svg[^>]*><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"\/><polyline points="14 2 14 8 20 8"\/><line x1="16" y1="13" x2="8" y2="13"\/><line x1="16" y1="17" x2="8" y2="17"\/><\/svg>/g, replace: '<SvgIcon name="file" size="14" />' },
  { match: /<svg[^>]*><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"\/><path d="M18\.5 2\.5a2\.121 2\.121 0 0 1 3 3L12 15l-4 1 1-4 9\.5-9\.5z"\/><\/svg>/g, replace: '<SvgIcon name="pencil" size="14" />' },
  { match: /<svg[^>]*><path d="M22 22 2 22 2 2"\/><path d="M22 6 22 2 18 2"\/><\/svg>/g, replace: '<SvgIcon name="externalLink" size="14" />' },

  // === Alert ===
  { match: /<svg[^>]*><path d="M10\.29 3\.86L1\.82 18a2 2 0 0 0 1\.71 3h16\.94a2 2 0 0 0 1\.71-3L13\.71 3\.86a2 2 0 0 0-3\.42 0z"\/><line x1="12" y1="9" x2="12" y2="13"\/><line x1="12" y1="17" x2="12\.01" y2="17"\/><\/svg>/g, replace: '<SvgIcon name="alertTriangle" size="14" />' },

  // === Lock / unlock ===
  { match: /<svg[^>]*><rect x="3" y="11" width="18" height="11" rx="2" ry="2"\/><path d="M7 11V7a5 5 0 0 1 10 0v4"\/><\/svg>/g, replace: '<SvgIcon name="lock" size="14" />' },

  // === Map pin ===
  { match: /<svg[^>]*><path d="M21 10c0 7-9 13-9 13s-9-6-9-13a9 9 0 0 1 18 0z"\/><circle cx="12" cy="10" r="3"\/><\/svg>/g, replace: '<SvgIcon name="mapPin" size="14" />' },

  // === Send / antenna ===
  { match: /<svg[^>]*><path d="M22 2 11 13"\/><path d="M22 2l-7 20-4-9-9-4z"\/><\/svg>/g, replace: '<SvgIcon name="send" size="14" />' },

  // === Link ===
  { match: /<svg[^>]*><path d="M10 13a5 5 0 0 0 7\.54\.54l3-3a5 5 0 0 0-7\.07-7\.07l-1\.72 1\.71"\/><path d="M14 11a5 5 0 0 0-7\.54-\.54l-3 3a5 5 0 0 0 7\.07 7\.07l1\.71-1\.71"\/><\/svg>/g, replace: '<SvgIcon name="link" size="14" />' },

  // === Server ===
  { match: /<svg[^>]*><rect x="2" y="2" width="20" height="8" rx="2" ry="2"\/><rect x="2" y="14" width="20" height="8" rx="2" ry="2"\/><line x1="6" y1="6" x2="6\.01" y2="6"\/><line x1="6" y1="18" x2="6\.01" y2="18"\/><\/svg>/g, replace: '<SvgIcon name="server" size="14" />' },
];

let changed = false;

// Try to add import if not present
const importLine = "import SvgIcon from '@/components/ui/SvgIcon.vue'";
if (!content.includes(importLine)) {
  // Find the last import statement in script setup
  const scriptMatch = content.match(/<script[^>]*>[\s\S]*?<\/script>/);
  if (scriptMatch) {
    const script = scriptMatch[0];
    const lastImport = script.lastIndexOf('from');
    if (lastImport !== -1) {
      const afterLastImport = script.indexOf('\n', lastImport);
      const insertPos = scriptMatch.index + afterLastImport + 1;
      content = content.slice(0, insertPos) + '\n' + importLine + content.slice(insertPos);
      changed = true;
    }
  }
}

// Apply pattern replacements
for (const p of patterns) {
  const match = content.match(p.match);
  if (match) {
    content = content.replace(p.match, p.replace);
    changed = true;
  }
}

if (changed) {
  fs.writeFileSync(filePath, content, 'utf-8');
  console.log(`✅ ${filePath}: updated`);
} else {
  console.log(`⏭️ ${filePath}: no changes`);
}

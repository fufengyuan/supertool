const fs = require('fs');
const path = require('path');

// Match based on actual SVG content from the template files
const iconMappings = [
  // File icon (most common - clipboard/copy icon in devtools)
  { check: (s) => s.includes('M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z') && s.includes('polyline points="14 2 14 8 20 8"') && s.includes('line x1="16" y1="13" x2="8" y2="13"') && s.includes('line x1="16" y1="17" x2="8" y2="17"'), name: 'file' },
  { check: (s) => s.includes('M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z') && s.includes('polyline points="14 2 14 8 20 8"'), name: 'file' },

  // Pencil (edit)
  { check: (s) => s.includes('M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7') && s.includes('M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z'), name: 'pencil' },

  // Folder
  { check: (s) => s.includes('M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z'), name: 'folder' },

  // FilePlus (new query)
  { check: (s) => s.includes('M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z') && s.includes('polyline points="14 2 14 8 20 8"') && s.includes('line x1="12" y1="11" x2="12" y2="17"') && s.includes('line x1="9" y1="14" x2="15" y2="14"'), name: 'filePlus' },

  // Search
  { check: (s) => s.includes('cx="11" cy="11" r="8"') && s.includes('M21 21l-4.35-4.35'), name: 'search' },
  { check: (s) => s.includes('cx="11" cy="11" r="8"') && s.includes('line x1="21" y1="21" x2="16.65" y2="16.65"'), name: 'search' },

  // X/close
  { check: (s) => s.includes('M18 6 6 18') && s.includes('m6 6 12 12'), name: 'x' },
  { check: (s) => s.includes('path d="M18 6 6 18"') && s.includes('path d="m6 6 12 12"'), name: 'x' },

  // Plus
  { check: (s) => s.includes('line x1="12" y1="5" x2="12" y2="19"') && s.includes('line x1="5" y1="12" x2="19" y2="12"'), name: 'plus' },
  { check: (s) => s.includes('M12 5 12 19') && s.includes('M5 12 19 12'), name: 'plus' },

  // Lock
  { check: (s) => s.includes('rect x="3" y="11" width="18" height="11" rx="2" ry="2"') && s.includes('M7 11V7a5 5 0 0 1 10 0v4'), name: 'lock' },

  // Key
  { check: (s) => s.includes('M21 2l-2 2m-7.61 7.61') && s.includes('L15.5 7.5m0 0l3 3L22 7l-3-3'), name: 'key' },

  // Trash
  { check: (s) => s.includes('polyline points="3 6 5 6 21 6"') && s.includes('M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2'), name: 'trash' },

  // Database
  { check: (s) => s.includes('ellipse cx="12" cy="5" rx="9" ry="3"') && s.includes('M21 12c0 1.66-4 3-9 3s-9-1.34-9-3'), name: 'database' },

  // Archive
  { check: (s) => s.includes('rect x="2" y="3" width="20" height="18" rx="2" ry="2"') && s.includes('line x1="2" y1="9" x2="22" y2="9"'), name: 'archive' },

  // Tool/wrench
  { check: (s) => s.includes('M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6'), name: 'tool' },

  // Package (data sync)
  { check: (s) => (s.includes('line x1="16.5" y1="9.4" x2="7.5" y2="4.21"') || s.includes('M16.5 9.4 7.5 4.21')) && s.includes('M21 16V8a2 2 0 0 0-1-1.73l-7-4') && s.includes('M3.27 6.96 12 12.01 20.73 6.96'), name: 'package' },

  // Refresh
  { check: (s) => s.includes('polyline points="23 4 23 10 17 10"') && s.includes('M3.51 9a9 9 0 0 1 14.85-3.36L23 10'), name: 'refresh' },
  { check: (s) => s.includes('polyline points="23 4 23 10 17 10"') && s.includes('M20.49 15a9 9 0 1 1-2.12-9.36L23 10'), name: 'refresh' },
  { check: (s) => s.includes('polyline points="1 4 1 10 7 10"') && s.includes('polyline points="23 20 23 14 17 14"') && s.includes('M20.49 9A9 9 0 0 0 5.64 5.64L1 10') && s.includes('m22 4l-4.64 4.36'), name: 'refresh' },

  // Undo
  { check: (s) => s.includes('polyline points="1 4 1 10 7 10"') && s.includes('M3.51 15a9 9 0 1 0 2.12-9.36L1 10'), name: 'undo' },

  // Save
  { check: (s) => s.includes('M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11') && (s.includes('M17 21 17 13 7 13 7 21') || s.includes('polyline points="17 21 17 13 7 13 7 21"')) && (s.includes('M7 3 7 8 15 8') || s.includes('polyline points="7 3 7 8 15 8"')), name: 'save' },

  // Bar chart
  { check: (s) => s.includes('line x1="18" y1="20" x2="18" y2="10"') && s.includes('line x1="12" y1="20" x2="12" y2="4"') && s.includes('line x1="6" y1="20" x2="6" y2="14"'), name: 'barChart' },

  // Clock
  { check: (s) => s.includes('cx="12" cy="12" r="10"') && s.includes('polyline points="12 6 12 12 16 14"'), name: 'clock' },

  // Calendar
  { check: (s) => s.includes('rect x="3" y="4" width="18" height="18" rx="2" ry="2"') && s.includes('line x1="16" y1="2" x2="16" y2="6"') && s.includes('line x1="8" y1="2" x2="8" y2="6"') && s.includes('line x1="3" y1="10" x2="21" y2="10"'), name: 'calendar' },

  // Filter
  { check: (s) => s.includes('polygon points="22 3 2 3 10 12.46 10 19 14 21 14 12.46 22 3"'), name: 'filter' },

  // Globe
  { check: (s) => s.includes('line x1="2" y1="12" x2="22" y2="12"') && s.includes('M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10'), name: 'globe' },

  // Link
  { check: (s) => s.includes('M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71') && s.includes('M14 11a5 5 0 0 0-7.54-.54l-3 3'), name: 'link' },

  // Plug
  { check: (s) => s.includes('M12 2v8') && s.includes('M4.93 10.93a8 8 0 1 1 14.14 0'), name: 'plug' },

  // Send/Play triangle
  { check: (s) => s.includes('polygon points="5 3 19 12 5 21 5 3"'), name: 'send' },

  // AlertCircle (error/exclamation in circle)
  { check: (s) => { return s.includes('cx="12" cy="12" r="10"') && (s.includes('line x1="12" y1="8" x2="12" y2="12"') || s.includes('M12 8 12 12')) && (s.includes('line x1="12" y1="16" x2="12.01" y2="16"') || s.includes('M12 16 12.01 16')); }, name: 'alertCircle' },

  // Dot (status indicator)
  { check: (s) => s.includes('cx="12" cy="12" r="6" fill="currentColor"') || s.includes('circle cx="12" cy="12" r="6" fill="currentColor"'), name: 'dot' },

  // Rocket
  { check: (s) => s.includes('M4.5 16.5c-1.5 1.26-2 5-2 5s3.74-.5 5-2') && s.includes('M12 15l-3-3a22 22 0 0 1 2-3.95'), name: 'rocket' },

  // Clipboard
  { check: (s) => s.includes('M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2') && s.includes('rect x="8" y="2" width="8" height="4" rx="1" ry="1"'), name: 'clipboard' },

  // Download
  { check: (s) => s.includes('M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4') && s.includes('M7 10 12 15 17 10') && s.includes('M12 15 12 3'), name: 'download' },

  // Send (paper plane)
  { check: (s) => s.includes('M22 2 11 13') && s.includes('M22 2l-7 20-4-9-9-4z'), name: 'send' },

  // Code
  { check: (s) => s.includes('polyline points="16 18 22 12 16 6"') && s.includes('polyline points="8 6 2 12 8 18"'), name: 'code' },

  // Menu/format (align left 4 lines)
  { check: (s) => s.includes('line x1="21" y1="10" x2="3" y2="10"') && s.includes('line x1="21" y1="6" x2="3" y2="6"') && s.includes('line x1="21" y1="14" x2="3" y2="14"') && s.includes('line x1="21" y1="18" x2="3" y2="18"'), name: 'menu' },

  // ChevronDown
  { check: (s) => s.includes('polyline points="6 9 12 15 18 9"'), name: 'chevronDown' },

  // ArrowDown
  { check: (s) => s.includes('line x1="12" y1="5" x2="12" y2="19"') && s.includes('polyline points="19 12 12 19 5 12"'), name: 'arrowDown' },

  // Settings
  { check: (s) => s.includes('cx="12" cy="12" r="3"') && s.includes('M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06') && s.includes('M4.68 15a1.65 1.65 0 0 0-1.51-1H3'), name: 'settings' },

  // Eye
  { check: (s) => s.includes('M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z') && s.includes('cx="12" cy="12" r="3"'), name: 'eye' },

  // Checkmark
  { check: (s) => s.includes('polyline points="20 6 9 17 4 12"'), name: 'check' },

  // CheckCircle
  { check: (s) => s.includes('M22 11.08V12a10 10 0 1 1-5.93-9.14') && (s.includes('M22 4 12 14.01 9 11.01') || s.includes('polyline points="22 4 12 14.01 9 11.01"')), name: 'checkCircle' },

  // Table/Grid 
  { check: (s) => s.includes('rect x="3" y="3" width="18" height="18" rx="2" ry="2"') && s.includes('line x1="3" y1="9" x2="21" y2="9"'), name: 'barChart' },

  // Loading spinner
  { check: (s) => s.includes('class="animate-spin"') && s.includes('M21 12a9 9'), name: 'refresh' },

  // Tag
  { check: (s) => s.includes('M20.59 13.41l-7.17 7.17a2 2 0 0 1-2.83 0L2 12V2h10l8.59 8.59a2 2 0 0 1 0 2.82z') && s.includes('line x1="7" y1="7" x2="7.01" y2="7"'), name: 'tag' },

  // Terminal 
  { check: (s) => s.includes('polyline points="4 17 10 11 4 5"') || s.includes('M4 17 10 11 4 5'), name: 'terminal' },
  
  // Upload
  { check: (s) => s.includes('M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4') && s.includes('M17 8 12 3 7 8') && s.includes('M12 3 12 15'), name: 'upload' },

  // Inbox
  { check: (s) => s.includes('polyline points="22 12 16 12 14 15 10 15 8 12 2 12"') && s.includes('M5.45 5.11L2 12v6a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-6l-3.45-6.89'), name: 'inbox' },

  // ExternalLink
  { check: (s) => s.includes('M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6') && s.includes('M15 3 21 3 21 9') && s.includes('M10 14 21 3'), name: 'externalLink' },

  // ChevronRight
  { check: (s) => s.includes('polyline points="9 18 15 12 9 6"') || s.includes('M9 18 15 12 9 6'), name: 'chevronRight' },

  // GitBranch
  { check: (s) => s.includes('line x1="6" y1="3" x2="6" y2="15"') && s.includes('circle cx="18" cy="6" r="3"') && s.includes('circle cx="6" cy="18" r="3"') && s.includes('M18 9a9 9 0 0 1-9 9'), name: 'gitBranch' },

  // Wifi (perimeter signal)
  { check: (s) => s.includes('cx="12" cy="12" r="2"') && s.includes('M16.24 7.76a6 6 0 0 1 0 8.49') && s.includes('m11.31-2.82a10 10 0 0 1 0 14.14') && s.includes('m-14.14 0a10 10 0 0 1 0-14.14'), name: 'wifi' },

  // MapPin
  { check: (s) => s.includes('M21 10c0 7-9 13-9 13s-9-6-9-13a9 9 0 0 1 18 0z') && s.includes('circle cx="12" cy="10" r="3"') || s.includes('cx="12" cy="10" r="3"') && s.includes('M21 10c0 7-9 13-9 13s-9-6'), name: 'mapPin' },

  // Monitor/Screen
  { check: (s) => s.includes('rect x="4" y="2" width="16" height="20" rx="2"') && s.includes('line x1="9" y1="6" x2="9" y2="10"') && s.includes('line x1="15" y1="6" x2="15" y2="10"') && s.includes('line x1="9" y1="14" x2="9" y2="18"') && s.includes('line x1="15" y1="14" x2="15" y2="18"'), name: 'monitor' },

  // Flag/Server (IpTool)
  { check: (s) => s.includes('M4 15s1-1 4-1 5 2 8 2 4-1 4-1V3s-1 1-4 1-5-2-8-2-4 1-4 1z') && s.includes('line x1="4" y1="22" x2="4" y2="15"'), name: 'server' },
];

function findIconName(svgContent) {
  for (const mapping of iconMappings) {
    try {
      if (mapping.check(svgContent)) {
        return mapping.name;
      }
    } catch (e) {
      // ignore
    }
  }
  return null;
}

function processFile(filePath) {
  let content = fs.readFileSync(filePath, 'utf-8');
  let modified = false;

  // 1. Add import if not present
  if (!content.includes("import SvgIcon from '@/components/ui/SvgIcon.vue'")) {
    content = content.replace(
      /(<script setup[^>]*>)/,
      `$1\nimport SvgIcon from '@/components/ui/SvgIcon.vue'`
    );
    modified = true;
  }

  // 2. Find and replace inline SVGs in template only
  const templateMatch = content.match(/<template>([\s\S]*?)<\/template>/);
  if (!templateMatch) return false;

  let templateContent = templateMatch[1];
  const originalTemplate = templateContent;

  const svgRegex = /<svg[\s\S]*?<\/svg>/g;
  let replacements = [];

  while ((match = svgRegex.exec(templateContent)) !== null) {
    const fullSvg = match[0];
    const iconName = findIconName(fullSvg);
    if (iconName) {
      let size = 16;
      const sizeMatch = fullSvg.match(/width="(\d+)"/);
      if (sizeMatch) size = parseInt(sizeMatch[1]);
      
      let classAttr = '';
      const classMatch = fullSvg.match(/class="([^"]*)"/);
      if (classMatch) {
        classAttr = classMatch[1]
          .replace(/\bshrink-0\b/g, '')
          .replace(/\binline-block\b/g, '')
          .replace(/\bpointer-events-none\b/g, '')
          .replace(/\bz-10\b/g, '')
          .replace(/\s+/g, ' ').trim();
      }

      let replacement = `<SvgIcon name="${iconName}" size="${size}"`;
      if (classAttr) {
        replacement += ` class="${classAttr}"`;
      }
      replacement += ' />';

      replacements.push({ old: fullSvg, new: replacement });
    } else {
      console.log(`  [UNMAPPED] ${path.basename(filePath)}: ${fullSvg.replace(/\s+/g, ' ').substring(0, 120)}...`);
    }
  }

  for (const r of replacements.reverse()) {
    const idx = templateContent.indexOf(r.old);
    if (idx !== -1) {
      templateContent = templateContent.substring(0, idx) + r.new + templateContent.substring(idx + r.old.length);
    }
  }

  if (originalTemplate !== templateContent) {
    content = content.replace(originalTemplate, templateContent);
    modified = true;
  }

  if (modified) {
    fs.writeFileSync(filePath, content, 'utf-8');
    return true;
  }
  return false;
}

// Files to process
const dbDir = 'src/views/db';
const devtoolsDir = 'src/views/devtools';

const dbFiles = [
  'ConnectionForm.vue', 'ConnectionTree.vue', 'DBBackup.vue', 'DBManager.vue',
  'DataSync.vue', 'FilterBar.vue', 'RedisFolderNode.vue', 'RedisQueueManager.vue',
  'SqlEditor.vue', 'StructureSync.vue', 'TableStructure.vue'
];

const devtoolsFiles = [
  'DevTools.vue',
  'tools/ApiDebugger.vue', 'tools/ArmHexTool.vue', 'tools/AsciiTool.vue',
  'tools/BarcodeTool.vue', 'tools/Base64Tool.vue', 'tools/BaseConvTool.vue',
  'tools/CodeFormatTool.vue', 'tools/ComplementTool.vue', 'tools/CrontabTool.vue',
  'tools/CryptoTool.vue', 'tools/DiffTool.vue', 'tools/EncryptTool.vue',
  'tools/HexTool.vue', 'tools/HtmlEntityTool.vue', 'tools/IpCalcTool.vue',
  'tools/IpTool.vue', 'tools/JsonTool.vue', 'tools/JwtTool.vue',
  'tools/PinyinTool.vue', 'tools/QrCodeTool.vue', 'tools/RandomTool.vue',
  'tools/RegexTool.vue', 'tools/SerialTool.vue', 'tools/TextTool.vue',
  'tools/TimeCalcTool.vue', 'tools/TimeTool.vue', 'tools/UnicodeTool.vue',
  'tools/UnitTool.vue', 'tools/UrlTool.vue', 'tools/UuidTool.vue',
  'tools/VariableTool.vue', 'tools/WsTool.vue'
];

let processed = 0;
let failed = 0;
let unmappedCount = 0;

for (const f of dbFiles) {
  const fp = path.join(dbDir, f);
  if (fs.existsSync(fp)) {
    try {
      if (processFile(fp)) {
        console.log(`✓ ${f}`);
        processed++;
      } else {
        console.log(`- ${f} (no changes)`);
      }
    } catch (e) {
      console.log(`✗ ${f}: ${e.message}`);
      failed++;
    }
  } else {
    console.log(`? ${f} (not found)`);
  }
}

for (const f of devtoolsFiles) {
  const fp = path.join(devtoolsDir, f);
  if (fs.existsSync(fp)) {
    try {
      if (processFile(fp)) {
        console.log(`✓ ${f}`);
        processed++;
      } else {
        console.log(`- ${f} (no changes)`);
      }
    } catch (e) {
      console.log(`✗ ${f}: ${e.message}`);
      failed++;
    }
  } else {
    console.log(`? ${f} (not found)`);
  }
}

console.log(`\nDone. ${processed} files modified, ${failed} failed.`);

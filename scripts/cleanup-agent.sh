#!/bin/bash
set -e
cd /Users/fufengyuan/WebstormProjects/supertool

echo "=== Cleaning up agent references ==="

# 1. Remove hermes/claw imports from tauri-api.ts
echo "1. Cleaning tauri-api.ts imports..."
sed -i '' 's/ HermesConfigInfo, ConfigExportResult, ConfigImportResult,//' src/utils/tauri-api.ts

# 2. Remove hermes/claw interface declarations (lines ~2000-2085)
echo "2. Removing hermes/claw interface declarations..."
sed -i '' '/\/\/ Hermes Tools/,/^\/\/.*$/{
  /^\/\/.*$/!d
}' src/utils/tauri-api.ts

# Actually let me just remove specific comment blocks and their following function declarations
sed -i '' '/^    \/\/ ============ Hermes Config Export\/Import ============$/,/^    \/\/ ============ .* ============$/{
  /^    \/\/ ============ .* ============$/!d
}' src/utils/tauri-api.ts

echo "Done"

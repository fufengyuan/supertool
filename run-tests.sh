#!/bin/bash
cd /Users/duormi/workspace/supertool && npx vitest run --reporter verbose src/views/agent/chat/ 2>&1
echo "EXIT_CODE=$?"

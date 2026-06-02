#!/bin/bash
cd /path/to/supertool && npx vitest run --reporter verbose src/views/agent/chat/ 2>&1
echo "EXIT_CODE=$?"

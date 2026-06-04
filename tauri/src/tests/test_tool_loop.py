#!/usr/bin/env python3
"""Quick smoke test: exercises the Claw tool loop via the Tauri IPC path.

This script:
1. Sets up environment from ~/.claw/config.json
2. Calls send_turn directly through a Rust integration test
3. Verifies tools are actually used

Run: cd supertool && cargo test -p supertool -- claw_chat::test_tool_loop --nocapture
"""

# This is a placeholder — the actual test is written in Rust below.
print("See tauri/src/tests/claw_chat.rs for the integration test")

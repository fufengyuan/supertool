#!/bin/bash
# Cargo rustc wrapper — use sccache if available, pass through otherwise
if command -v sccache &>/dev/null; then
    exec sccache "$@"
fi
exec "$@"

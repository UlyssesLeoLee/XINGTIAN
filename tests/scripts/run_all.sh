#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../.."
mkdir -p tests/results
if ! command -v cargo >/dev/null 2>&1; then echo 'ERROR: Rust cargo not installed; Rust tests NOT EXECUTED'; exit 3; fi
cargo fmt --all
cargo test --workspace --all-targets -- --nocapture
cargo clippy --workspace --all-targets -- -D warnings
python3 tests/scripts/validate_traceability.py
if [[ "${RUN_BROWSER_ORACLE:-0}" == "1" ]]; then
  (cd tests/playwright && npm install && npx playwright install chromium && npm test)
else
  echo 'Chromium oracle skipped. Set RUN_BROWSER_ORACLE=1 to run separately (NOT native compatibility).'
fi

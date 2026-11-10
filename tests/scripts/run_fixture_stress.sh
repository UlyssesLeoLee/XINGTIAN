#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../.."
if ! command -v cargo >/dev/null 2>&1; then echo 'BLOCKED: cargo unavailable, fixture volume tests not executed'; exit 3; fi
cargo test -p aiweb-core --test stress_fixture -- --ignored --nocapture
echo 'These were FIXTURE checks ONLY, NOT 8/16/32 x 60min real browser ST.'

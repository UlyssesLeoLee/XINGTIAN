#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../.."
echo 'BLOCKED: Native sandbox renderer, network broker, and persistence are not implemented. No meaningful real SEC test can execute.'
python3 tests/scripts/release_gate.py

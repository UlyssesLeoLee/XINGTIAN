$ErrorActionPreference = 'Stop'
Set-Location (Join-Path $PSScriptRoot '..\..')
New-Item -ItemType Directory -Force 'tests/results' | Out-Null
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) { throw 'Rust cargo not installed: Rust tests NOT EXECUTED' }
cargo fmt --all; if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
cargo test --workspace --all-targets -- --nocapture; if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
cargo clippy --workspace --all-targets -- -D warnings; if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
python tests/scripts/validate_traceability.py; if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
Write-Host 'Chromium oracle is a separate test; run in tests/playwright after npm install + npx playwright install chromium.'

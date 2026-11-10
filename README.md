# AIWEB — Agent-Native Parallel Web Engine / controlled Rust POC v0.1

**This is a source-code and test-design starter, NOT a production browser engine.**

Chinese guide: [README.zh-CN.md](README.zh-CN.md). Full test design: `docs/design/Test_Design_UT_IT_ST_v1.0.md` and `docs/design/Test_Traceability_v1.0.xlsx`.

## Run

```bash
cargo test --workspace --all-targets
cargo run -p aiwebctl -- capabilities
cargo run -p aiwebctl -- demo
python3 tests/scripts/validate_traceability.py
bash tests/scripts/run_all.sh
```

The Rust core uses only the Rust standard library. On Windows run `tests/scripts/run_all.ps1`. To run the separate Chromium *oracle* suite, install dependencies in `tests/playwright/` and run `npm test` after installing the Chromium browser.

## Code

- `graph`: DAG rejection and topological order
- `scheduler`: bounded per-tenant round-robin and budget tokens (not OS resource enforcement)
- `state`, `action`: version-vector strict CAS, fixture locator and single-dispatch guard
- `effects`: in-memory idempotency/effect journal, uncertainty without auto replay
- `security`: simulated authenticated-channel / lease / permit checks, never opens sockets
- `events`, `time`: per-stream gap and boot-scoped monotonic comparisons
- `capabilities`: authoritative unsupported feature list

## Not implemented

HTML5/DOM real parser, JavaScript VM, Web Agent event loop, CSS/layout, paint, native Playwright driver, network broker, process sandbox, durable effect storage, real browser navigation, WPT compatibility, and real Chrome-comparative benchmarks. **Do not use this prototype to browse untrusted websites or execute untrusted scripts.**

The source is explicitly limited to fixture semantics; neither `cargo test` success nor the Chromium-only Playwright oracle tests count as native browser correctness or security certification.

## Governance

Requirements v1.1, basic design v1.1, and detailed design v1.1 are bundled as upstream copies in `docs/upstream/`. Primary reference `AIPWE-DD-001`; 157 upstream IDs and 81 DD IDs are preserved, and all 34+22 planned tests remain `未执行` in the report until verified evidence is supplied. `tests/scripts/release_gate.py` fails closed by default.

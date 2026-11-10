#!/usr/bin/env python3
"""Fail-closed release gate; requires evidence written by actual CI, never infers pass from docs."""
import pathlib,json,sys
root=pathlib.Path(__file__).resolve().parents[2]
file=root/'tests/results/verified_execution.json'
if not file.exists():
 print('BLOCKED: no verified_execution.json from actual native Linux sandbox test run')
 sys.exit(4)
data=json.loads(file.read_text())
required={f'RED-DD-{i:03d}' for i in range(1,11)}|{f'RED-BD-{i:03d}' for i in range(19,27)}
missing=sorted(t for t in required if data.get('cases',{}).get(t,{}).get('state')!='passed')
if missing or not data.get('code_sha') or not data.get('linux_sandbox_digest') or not data.get('signed_adr_approvals'):
 print('BLOCKED: missing passed P0, immutable evidence or ADR approval:',','.join(missing))
 sys.exit(4)
print('P0 evidence metadata complete; human security review and performance gate remain independent.')

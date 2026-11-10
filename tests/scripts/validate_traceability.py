#!/usr/bin/env python3
"""Offline consistency gate for design registry. Does NOT execute Rust or claim browser compliance."""
import json, pathlib, sys, re
root=pathlib.Path(__file__).resolve().parents[2]
r=json.loads((root/'tests/results/test_registry.json').read_text(encoding='utf8'))
assert len(r['requirements'])==157 and len(r['design_controls'])==81 and len(r['test_cases'])==56
assert len(set(x['id'] for x in r['requirements']))==157
assert len(set(x['id'] for x in r['design_controls']))==81
assert len(set(x['id'] for x in r['test_cases']))==56
control_ids={x['id'] for x in r['design_controls']}
assert all(set(x['design_ids'])<=control_ids and x['design_ids'] for x in r['requirements'])
assert all(x['status']=='未执行' for x in r['test_cases']), 'Plans must not silently become PASS'
assert sum(x['id'].startswith('RED-DD') for x in r['test_cases'])==22
assert sum(x['id'].endswith('BD-'+x['id'][-3:]) for x in r['test_cases'])==34
assert all((root/x['script']).is_file() for x in r['test_cases'] if x['script'])
for x in r['test_cases']:
 if x['id'].startswith('RED-DD') and x['script']:
  source=(root/x['script']).read_text()
  assert re.search(r'fn\s+red_dd_'+x['id'][-3:]+r'_',source),x['id']
print('OK: 157 requirements, 81 controls, 34 inherited + 22 red cases, all unexecuted; paths resolved')

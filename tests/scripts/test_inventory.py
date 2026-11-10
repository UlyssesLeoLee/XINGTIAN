import unittest, pathlib, json, sys
root=pathlib.Path(__file__).resolve().parents[2]
r=json.loads((root/'tests/results/test_registry.json').read_text())
class StaticInventory(unittest.TestCase):
 def test_requirements(self): self.assertEqual(len(r['requirements']),157)
 def test_controls(self): self.assertEqual(len(r['design_controls']),81)
 def test_plans(self): self.assertEqual(len(r['test_cases']),56)
 def test_red_count(self): self.assertEqual(sum(t['id'].startswith('RED-DD') for t in r['test_cases']),22)
 def test_not_falsely_passed(self): self.assertTrue(all(t['status']=='未执行' for t in r['test_cases']))
 def test_all_requirements_mapped(self): self.assertTrue(all(t['design_ids'] for t in r['requirements']))
 def test_no_native_claim(self): self.assertFalse(r['declared_live_native_browser_support'])
if __name__=='__main__':unittest.main()

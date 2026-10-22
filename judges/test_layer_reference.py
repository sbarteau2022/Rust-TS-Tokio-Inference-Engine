"""Layer/state layout checks without a model dependency."""
import unittest
import json
import tempfile
from pathlib import Path
import layer_reference as layer


class LayoutTests(unittest.TestCase):
    def test_bounds_and_contiguous_layout(self):
        for position in [1, 8, 31, 512]:
            layout = layer.state_layout(position)
            self.assertEqual(len(layout), 80)
            self.assertEqual(layout[0]['start'], 0)
            self.assertTrue(all(a['end'] == b['start'] for a, b in zip(layout, layout[1:])))
            self.assertEqual(layout[-1]['end'], 30 * (3 * 8192 + 32 * 128 * 128) + 10 * 2 * 2 * position * 256)
            self.assertEqual(layout[6]['state'], 'key')
            self.assertEqual(layout[6]['shape'], (1, 2, position, 256))
        for position in [0, -1, 513]:
            with self.assertRaises(ValueError):
                layer.state_layout(position)


    def test_incomplete_and_malformed_captures_fail_closed(self):
        inventory = {c['id']: c for c in layer.base.inventory() if c['suite'] == 'domain'}
        rows = []
        for caseid in ['s10', 'r11']:
            ids = [1] * 17
            for offset in range(0, len(ids), 8):
                count = min(8, len(ids) - offset)
                rows.append(dict(case=caseid, prompt=inventory[caseid]['prompt'], prompt_ids=ids,
                    identity=layer.base.IDENTITY, offset=offset, count=count, position=offset+count,
                    trace_values=40*5*count*2048, state_values=layer.state_layout(offset+count)[-1]['end'],
                    logit_control=0., state_control=0.))
        with tempfile.TemporaryDirectory() as d:
            p = Path(d)
            (p/'drain.txt').write_text('leases=0 cases=2\n')
            def write(data):
                (p/'native.jsonl').write_text(''.join(json.dumps(r)+'\n' for r in data))
            write(rows)
            self.assertEqual(len(layer.records(p)), 6)
            for mutation in ['missing', 'extra', 'shape', 'order', 'identity', 'nonfinite']:
                data = json.loads(json.dumps(rows))
                if mutation == 'missing': data.pop()
                if mutation == 'extra': data.append(data[-1])
                if mutation == 'shape': data[0]['state_values'] -= 1
                if mutation == 'order': data[0], data[1] = data[1], data[0]
                if mutation == 'identity': data[0]['identity'] = 'wrong'
                if mutation == 'nonfinite': data[0]['state_control'] = float('nan')
                write(data)
                with self.assertRaises(ValueError):
                    layer.records(p)


if __name__ == '__main__':
    unittest.main()

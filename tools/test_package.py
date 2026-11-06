#!/usr/bin/env python3
"""Validate reproducible, source-only launcher packaging and compatibility pins."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tomllib
import unittest
import zipfile

ROOT = Path(__file__).resolve().parents[1]


class PackageTests(unittest.TestCase):
    def test_reproducible_source_bundle(self):
        def build():
            result = subprocess.check_output([sys.executable, 'tools/package.py'], cwd=ROOT, text=True)
            report = json.loads(result)
            data = Path(report['file']).read_bytes()
            self.assertEqual(report['size'], len(data))
            self.assertEqual(report['sha256'], hashlib.sha256(data).hexdigest())
            return report, data

        first, data = build()
        self.assertEqual((first, data), build())
        compat = json.loads((ROOT / 'compatibility.json').read_text())
        project = tomllib.loads((ROOT / 'Cargo.toml').read_text())['package']
        expected = {'horizon-native.json'} | {
            f'mixins/veinminer/{name}' for name in
            ('command.rs', 'drop.rs', 'guest-command.rs', 'horizon-mixin.toml', 'mining.rs', 'runtime.rs', 'speed.rs', 'tick.rs')
        }
        with zipfile.ZipFile(first['file']) as archive:
            self.assertEqual(set(archive.namelist()), expected)
            self.assertEqual(len(archive.namelist()), len(expected))
            self.assertIsNone(archive.testzip())
            self.assertEqual(json.loads(archive.read('horizon-native.json')), {
                'schema': 1, 'id': 'veinminer', 'version': project['version'], 'game': '26.3',
                'horizon': compat['horizon_revision'], 'mixins': compat['horizon_mixins_revision'],
            })
            for info in archive.infolist():
                self.assertEqual(info.date_time, (2026, 1, 1, 0, 0, 0))
                self.assertEqual(info.external_attr >> 16, 0o100644)
                if info.filename != 'horizon-native.json':
                    self.assertEqual(archive.read(info.filename), (ROOT / info.filename).read_bytes())

    def test_reproducible_enchantment_pack(self):
        result = json.loads(subprocess.check_output([sys.executable, 'tools/package.py'], cwd=ROOT, text=True))
        path = Path(result['enchantment_file'])
        data = path.read_bytes()
        self.assertEqual(result['enchantment_size'], len(data))
        self.assertEqual(result['enchantment_sha256'], hashlib.sha256(data).hexdigest())
        with zipfile.ZipFile(path) as archive:
            expected = {
                'pack.mcmeta',
                'data/veinminer_enchantment/enchantment/veinminer.json',
                'data/minecraft/tags/enchantment/in_enchanting_table.json',
                'data/minecraft/tags/enchantment/tradeable.json',
                'data/minecraft/tags/enchantment/on_random_loot.json',
                'data/minecraft/tags/enchantment/treasure.json',
            }
            self.assertEqual(set(archive.namelist()), expected)
            self.assertIsNone(archive.testzip())
            self.assertEqual(json.loads(archive.read('pack.mcmeta'))['pack']['min_format'], [121, 0])
            enchantment = json.loads(archive.read('data/veinminer_enchantment/enchantment/veinminer.json'))
            self.assertEqual(enchantment['max_level'], 1)
            self.assertEqual(enchantment['slots'], ['mainhand'])


if __name__ == '__main__':
    unittest.main()

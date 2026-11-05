import json
import os
import sys
import unittest

sys.path.insert(0, os.path.dirname(__file__))
import check_zenith_intact as c
from gitutil import commit, git, new_repo, write

NEEDLES = {}
for _f, _n in c.IDENTIFIERS:
    NEEDLES.setdefault(_f, []).append(_n)


class IntactTest(unittest.TestCase):
    def repo(self):
        r = new_repo()
        git(r, "checkout", "-q", "-b", "up")
        write(r, "crates/a.rs", "fn a() {}\n")
        commit(r, "upstream")
        git(r, "checkout", "-q", "-b", "main")
        for f in c.EXTRA_FILES:
            write(r, f, "x\n")
        for f, needles in NEEDLES.items():
            write(r, f, "\n".join(needles) + "\n")
        write(r, "crates/a.rs", "fn a() {} // zenith: one\nfn b() {} // zenith: two\n")
        write(r, "zenith/own.py", "own\n")
        commit(r, "ours")
        self.m = c.build(r, "up")
        return r

    def test_intact_tree_passes_and_manifest_content(self):
        r = self.repo()
        self.assertEqual(self.m["markers"]["crates/a.rs"], 2)
        self.assertIn("zenith/own.py", self.m["own_files"])
        self.assertEqual(c.check(r, self.m), [])
        json.dumps(self.m)

    def test_deleted_own_file_trips(self):
        r = self.repo()
        os.remove(os.path.join(r, "zenith/own.py"))
        self.assertTrue(any("own file is gone" in b for b in c.check(r, self.m)))

    def test_removed_marker_region_trips(self):
        r = self.repo()
        write(r, "crates/a.rs", "fn a() {} // zenith: one\n")
        self.assertTrue(any("markers fell from 2 to 1" in b for b in c.check(r, self.m)))

    def test_missing_identifier_trips(self):
        r = self.repo()
        write(r, "crates/horizon-cosmetics/src/lib.rs", "pub const ON_BY_DEFAULT: bool = true;\n")
        self.assertTrue(any("ON_BY_DEFAULT" in b for b in c.check(r, self.m)))


if __name__ == "__main__":
    unittest.main()

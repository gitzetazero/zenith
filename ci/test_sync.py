import json
import os
import sys
import tempfile
import unittest

sys.path.insert(0, os.path.dirname(__file__))
import sync
from gitutil import commit, git, write
from test_port_upstream import fork, upstream_commit

CORE = "crates__horizon-core__src__lib.rs"


def with_origin(z):
    """A bare 'GitHub' that Zenith's main was pushed to."""
    o = tempfile.mkdtemp(prefix="origin")
    git(o, "init", "-q", "--bare", "-b", "main")
    git(z, "remote", "add", "origin", o)
    git(z, "push", "-q", "origin", "main")
    return o


def tip(repo, ref="main"):
    return git(repo, "rev-parse", ref).stdout.strip()


class SyncTest(unittest.TestCase):
    def test_all_green_lands_on_main_with_a_normal_push_and_records_the_sha(self):
        u, z, _ = fork()
        o = with_origin(z)
        up = upstream_commit(u, **{CORE: "pub fn core() { new(); }\n"})
        git(z, "fetch", "-q", "upstream")
        rc = sync.run_all(z, "upstream/main", remote=o, no_cargo=True, dry=True, gate=lambda *a: 0)
        self.assertEqual(rc, 0)
        self.assertEqual(tip(o), tip(z, "HEAD"))
        self.assertEqual(git(o, "show", "main:ci/upstream-synced.txt").stdout.split()[0], up)
        self.assertIn("new();", git(o, "show", "main:crates/zenith-core/src/lib.rs").stdout)
        self.assertEqual(git(o, "branch", "--list", "sync/*").stdout.strip(), "")  # nothing was blocked

    def test_failed_gate_never_touches_main_and_reports_a_branch(self):
        u, z, _ = fork()
        o = with_origin(z)
        before = tip(o)
        upstream_commit(u, **{CORE: "pub fn core() { new(); }\n"})
        git(z, "fetch", "-q", "upstream")
        rc = sync.run_all(z, "upstream/main", remote=o, no_cargo=True, dry=True, gate=lambda *a: 1)
        self.assertEqual(rc, 1)
        self.assertEqual(tip(o), before)
        branches = git(o, "branch", "--list", "sync/*").stdout.split()
        self.assertEqual(len(branches), 1)
        self.assertIn("new();", git(o, "show", f"{branches[0]}:crates/zenith-core/src/lib.rs").stdout)

    def test_conflict_never_touches_main_and_reports_the_partial_port(self):
        u, z, _ = fork()
        write(z, "crates/zenith-core/src/lib.rs", "pub fn core() { /* hook */ }\n")
        commit(z, "ours")
        o = with_origin(z)
        before = tip(o)
        upstream_commit(u, **{CORE: "pub fn core() { new(); }\n"})
        git(z, "fetch", "-q", "upstream")
        rc = sync.run_all(z, "upstream/main", remote=o, no_cargo=True, dry=True, gate=lambda *a: self.fail("no gate on a conflict"))
        self.assertEqual(rc, 1)
        self.assertEqual(tip(o), before)
        self.assertEqual(len(git(o, "branch", "--list", "sync/*").stdout.split()), 1)

    def test_land_refuses_when_main_moved_and_never_forces(self):
        u, z, _ = fork()
        o = with_origin(z)
        upstream_commit(u, **{CORE: "pub fn core() { new(); }\n"})
        git(z, "fetch", "-q", "upstream")
        import port_upstream
        self.assertEqual(port_upstream.port(z, "upstream/main", no_cargo=True), 0)
        other = tempfile.mkdtemp(prefix="other")  # somebody else pushes to main meanwhile
        git(other, "clone", "-q", o, other + "/c")
        write(other + "/c", "elsewhere.txt", "x\n")
        commit(other + "/c", "elsewhere")
        git(other + "/c", "push", "-q", "origin", "main")
        moved = tip(o)
        self.assertEqual(sync.land(z, o), sync.MOVED)
        self.assertEqual(tip(o), moved)

    def test_land_refuses_a_dirty_tree(self):
        u, z, _ = fork()
        o = with_origin(z)
        write(z, "crates/zenith-core/src/lib.rs", "dirty\n")
        self.assertEqual(sync.land(z, o), sync.BLOCKED)

    def test_up_to_date_does_nothing(self):
        u, z, _ = fork()
        o = with_origin(z)
        rc = sync.run_all(z, "upstream/main", remote=o, no_cargo=True, dry=True, gate=lambda *a: self.fail("nothing to check"))
        self.assertEqual(rc, 0)

    def test_the_gate_includes_a_workspace_build_with_tests(self):
        names = {n: c for n, c in sync.steps(False)}
        self.assertEqual(names["cargo check workspace"], ["cargo", "check", "--workspace", "--tests", "--locked"])
        for needed in ("ci unit tests", "check_zenith_intact", "cargo test protocol", "bridge cargo test", "launch API check"):
            self.assertIn(needed, names)

    def test_release_decision(self):
        u, z, base = fork()
        git(z, "tag", "nightly-source/2026-11-05-aaaaaaa", base)
        pick = json.dumps({"tag": "nightly-source/2026-11-05-aaaaaaa", "release": "nightly"})
        self.assertTrue(sync.covered(z, base, "nightly-source/2026-11-05-aaaaaaa"))
        newer = upstream_commit(u, **{CORE: "pub fn core2() {}\n"})
        git(z, "fetch", "-q", "upstream")
        self.assertFalse(sync.covered(z, newer, "nightly-source/2026-11-05-aaaaaaa"))
        self.assertEqual(sync.release(z, newer, pick, True), 0)  # pending: prints, dispatches nothing


if __name__ == "__main__":
    unittest.main()
